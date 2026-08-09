use std::fs;
use std::future::{Future, pending};
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use homie_app::daemon_launch::resolve_daemon_executable;
use homie_app::runtime_bridge::{
    BridgeConnectionState, BridgeDriver, BridgeEvent, BridgeEventSender, BridgeProjection,
    RuntimeBridge, RuntimeBridgeConfig, RuntimeCommand,
};
use homie_proto::grid::GridUpdate;
use homie_proto::model::{
    ArtifactKind, ArtifactScan, ListeningPort, RuntimeEvent, SessionArtifact, SessionSummary,
    StateSnapshot, WorktreeOverviewEntry, WorktreeOverviewResult,
};
use homie_proto::stream::{StreamKind, StreamOpenRequest};
use homie_proto::transport::{
    AckResult, ClientRole, EndpointRole, FRAME_HEADER_LEN, Frame, FrameHeader, FrameKind,
    HelloResponse, MAX_FRAME_LEN, PREFACE_LEN, Preface, WIRE_MAJOR, WIRE_MINOR,
};
use homie_proto::{ControlMessage, Method, RequestId};
use sha2::{Digest as _, Sha256};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc as tokio_mpsc;

#[test]
fn runtime_bridge_owns_exactly_two_named_workers() {
    let bridge =
        RuntimeBridge::start_with_driver(Box::new(PendingDriver)).expect("start runtime bridge");
    let deadline = Instant::now() + Duration::from_secs(2);

    let facts = loop {
        let facts = bridge.runtime_facts();
        if facts.observed_worker_threads == 2 {
            break facts;
        }
        assert!(
            Instant::now() < deadline,
            "Tokio workers did not start: {facts:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    };

    assert_eq!(facts.configured_worker_threads, 2);
    assert_eq!(facts.observed_worker_threads, 2);
    assert_eq!(
        facts.worker_thread_names,
        vec!["homie-async".to_string(), "homie-async".to_string()]
    );
}

#[test]
fn first_frame_projection_does_not_wait_for_background_driver() {
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();

    let bridge = RuntimeBridge::start_with_driver(Box::new(GatedDriver {
        started_tx,
        release_rx,
    }))
    .expect("start runtime bridge");

    started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("driver started");
    assert_eq!(
        bridge.projection(),
        &BridgeProjection {
            connection: BridgeConnectionState::Connecting,
            runtime_available: false,
            ..BridgeProjection::default()
        }
    );

    release_tx.send(()).expect("release driver");
}

#[test]
fn mock_driver_commands_update_authoritative_session_projection() {
    let (command_tx, command_rx) = mpsc::channel();
    let mut bridge = RuntimeBridge::start_with_driver(Box::new(RecordingDriver { command_tx }))
        .expect("start runtime bridge");

    bridge
        .dispatch(RuntimeCommand::RefreshSessions)
        .expect("queue refresh");
    assert_eq!(
        command_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("recorded command"),
        RuntimeCommand::RefreshSessions
    );

    wait_for_projection(&mut bridge, |projection| {
        projection.sessions == vec![session("session-1", "running")] && projection.event_cursor == 7
    });

    assert_eq!(
        bridge.projection().sessions,
        vec![session("session-1", "running")]
    );
}

#[test]
fn full_event_queue_waits_for_drain_and_driver_continues_accepting_commands() {
    let (progress_tx, progress_rx) = mpsc::channel();
    let (command_tx, command_rx) = mpsc::channel();
    let mut bridge = RuntimeBridge::start_with_driver(Box::new(BackpressureDriver {
        progress_tx,
        command_tx,
    }))
    .expect("start runtime bridge");

    assert_eq!(
        progress_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("event queue filled"),
        256
    );
    assert!(progress_rx.try_recv().is_err(), "producer did not wait");

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut events = Vec::new();
    while events.len() < 300 {
        events.extend(bridge.drain_events());
        assert!(
            Instant::now() < deadline,
            "did not drain all events: {}",
            events.len()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(events.len(), 300);

    bridge
        .dispatch(RuntimeCommand::RefreshSessions)
        .expect("driver remains available");
    assert_eq!(
        command_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("driver received command"),
        RuntimeCommand::RefreshSessions
    );
}

#[test]
fn driver_panic_is_observed_as_runtime_unavailable() {
    let mut bridge =
        RuntimeBridge::start_with_driver(Box::new(PanicDriver)).expect("start runtime bridge");
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        bridge.drain_events();
        if bridge.projection().connection == BridgeConnectionState::Unavailable
            && bridge.projection().last_error_code.as_deref() == Some("internal")
        {
            break;
        }
        assert!(Instant::now() < deadline, "driver panic was not observed");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn runtime_events_update_existing_session_status_without_replacing_snapshot() {
    let mut projection = BridgeProjection::default();
    projection.apply(BridgeEvent::Snapshot(StateSnapshot {
        sessions: vec![session("session-1", "starting")],
        event_cursor: 4,
    }));

    projection.apply(BridgeEvent::RuntimeEvent(RuntimeEvent {
        seq: 5,
        event: "session.updated".to_string(),
        session_id: Some("session-1".to_string()),
        status: Some("running".to_string()),
    }));

    assert_eq!(
        (projection.event_cursor, projection.sessions),
        (5, vec![session("session-1", "running")])
    );
}

#[test]
fn daemon_path_is_the_canonical_absolute_current_executable_sibling() {
    let temp = tempfile::tempdir().expect("tempdir");
    let bin_dir = temp.path().join("bin");
    fs::create_dir(&bin_dir).expect("bin dir");
    let current_executable = bin_dir.join("homie-app");
    let daemon_executable = bin_dir.join("homie-runtime-daemon");
    write_executable(&current_executable);
    write_executable(&daemon_executable);

    let resolved =
        resolve_daemon_executable(&current_executable).expect("canonical sibling daemon");

    assert_eq!(
        resolved,
        fs::canonicalize(daemon_executable).expect("canonical daemon")
    );
    assert!(resolved.is_absolute());
}

#[test]
fn bundled_daemon_path_is_the_fixed_canonical_resources_binary() {
    let temp = tempfile::tempdir().expect("tempdir");
    let contents = temp.path().join("Homie.app/Contents");
    let macos = contents.join("MacOS");
    let resources_bin = contents.join("Resources/bin");
    fs::create_dir_all(&macos).expect("MacOS dir");
    fs::create_dir_all(&resources_bin).expect("Resources bin dir");
    let current_executable = macos.join("Homie");
    let daemon_executable = resources_bin.join("homie-runtime-daemon");
    write_executable(&current_executable);
    write_executable(&daemon_executable);

    let resolved =
        resolve_daemon_executable(&current_executable).expect("canonical bundled daemon");

    assert_eq!(
        resolved,
        fs::canonicalize(daemon_executable).expect("canonical bundled daemon")
    );
}

#[test]
fn bundled_daemon_resolution_does_not_fall_back_to_macos_sibling() {
    let temp = tempfile::tempdir().expect("tempdir");
    let macos = temp.path().join("Homie.app/Contents/MacOS");
    fs::create_dir_all(&macos).expect("MacOS dir");
    let current_executable = macos.join("Homie");
    write_executable(&current_executable);
    write_executable(&macos.join("homie-runtime-daemon"));

    resolve_daemon_executable(&current_executable)
        .expect_err("bundle must require Resources/bin daemon");
}

#[test]
fn production_bridge_projects_snapshot_and_event_from_local_daemon() {
    let fixture = ProductionFixture::new();
    let server = fixture.start_server(false);
    let mut bridge = RuntimeBridge::start(RuntimeBridgeConfig {
        data_dir: fixture.data_dir.clone(),
        current_executable: fixture.current_executable.clone(),
        workspace: PathBuf::from("/tmp/workspace"),
        startup_probe_timeout: Duration::from_secs(1),
        connect_timeout: Duration::from_secs(2),
        request_timeout: Duration::from_secs(1),
    })
    .expect("start production bridge");

    wait_for_projection(&mut bridge, |projection| {
        projection.connection == BridgeConnectionState::Connected
            && projection.daemon_instance_id.as_deref() == Some("daemon-app-test")
            && projection.event_cursor == 9
            && projection.sessions == vec![session("session-1", "idle")]
    });

    drop(bridge);
    server.join().expect("mock daemon");
}

#[test]
fn production_bridge_routes_runtime_commands_and_terminal_stream() {
    let fixture = ProductionFixture::new();
    let server = fixture.start_server(true);
    let mut bridge = RuntimeBridge::start(RuntimeBridgeConfig {
        data_dir: fixture.data_dir.clone(),
        current_executable: fixture.current_executable.clone(),
        workspace: PathBuf::from("/tmp/workspace"),
        startup_probe_timeout: Duration::from_secs(1),
        connect_timeout: Duration::from_secs(2),
        request_timeout: Duration::from_secs(1),
    })
    .expect("start production bridge");
    wait_for_projection(&mut bridge, |projection| {
        projection.connection == BridgeConnectionState::Connected && projection.event_cursor == 8
    });

    for command in [
        RuntimeCommand::RefreshSessions,
        RuntimeCommand::SpawnSession {
            cwd: PathBuf::from("/tmp/workspace"),
            title: Some("Second shell".to_string()),
        },
        RuntimeCommand::SendText {
            session_id: "session-1".to_string(),
            text: "pwd".to_string(),
            submit: true,
        },
        RuntimeCommand::SelectSession {
            session_id: "session-1".to_string(),
            output_offset: 17,
        },
        RuntimeCommand::Resize {
            session_id: "session-1".to_string(),
            cols: 120,
            rows: 40,
        },
        RuntimeCommand::RefreshArtifacts {
            session_id: "session-1".to_string(),
        },
        RuntimeCommand::RefreshWorktrees,
    ] {
        bridge.dispatch(command).expect("dispatch runtime command");
    }

    let deadline = Instant::now() + Duration::from_secs(1);
    let mut terminal_attached = false;
    loop {
        terminal_attached |= bridge.drain_events().into_iter().any(|event| {
            event
                == BridgeEvent::TerminalAttached {
                    session_id: "session-1".to_string(),
                }
        });
        let projection = bridge.projection();
        if terminal_attached
            && projection
                .sessions
                .iter()
                .any(|session| session.id == "session-2")
            && projection.artifacts.ports.len() == 1
            && projection.worktrees.entries.len() == 1
            && projection.terminal_grid == Some(empty_full_grid())
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "runtime commands or terminal attach were not projected"
        );
        std::thread::sleep(Duration::from_millis(10));
    }

    drop(bridge);
    server.join().expect("mock daemon");
}

struct PendingDriver;

impl BridgeDriver for PendingDriver {
    fn run(
        self: Box<Self>,
        _commands: tokio_mpsc::Receiver<RuntimeCommand>,
        _events: BridgeEventSender,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(pending())
    }
}

struct PanicDriver;

impl BridgeDriver for PanicDriver {
    fn run(
        self: Box<Self>,
        _commands: tokio_mpsc::Receiver<RuntimeCommand>,
        _events: BridgeEventSender,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            panic!("intentional bridge driver panic");
        })
    }
}

struct GatedDriver {
    started_tx: mpsc::Sender<()>,
    release_rx: mpsc::Receiver<()>,
}

impl BridgeDriver for GatedDriver {
    fn run(
        self: Box<Self>,
        _commands: tokio_mpsc::Receiver<RuntimeCommand>,
        _events: BridgeEventSender,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            self.started_tx.send(()).expect("report driver start");
            tokio::task::spawn_blocking(move || self.release_rx.recv())
                .await
                .expect("join gate")
                .expect("release driver");
        })
    }
}

struct RecordingDriver {
    command_tx: mpsc::Sender<RuntimeCommand>,
}

impl BridgeDriver for RecordingDriver {
    fn run(
        self: Box<Self>,
        mut commands: tokio_mpsc::Receiver<RuntimeCommand>,
        events: BridgeEventSender,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            let Some(command) = commands.recv().await else {
                return;
            };
            self.command_tx.send(command).expect("record command");
            events
                .send(BridgeEvent::Snapshot(StateSnapshot {
                    sessions: vec![session("session-1", "running")],
                    event_cursor: 7,
                }))
                .await
                .expect("send snapshot");
        })
    }
}

struct BackpressureDriver {
    progress_tx: mpsc::Sender<u64>,
    command_tx: mpsc::Sender<RuntimeCommand>,
}

impl BridgeDriver for BackpressureDriver {
    fn run(
        self: Box<Self>,
        mut commands: tokio_mpsc::Receiver<RuntimeCommand>,
        events: BridgeEventSender,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            for seq in 1..=300 {
                events
                    .send(BridgeEvent::RuntimeEvent(RuntimeEvent {
                        seq,
                        event: "session.updated".to_string(),
                        session_id: None,
                        status: None,
                    }))
                    .await
                    .expect("event receiver remains open");
                if seq == 256 {
                    self.progress_tx.send(seq).expect("report full queue");
                }
            }
            let Some(command) = commands.recv().await else {
                return;
            };
            self.command_tx.send(command).expect("record command");
        })
    }
}

fn wait_for_projection(bridge: &mut RuntimeBridge, ready: impl Fn(&BridgeProjection) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        bridge.drain();
        if ready(bridge.projection()) {
            return;
        }
        assert!(Instant::now() < deadline, "projection was not updated");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn session(id: &str, status: &str) -> SessionSummary {
    SessionSummary {
        id: id.to_string(),
        title: "Homie shell".to_string(),
        status: status.to_string(),
        workspace: "/tmp/workspace".to_string(),
        agent_profile_id: "agent".to_string(),
        runtime_id: "runtime".to_string(),
        llm_profile_id: "llm".to_string(),
        permission_profile_id: "permission".to_string(),
    }
}

struct ProductionFixture {
    _temp: tempfile::TempDir,
    data_dir: PathBuf,
    current_executable: PathBuf,
    daemon_executable: PathBuf,
}

impl ProductionFixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("tempdir");
        let data_dir = temp.path().join("data");
        let runtime_dir = data_dir.join("runtime");
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&runtime_dir).expect("runtime dir");
        fs::set_permissions(&runtime_dir, fs::Permissions::from_mode(0o700)).expect("runtime mode");
        fs::create_dir(&bin_dir).expect("bin dir");
        let current_executable = bin_dir.join("homie-app");
        let daemon_executable = bin_dir.join("homie-runtime-daemon");
        write_executable(&current_executable);
        write_executable(&daemon_executable);
        Self {
            _temp: temp,
            data_dir,
            current_executable,
            daemon_executable,
        }
    }

    fn start_server(&self, command_mode: bool) -> thread::JoinHandle<()> {
        let endpoint = self.data_dir.join("runtime/daemon.sock");
        let executable_hash = file_hash(&self.daemon_executable);
        let (ready_tx, ready_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("mock runtime");
            runtime.block_on(async move {
                let listener = UnixListener::bind(endpoint).expect("bind mock daemon");
                ready_tx.send(()).expect("server ready");
                tokio::time::timeout(Duration::from_secs(5), async {
                    serve_launcher_probe(&listener, &executable_hash).await;
                    serve_app_client(&listener, &executable_hash, command_mode).await;
                })
                .await
                .expect("mock daemon interaction timeout");
            });
        });
        ready_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("mock daemon ready");
        server
    }
}

async fn serve_launcher_probe(listener: &UnixListener, executable_hash: &str) {
    let (mut stream, _) = listener.accept().await.expect("accept launcher");
    read_hello(&mut stream, ClientRole::Cli).await;
    write_hello(&mut stream, executable_hash).await;
}

async fn serve_app_client(listener: &UnixListener, executable_hash: &str, command_mode: bool) {
    let (mut stream, _) = listener.accept().await.expect("accept app");
    read_hello(&mut stream, ClientRole::App).await;
    write_hello(&mut stream, executable_hash).await;

    let snapshot_request = read_frame(&mut stream).await;
    let (message_id, method) = request_identity(&snapshot_request);
    assert_eq!(method, Method::STATE_SNAPSHOT);
    write_response(
        &mut stream,
        message_id,
        serde_json::to_value(StateSnapshot {
            sessions: vec![session("session-1", "starting")],
            event_cursor: 7,
        })
        .expect("snapshot"),
    )
    .await;

    let open = read_frame(&mut stream).await;
    assert_eq!(open.header.kind, FrameKind::StreamOpen);
    let request: StreamOpenRequest = serde_json::from_slice(&open.payload).expect("stream open");
    assert!(matches!(request, StreamOpenRequest::Events(_)));
    write_frame(
        &mut stream,
        Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind: FrameKind::StreamOpened,
                flags: 0,
                stream_id: open.header.stream_id,
                message_id: 0,
                sequence: 0,
            },
            payload: serde_json::to_vec(&serde_json::json!({})).expect("opened"),
        },
    )
    .await;
    write_frame(
        &mut stream,
        Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind: FrameKind::Event,
                flags: 0,
                stream_id: open.header.stream_id,
                message_id: 0,
                sequence: 8,
            },
            payload: serde_json::to_vec(&RuntimeEvent {
                seq: 8,
                event: "session.updated".to_string(),
                session_id: Some("session-1".to_string()),
                status: Some("running".to_string()),
            })
            .expect("event"),
        },
    )
    .await;

    if command_mode {
        serve_runtime_commands(&mut stream, open.header.stream_id).await;
        return;
    }
    write_stream_frame(
        &mut stream,
        FrameKind::StreamClose,
        open.header.stream_id,
        0,
        Vec::new(),
    )
    .await;

    let snapshot_request = read_frame(&mut stream).await;
    let (message_id, method) = request_identity(&snapshot_request);
    assert_eq!(method, Method::STATE_SNAPSHOT);
    write_response(
        &mut stream,
        message_id,
        serde_json::to_value(StateSnapshot {
            sessions: vec![session("session-1", "running")],
            event_cursor: 8,
        })
        .expect("recovery snapshot"),
    )
    .await;

    let reopened = read_frame(&mut stream).await;
    assert_eq!(reopened.header.kind, FrameKind::StreamOpen);
    let request: StreamOpenRequest =
        serde_json::from_slice(&reopened.payload).expect("reopened event stream");
    let StreamOpenRequest::Events(request) = request else {
        panic!("expected reopened event stream");
    };
    assert_eq!(request.after_seq, 8);
    write_stream_frame(
        &mut stream,
        FrameKind::StreamOpened,
        reopened.header.stream_id,
        0,
        serde_json::to_vec(&serde_json::json!({})).expect("reopened"),
    )
    .await;
    write_stream_frame(
        &mut stream,
        FrameKind::Event,
        reopened.header.stream_id,
        9,
        serde_json::to_vec(&RuntimeEvent {
            seq: 9,
            event: "session.updated".to_string(),
            session_id: Some("session-1".to_string()),
            status: Some("idle".to_string()),
        })
        .expect("recovered event"),
    )
    .await;
    let _ = tokio::time::timeout(Duration::from_secs(2), stream.read_u8()).await;
}

async fn serve_runtime_commands(stream: &mut UnixStream, event_stream_id: u32) {
    let list = read_frame(stream).await;
    let (message_id, method) = request_identity(&list);
    assert_eq!(method, Method::SESSION_LIST);
    write_response(
        stream,
        message_id,
        serde_json::to_value(vec![session("session-1", "running")]).expect("sessions"),
    )
    .await;

    let spawn = read_frame(stream).await;
    let (message_id, method) = request_identity(&spawn);
    assert_eq!(method, Method::SESSION_SPAWN);
    write_response(
        stream,
        message_id,
        serde_json::to_value(session("session-2", "starting")).expect("spawned session"),
    )
    .await;

    let send = read_frame(stream).await;
    let (message_id, method) = request_identity(&send);
    assert_eq!(method, Method::SESSION_SEND_TEXT);
    write_response(
        stream,
        message_id,
        serde_json::to_value(AckResult { ok: true }).expect("send ack"),
    )
    .await;

    let terminal = read_frame(stream).await;
    assert_eq!(terminal.header.kind, FrameKind::StreamOpen);
    assert_ne!(terminal.header.stream_id, event_stream_id);
    let request: StreamOpenRequest =
        serde_json::from_slice(&terminal.payload).expect("terminal stream request");
    let StreamOpenRequest::Terminal(request) = request else {
        panic!("expected terminal stream");
    };
    assert_eq!(request.output_offset, 17);
    let terminal_stream_id = terminal.header.stream_id;
    write_stream_frame(
        stream,
        FrameKind::StreamOpened,
        terminal_stream_id,
        0,
        serde_json::to_vec(&serde_json::json!({})).expect("terminal opened"),
    )
    .await;
    write_stream_frame(
        stream,
        FrameKind::ReplayBegin,
        terminal_stream_id,
        1,
        0_u64.to_be_bytes().to_vec(),
    )
    .await;
    write_stream_frame(
        stream,
        FrameKind::ReplayEnd,
        terminal_stream_id,
        2,
        0_u64.to_be_bytes().to_vec(),
    )
    .await;
    write_stream_frame(
        stream,
        FrameKind::Grid,
        terminal_stream_id,
        3,
        encoded_empty_full_grid(),
    )
    .await;
    write_stream_frame(stream, FrameKind::Modes, terminal_stream_id, 4, Vec::new()).await;

    let resize = read_frame(stream).await;
    assert_eq!(resize.header.kind, FrameKind::Resize);
    assert_eq!(resize.header.stream_id, terminal_stream_id);
    assert_eq!(resize.payload, [0, 120, 0, 40]);

    let artifacts = read_frame(stream).await;
    let (message_id, method) = request_identity(&artifacts);
    assert_eq!(method, Method::SESSION_ARTIFACTS);
    write_response(
        stream,
        message_id,
        serde_json::to_value(ArtifactScan {
            artifacts: vec![SessionArtifact {
                kind: ArtifactKind::Link,
                url: "https://example.invalid".to_string(),
                label: "Link".to_string(),
            }],
            ports: vec![ListeningPort {
                port: 3000,
                url: "http://localhost:3000".to_string(),
            }],
        })
        .expect("artifacts"),
    )
    .await;

    let worktrees = read_frame(stream).await;
    let (message_id, method) = request_identity(&worktrees);
    assert_eq!(method, Method::WORKTREE_OVERVIEW);
    write_response(
        stream,
        message_id,
        serde_json::to_value(WorktreeOverviewResult {
            entries: vec![WorktreeOverviewEntry {
                project_root: "/tmp/repo".to_string(),
                path: "/tmp/repo-feature".to_string(),
                branch: Some("feature".to_string()),
                session_id: Some("session-1".to_string()),
                session_status: Some("running".to_string()),
                dirty: false,
                merged: true,
                age_days: 20,
                stale_suggestion: true,
            }],
        })
        .expect("worktrees"),
    )
    .await;
}

async fn read_hello(stream: &mut UnixStream, expected_role: ClientRole) {
    let mut preface = [0_u8; PREFACE_LEN];
    stream.read_exact(&mut preface).await.expect("preface");
    assert_eq!(
        Preface::decode(&preface).expect("decode preface"),
        Preface {
            major: WIRE_MAJOR,
            minor: WIRE_MINOR
        }
    );
    let hello = read_frame(stream).await;
    assert_eq!(hello.header.kind, FrameKind::Hello);
    let request: homie_proto::transport::HelloRequest =
        serde_json::from_slice(&hello.payload).expect("hello");
    assert_eq!(request.client_role, expected_role);
}

async fn write_hello(stream: &mut UnixStream, executable_hash: &str) {
    write_frame(
        stream,
        Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind: FrameKind::HelloAck,
                flags: 0,
                stream_id: 0,
                message_id: 0,
                sequence: 0,
            },
            payload: serde_json::to_vec(&HelloResponse {
                wire_major: WIRE_MAJOR,
                wire_minor: WIRE_MINOR,
                daemon_build: "mock".to_string(),
                daemon_version: "0.1.0".to_string(),
                daemon_pid: std::process::id(),
                daemon_instance_id: "daemon-app-test".to_string(),
                executable_hash: executable_hash.to_string(),
                method_capabilities: vec![
                    Method::STATE_SNAPSHOT.to_string(),
                    Method::SESSION_LIST.to_string(),
                    Method::SESSION_SPAWN.to_string(),
                    Method::SESSION_SEND_TEXT.to_string(),
                    Method::SESSION_RESIZE.to_string(),
                    Method::SESSION_ARTIFACTS.to_string(),
                    Method::WORKTREE_OVERVIEW.to_string(),
                ],
                stream_capabilities: vec![StreamKind::EventsV1, StreamKind::TerminalV1],
                event_oldest_seq: 0,
                event_latest_seq: 7,
            })
            .expect("hello response"),
        },
    )
    .await;
}

fn request_identity(frame: &Frame) -> (u64, String) {
    assert_eq!(frame.header.kind, FrameKind::Request);
    let request: ControlMessage = serde_json::from_slice(&frame.payload).expect("request");
    let ControlMessage::Request {
        request_id, method, ..
    } = request
    else {
        panic!("request payload");
    };
    assert_eq!(request_id.as_u64(), frame.header.message_id);
    (frame.header.message_id, method)
}

async fn write_response(stream: &mut UnixStream, message_id: u64, result: serde_json::Value) {
    write_frame(
        stream,
        Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind: FrameKind::Response,
                flags: 0,
                stream_id: 0,
                message_id,
                sequence: 0,
            },
            payload: serde_json::to_vec(&ControlMessage::success(
                RequestId::from(message_id),
                result,
            ))
            .expect("response"),
        },
    )
    .await;
}

async fn read_frame(stream: &mut UnixStream) -> Frame {
    let mut length = [0_u8; 4];
    stream.read_exact(&mut length).await.expect("frame length");
    let frame_len = u32::from_be_bytes(length) as usize;
    assert!((FRAME_HEADER_LEN..=MAX_FRAME_LEN).contains(&frame_len));
    let mut encoded = vec![0_u8; 4 + frame_len];
    encoded[..4].copy_from_slice(&length);
    stream.read_exact(&mut encoded[4..]).await.expect("frame");
    Frame::decode(&encoded, EndpointRole::Client)
        .expect("decode frame")
        .expect("complete frame")
        .0
}

async fn write_frame(stream: &mut UnixStream, frame: Frame) {
    stream
        .write_all(&frame.encode(EndpointRole::Server).expect("encode frame"))
        .await
        .expect("write frame");
}

async fn write_stream_frame(
    stream: &mut UnixStream,
    kind: FrameKind,
    stream_id: u32,
    sequence: u64,
    payload: Vec<u8>,
) {
    write_frame(
        stream,
        Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind,
                flags: 0,
                stream_id,
                message_id: 0,
                sequence,
            },
            payload,
        },
    )
    .await;
}

fn empty_full_grid() -> GridUpdate {
    GridUpdate {
        cols: 120,
        rows: 40,
        cursor_col: 0,
        cursor_row: 0,
        cursor_visible: true,
        is_full_snapshot: true,
        changed_rows: Vec::new(),
    }
}

fn encoded_empty_full_grid() -> Vec<u8> {
    let grid = empty_full_grid();
    let mut payload = Vec::new();
    payload.extend_from_slice(&grid.cols.to_be_bytes());
    payload.extend_from_slice(&grid.rows.to_be_bytes());
    payload.extend_from_slice(&grid.cursor_col.to_be_bytes());
    payload.extend_from_slice(&grid.cursor_row.to_be_bytes());
    payload.push(u8::from(grid.cursor_visible) | (u8::from(grid.is_full_snapshot) << 1));
    payload.extend_from_slice(&0_u16.to_be_bytes());
    payload
}

fn write_executable(path: &Path) {
    fs::write(path, "#!/bin/sh\nexit 0\n").expect("write executable");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("executable mode");
}

fn file_hash(path: &Path) -> String {
    let mut digest = Sha256::new();
    digest.update(fs::read(path).expect("read executable"));
    format!("{:x}", digest.finalize())
}
