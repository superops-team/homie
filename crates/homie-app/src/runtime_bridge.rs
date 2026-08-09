use std::collections::BTreeSet;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use homie_client::{
    ClientError, ClientOptions, ConnectionState, EventStream, EventStreamItem, HomieClient,
    StreamState, TerminalItem, TerminalStream,
};
use homie_proto::Method;
use homie_proto::grid::GridUpdate;
use homie_proto::model::{
    ArtifactScan, RuntimeEvent, SessionSummary, StateSnapshot, WorktreeOverviewResult,
};
use homie_proto::paths::RuntimeEndpoint;
use homie_proto::stream::{EventStreamOpen, TerminalStreamOpen};
use homie_proto::transport::ClientRole;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

use crate::daemon_launch::ensure_sibling_daemon;

const COMMAND_CAPACITY: usize = 256;
const EVENT_CAPACITY: usize = 256;
const WORKER_THREADS: usize = 2;
const WORKER_THREAD_NAME: &str = "homie-async";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeCommand {
    RefreshSessions,
    SpawnSession {
        cwd: PathBuf,
        title: Option<String>,
    },
    SelectSession {
        session_id: String,
        output_offset: u64,
    },
    SendText {
        session_id: String,
        text: String,
        submit: bool,
    },
    Resize {
        session_id: String,
        cols: u16,
        rows: u16,
    },
    RefreshArtifacts {
        session_id: String,
    },
    RefreshWorktrees,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BridgeConnectionState {
    Connecting,
    Connected,
    Degraded,
    Reconnecting,
    Unavailable,
    Shutdown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalOutput {
    pub offset: u64,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BridgeEvent {
    Connection(BridgeConnectionState),
    DaemonIdentity { instance_id: String },
    Snapshot(StateSnapshot),
    Sessions(Vec<SessionSummary>),
    RuntimeEvent(RuntimeEvent),
    SessionSpawned(SessionSummary),
    Artifacts(ArtifactScan),
    Worktrees(WorktreeOverviewResult),
    TerminalOutput(TerminalOutput),
    TerminalGrid(GridUpdate),
    TerminalAttached { session_id: String },
    TerminalUnavailable { last_confirmed_offset: u64 },
    CommandFailed { command: &'static str, code: String },
}

#[derive(Clone)]
pub struct BridgeEventSender {
    inner: mpsc::Sender<BridgeEvent>,
}

impl BridgeEventSender {
    pub async fn send(&self, event: BridgeEvent) -> Result<(), BridgeEventSendError> {
        self.inner
            .send(event)
            .await
            .map_err(|_| BridgeEventSendError::Closed)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeProjection {
    pub connection: BridgeConnectionState,
    pub runtime_available: bool,
    pub daemon_instance_id: Option<String>,
    pub sessions: Vec<SessionSummary>,
    pub selected_session_id: Option<String>,
    pub event_cursor: u64,
    pub artifacts: ArtifactScan,
    pub worktrees: WorktreeOverviewResult,
    pub terminal_output: Option<TerminalOutput>,
    pub terminal_grid: Option<GridUpdate>,
    pub terminal_retry_offset: Option<u64>,
    pub last_error_code: Option<String>,
}

impl Default for BridgeProjection {
    fn default() -> Self {
        Self {
            connection: BridgeConnectionState::Connecting,
            runtime_available: false,
            daemon_instance_id: None,
            sessions: Vec::new(),
            selected_session_id: None,
            event_cursor: 0,
            artifacts: ArtifactScan {
                artifacts: Vec::new(),
                ports: Vec::new(),
            },
            worktrees: WorktreeOverviewResult {
                entries: Vec::new(),
            },
            terminal_output: None,
            terminal_grid: None,
            terminal_retry_offset: None,
            last_error_code: None,
        }
    }
}

impl BridgeProjection {
    pub fn apply(&mut self, event: BridgeEvent) {
        match event {
            BridgeEvent::Connection(connection) => {
                self.runtime_available = connection == BridgeConnectionState::Connected;
                self.connection = connection;
            }
            BridgeEvent::DaemonIdentity { instance_id } => {
                self.daemon_instance_id = Some(instance_id);
            }
            BridgeEvent::Snapshot(snapshot) => {
                self.sessions = snapshot.sessions;
                self.event_cursor = snapshot.event_cursor;
                self.reconcile_selection();
            }
            BridgeEvent::Sessions(sessions) => {
                self.sessions = sessions;
                self.reconcile_selection();
            }
            BridgeEvent::RuntimeEvent(event) => {
                self.event_cursor = self.event_cursor.max(event.seq);
                if let (Some(session_id), Some(status)) = (event.session_id, event.status)
                    && let Some(session) = self
                        .sessions
                        .iter_mut()
                        .find(|session| session.id == session_id)
                {
                    session.status = status;
                }
            }
            BridgeEvent::SessionSpawned(session) => {
                self.selected_session_id = Some(session.id.clone());
                if let Some(existing) = self
                    .sessions
                    .iter_mut()
                    .find(|existing| existing.id == session.id)
                {
                    *existing = session;
                } else {
                    self.sessions.push(session);
                }
            }
            BridgeEvent::Artifacts(artifacts) => self.artifacts = artifacts,
            BridgeEvent::Worktrees(worktrees) => self.worktrees = worktrees,
            BridgeEvent::TerminalOutput(output) => {
                self.terminal_retry_offset =
                    Some(output.offset.saturating_add(output.bytes.len() as u64));
                self.terminal_output = Some(output);
            }
            BridgeEvent::TerminalGrid(grid) => {
                self.terminal_retry_offset = None;
                self.terminal_grid = Some(grid);
            }
            BridgeEvent::TerminalAttached { .. } => {}
            BridgeEvent::TerminalUnavailable {
                last_confirmed_offset,
            } => {
                self.terminal_retry_offset = Some(last_confirmed_offset);
            }
            BridgeEvent::CommandFailed { code, .. } => {
                self.last_error_code = Some(code);
            }
        }
    }

    fn reconcile_selection(&mut self) {
        let selection_exists = self
            .selected_session_id
            .as_ref()
            .is_some_and(|selected| self.sessions.iter().any(|session| &session.id == selected));
        if !selection_exists {
            self.selected_session_id = self.sessions.first().map(|session| session.id.clone());
        }
    }
}

pub trait BridgeDriver: Send {
    fn run(
        self: Box<Self>,
        commands: mpsc::Receiver<RuntimeCommand>,
        events: BridgeEventSender,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeFacts {
    pub configured_worker_threads: usize,
    pub observed_worker_threads: usize,
    pub worker_thread_names: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct RuntimeBridgeConfig {
    pub data_dir: PathBuf,
    pub current_executable: PathBuf,
    pub workspace: PathBuf,
    pub startup_probe_timeout: Duration,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
}

pub struct RuntimeBridge {
    runtime: Option<Runtime>,
    driver_task: Option<tokio::task::JoinHandle<()>>,
    commands: mpsc::Sender<RuntimeCommand>,
    events: mpsc::Receiver<BridgeEvent>,
    projection: BridgeProjection,
    worker_thread_names: Arc<Mutex<BTreeSet<String>>>,
}

impl RuntimeBridge {
    pub fn start(config: RuntimeBridgeConfig) -> Result<Self, BridgeStartError> {
        Self::start_with_driver(Box::new(ProductionBridgeDriver { config }))
    }

    pub fn start_with_driver(driver: Box<dyn BridgeDriver>) -> Result<Self, BridgeStartError> {
        let worker_thread_names = Arc::new(Mutex::new(BTreeSet::new()));
        let observed_names = worker_thread_names.clone();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(WORKER_THREADS)
            .thread_name(WORKER_THREAD_NAME)
            .on_thread_start(move || {
                let Some(name) = std::thread::current().name().map(str::to_string) else {
                    return;
                };
                observed_names
                    .lock()
                    .expect("runtime worker facts lock poisoned")
                    .insert(format!("{name}:{:?}", std::thread::current().id()));
            })
            .enable_all()
            .build()?;
        let (command_tx, command_rx) = mpsc::channel(COMMAND_CAPACITY);
        let (event_tx, event_rx) = mpsc::channel(EVENT_CAPACITY);
        let driver_task = runtime.spawn(async move {
            driver
                .run(command_rx, BridgeEventSender { inner: event_tx })
                .await;
        });
        Ok(Self {
            runtime: Some(runtime),
            driver_task: Some(driver_task),
            commands: command_tx,
            events: event_rx,
            projection: BridgeProjection::default(),
            worker_thread_names,
        })
    }

    pub fn dispatch(&self, command: RuntimeCommand) -> Result<(), BridgeDispatchError> {
        self.commands
            .try_send(command)
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => BridgeDispatchError::Backpressure,
                mpsc::error::TrySendError::Closed(_) => BridgeDispatchError::Unavailable,
            })
    }

    pub fn drain(&mut self) -> bool {
        !self.drain_events().is_empty()
    }

    pub fn drain_events(&mut self) -> Vec<BridgeEvent> {
        self.reap_driver();
        let mut drained = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            self.projection.apply(event.clone());
            drained.push(event);
        }
        drained
    }

    fn reap_driver(&mut self) {
        let Some(task) = self.driver_task.as_ref() else {
            return;
        };
        if !task.is_finished() {
            return;
        }
        let task = self.driver_task.take().expect("finished driver task");
        let result = self
            .runtime
            .as_ref()
            .expect("bridge runtime")
            .block_on(task);
        if let Err(error) = result
            && !error.is_cancelled()
        {
            self.projection
                .apply(BridgeEvent::Connection(BridgeConnectionState::Unavailable));
            self.projection.apply(BridgeEvent::CommandFailed {
                command: "runtime.bridge",
                code: "internal".to_string(),
            });
        }
    }

    #[must_use]
    pub fn projection(&self) -> &BridgeProjection {
        &self.projection
    }

    #[must_use]
    pub fn runtime_facts(&self) -> RuntimeFacts {
        let names = self
            .worker_thread_names
            .lock()
            .expect("runtime worker facts lock poisoned");
        let worker_thread_names = names
            .iter()
            .map(|name| {
                name.split_once(':')
                    .map_or_else(|| name.clone(), |(name, _)| name.to_string())
            })
            .collect::<Vec<_>>();
        RuntimeFacts {
            configured_worker_threads: WORKER_THREADS,
            observed_worker_threads: names.len(),
            worker_thread_names,
        }
    }
}

impl Drop for RuntimeBridge {
    fn drop(&mut self) {
        if let Some(task) = self.driver_task.take() {
            task.abort();
        }
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}

struct ProductionBridgeDriver {
    config: RuntimeBridgeConfig,
}

impl BridgeDriver for ProductionBridgeDriver {
    fn run(
        self: Box<Self>,
        commands: mpsc::Receiver<RuntimeCommand>,
        events: BridgeEventSender,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(run_production_bridge(self.config, commands, events))
    }
}

async fn run_production_bridge(
    config: RuntimeBridgeConfig,
    commands: mpsc::Receiver<RuntimeCommand>,
    events: BridgeEventSender,
) {
    if publish(
        &events,
        BridgeEvent::Connection(BridgeConnectionState::Connecting),
    )
    .await
    .is_err()
    {
        return;
    }
    let paths = match ensure_sibling_daemon(
        config.data_dir,
        config.current_executable,
        config.startup_probe_timeout,
    )
    .await
    {
        Ok(paths) => paths,
        Err(error) => {
            let _ = publish(
                &events,
                BridgeEvent::CommandFailed {
                    command: "runtime.launch",
                    code: error.code().to_string(),
                },
            )
            .await;
            let _ = publish(
                &events,
                BridgeEvent::Connection(BridgeConnectionState::Unavailable),
            )
            .await;
            return;
        }
    };
    let endpoint = match RuntimeEndpoint::new(paths.socket) {
        Ok(endpoint) => endpoint,
        Err(_) => {
            let _ = publish_startup_failure(&events, "bad_request").await;
            return;
        }
    };
    let client = match HomieClient::connect(ClientOptions {
        endpoint,
        role: ClientRole::App,
        connect_timeout: config.connect_timeout,
        request_timeout: config.request_timeout,
    })
    .await
    {
        Ok(client) => client,
        Err(error) => {
            let _ = publish_startup_failure(&events, error.code()).await;
            return;
        }
    };

    let mut connection = client.connection_state();
    let initial_connection = connection.borrow().clone();
    if publish_connection(&events, &initial_connection)
        .await
        .is_err()
    {
        let _ = client.close().await;
        return;
    }

    let mut event_stream = match restore_event_stream(&client, &events).await {
        Ok(stream) => stream,
        Err(_) => {
            let _ = client.close().await;
            return;
        }
    };

    drive_connected(
        &client,
        config.workspace,
        commands,
        &events,
        &mut connection,
        &mut event_stream,
    )
    .await;
    let _ = client.close().await;
    let _ = publish(
        &events,
        BridgeEvent::Connection(BridgeConnectionState::Shutdown),
    )
    .await;
}

async fn drive_connected(
    client: &HomieClient,
    workspace: PathBuf,
    mut commands: mpsc::Receiver<RuntimeCommand>,
    events: &BridgeEventSender,
    connection: &mut tokio::sync::watch::Receiver<ConnectionState>,
    event_stream: &mut Option<EventStream>,
) {
    let mut terminal: Option<TerminalBinding> = None;
    let mut terminal_state: Option<tokio::sync::watch::Receiver<StreamState>> = None;
    loop {
        tokio::select! {
            command = commands.recv() => {
                let Some(command) = command else {
                    return;
                };
                if handle_command(
                    client,
                    &workspace,
                    command,
                    events,
                    &mut terminal,
                    &mut terminal_state,
                )
                .await
                .is_err()
                {
                    return;
                }
            }
            changed = connection.changed() => {
                if changed.is_err() {
                    return;
                }
                let current_connection = connection.borrow().clone();
                if publish_connection(events, &current_connection).await.is_err() {
                    return;
                }
                if matches!(current_connection, ConnectionState::Connected { .. })
                    && event_stream.is_none()
                {
                    *event_stream = match restore_event_stream(client, events).await {
                        Ok(stream) => stream,
                        Err(_) => return,
                    };
                }
            }
            item = next_event_item(event_stream) => {
                let update = match item {
                    Some(Ok(Some(EventStreamItem::Event(event)))) => {
                        Some(BridgeEvent::RuntimeEvent(event))
                    }
                    Some(Ok(Some(EventStreamItem::Snapshot(snapshot)))) => {
                        Some(BridgeEvent::Snapshot(snapshot))
                    }
                    Some(Ok(None) | Err(_)) => {
                        *event_stream = None;
                        None
                    }
                    None => None,
                };
                if let Some(update) = update
                    && publish(events, update).await.is_err()
                {
                    return
                }
            }
            () = tokio::time::sleep(Duration::from_millis(250)),
                if event_stream.is_none()
                    && matches!(*connection.borrow(), ConnectionState::Connected { .. }) =>
            {
                *event_stream = match restore_event_stream(client, events).await {
                    Ok(stream) => stream,
                    Err(_) => return,
                };
            }
            item = next_terminal_item(&mut terminal) => {
                let update = match item {
                    Some(Ok(Some(TerminalItem::Output { offset, bytes }))) => {
                        Some(BridgeEvent::TerminalOutput(TerminalOutput { offset, bytes }))
                    }
                    Some(Ok(Some(TerminalItem::Grid(grid)))) => {
                        Some(BridgeEvent::TerminalGrid(grid))
                    }
                    Some(Ok(Some(
                        TerminalItem::ReplayBegin(_)
                        | TerminalItem::ReplayEnd(_)
                        | TerminalItem::Modes(_),
                    ))) => None,
                    Some(Ok(None) | Err(_)) => {
                        let last_confirmed_offset = terminal
                            .as_ref()
                            .map_or(0, |binding| binding.stream.last_confirmed_offset());
                        terminal = None;
                        terminal_state = None;
                        Some(BridgeEvent::TerminalUnavailable {
                            last_confirmed_offset,
                        })
                    }
                    None => None,
                };
                if let Some(update) = update
                    && publish(events, update).await.is_err()
                {
                    return
                }
            }
            state = next_terminal_state(&mut terminal_state) => {
                if let Some(state) = state {
                    match state {
                        StreamState::ResyncRequired {
                            last_confirmed_offset: Some(last_confirmed_offset),
                        } => {
                            let _ = publish(
                                events,
                                BridgeEvent::TerminalUnavailable {
                                    last_confirmed_offset,
                                },
                            )
                            .await;
                        }
                        StreamState::Closed => {
                            let last_confirmed_offset = terminal
                                .as_ref()
                                .map_or(0, |binding| binding.stream.last_confirmed_offset());
                            terminal = None;
                            terminal_state = None;
                            if publish(
                                events,
                                BridgeEvent::TerminalUnavailable {
                                    last_confirmed_offset,
                                },
                            )
                            .await
                            .is_err()
                            {
                                return;
                            }
                        }
                        StreamState::Opening
                        | StreamState::Open
                        | StreamState::Reconnecting
                        | StreamState::ResyncRequired {
                            last_confirmed_offset: None,
                        } => {}
                    }
                }
            }
        }
    }
}

async fn handle_command(
    client: &HomieClient,
    workspace: &std::path::Path,
    command: RuntimeCommand,
    events: &BridgeEventSender,
    terminal: &mut Option<TerminalBinding>,
    terminal_state: &mut Option<tokio::sync::watch::Receiver<StreamState>>,
) -> Result<(), BridgeEventSendError> {
    match command {
        RuntimeCommand::RefreshSessions => match client.list_sessions().await {
            Ok(sessions) => publish(events, BridgeEvent::Sessions(sessions)).await,
            Err(error) => publish_failure(events, "session.list", &error).await,
        },
        RuntimeCommand::SpawnSession { cwd, title } => {
            let cwd = if cwd.is_absolute() {
                cwd
            } else {
                workspace.to_path_buf()
            };
            match client.spawn_shell(&cwd, title.as_deref()).await {
                Ok(session) => publish(events, BridgeEvent::SessionSpawned(session)).await,
                Err(error) => publish_failure(events, "session.spawn", &error).await,
            }
        }
        RuntimeCommand::SelectSession {
            session_id,
            output_offset,
        } => {
            match client
                .open_terminal(TerminalStreamOpen {
                    session_id: session_id.clone(),
                    output_offset,
                    client_role: ClientRole::App,
                    last_grid_sequence: None,
                })
                .await
            {
                Ok(stream) => {
                    *terminal_state = Some(stream.state());
                    *terminal = Some(TerminalBinding {
                        session_id: session_id.clone(),
                        stream,
                    });
                    publish(events, BridgeEvent::TerminalAttached { session_id }).await
                }
                Err(error) => publish_failure(events, "terminal.open", &error).await,
            }
        }
        RuntimeCommand::SendText {
            session_id,
            text,
            submit,
        } => {
            if let Err(error) = client.send_text(&session_id, &text, submit).await {
                publish_failure(events, "session.send_text", &error).await
            } else {
                Ok(())
            }
        }
        RuntimeCommand::Resize {
            session_id,
            cols,
            rows,
        } => {
            let result = if let Some(binding) = terminal
                && binding.session_id == session_id
            {
                binding.stream.resize(cols, rows)
            } else {
                client.resize_session(&session_id, cols, rows).await
            };
            if let Err(error) = result {
                publish_failure(events, "session.resize", &error).await
            } else {
                Ok(())
            }
        }
        RuntimeCommand::RefreshArtifacts { session_id } => {
            match client.scan_session_artifacts(&session_id).await {
                Ok(artifacts) => publish(events, BridgeEvent::Artifacts(artifacts)).await,
                Err(error) => publish_failure(events, "session.artifacts", &error).await,
            }
        }
        RuntimeCommand::RefreshWorktrees => match client.worktree_overview().await {
            Ok(worktrees) => publish(events, BridgeEvent::Worktrees(worktrees)).await,
            Err(error) => publish_failure(events, "worktree.overview", &error).await,
        },
    }
}

async fn restore_event_stream(
    client: &HomieClient,
    events: &BridgeEventSender,
) -> Result<Option<EventStream>, BridgeEventSendError> {
    let snapshot = match client
        .request::<_, StateSnapshot>(Method::STATE_SNAPSHOT, serde_json::json!({}))
        .await
    {
        Ok(snapshot) => snapshot,
        Err(error) => {
            publish_failure(events, "state.snapshot", &error).await?;
            return Ok(None);
        }
    };
    let cursor = snapshot.event_cursor;
    publish(events, BridgeEvent::Snapshot(snapshot)).await?;
    match client
        .subscribe_events(EventStreamOpen {
            after_seq: cursor,
            event_filter: Vec::new(),
        })
        .await
    {
        Ok(stream) => Ok(Some(stream)),
        Err(error) => {
            publish_failure(events, "events.subscribe", &error).await?;
            Ok(None)
        }
    }
}

struct TerminalBinding {
    session_id: String,
    stream: TerminalStream,
}

async fn next_event_item(
    stream: &mut Option<EventStream>,
) -> Option<Result<Option<EventStreamItem>, ClientError>> {
    match stream {
        Some(stream) => Some(stream.recv().await),
        None => std::future::pending().await,
    }
}

async fn next_terminal_item(
    terminal: &mut Option<TerminalBinding>,
) -> Option<Result<Option<TerminalItem>, ClientError>> {
    match terminal {
        Some(terminal) => Some(terminal.stream.recv().await),
        None => std::future::pending().await,
    }
}

async fn next_terminal_state(
    state: &mut Option<tokio::sync::watch::Receiver<StreamState>>,
) -> Option<StreamState> {
    let Some(state) = state.as_mut() else {
        return std::future::pending().await;
    };
    if state.changed().await.is_err() {
        return Some(StreamState::Closed);
    }
    Some(state.borrow_and_update().clone())
}

fn map_connection_state(state: &ConnectionState) -> BridgeConnectionState {
    match state {
        ConnectionState::Disconnected => BridgeConnectionState::Unavailable,
        ConnectionState::Connecting | ConnectionState::Handshaking => {
            BridgeConnectionState::Connecting
        }
        ConnectionState::Connected { .. } => BridgeConnectionState::Connected,
        ConnectionState::Degraded { .. } => BridgeConnectionState::Degraded,
        ConnectionState::Reconnecting { .. } => BridgeConnectionState::Reconnecting,
        ConnectionState::Shutdown => BridgeConnectionState::Shutdown,
    }
}

async fn publish_connection(
    events: &BridgeEventSender,
    state: &ConnectionState,
) -> Result<(), BridgeEventSendError> {
    publish(events, BridgeEvent::Connection(map_connection_state(state))).await?;
    if let ConnectionState::Connected {
        daemon_instance_id, ..
    } = state
    {
        publish(
            events,
            BridgeEvent::DaemonIdentity {
                instance_id: daemon_instance_id.clone(),
            },
        )
        .await?;
    }
    Ok(())
}

async fn publish(
    events: &BridgeEventSender,
    event: BridgeEvent,
) -> Result<(), BridgeEventSendError> {
    events.send(event).await
}

async fn publish_startup_failure(
    events: &BridgeEventSender,
    code: &str,
) -> Result<(), BridgeEventSendError> {
    publish(
        events,
        BridgeEvent::CommandFailed {
            command: "runtime.connect",
            code: code.to_string(),
        },
    )
    .await?;
    publish(
        events,
        BridgeEvent::Connection(BridgeConnectionState::Unavailable),
    )
    .await
}

async fn publish_failure(
    events: &BridgeEventSender,
    command: &'static str,
    error: &ClientError,
) -> Result<(), BridgeEventSendError> {
    publish(
        events,
        BridgeEvent::CommandFailed {
            command,
            code: error.code().to_string(),
        },
    )
    .await
}

#[derive(Debug, thiserror::Error)]
pub enum BridgeStartError {
    #[error("failed to create the app async runtime: {0}")]
    Runtime(#[from] std::io::Error),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum BridgeDispatchError {
    #[error("runtime bridge command queue is full")]
    Backpressure,
    #[error("runtime bridge is unavailable")]
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum BridgeEventSendError {
    #[error("runtime bridge event receiver is closed")]
    Closed,
}
