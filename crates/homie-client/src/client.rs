use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use homie_proto::paths::RuntimeEndpoint;
use homie_proto::stream::{EventStreamOpen, StreamKind, TerminalStreamOpen};
use homie_proto::transport::{
    ClientRole, Frame, FrameHeader, FrameKind, HelloResponse, StableErrorCode, WIRE_MAJOR,
};
use homie_proto::{ControlMessage, ErrorEnvelope, Method, RequestId};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::sync::{oneshot, watch};
use tokio::task::JoinHandle;

use crate::connection;
use crate::events::EventStream;
use crate::streams::{StreamRegistry, stream_close_frame};
use crate::terminal::TerminalStream;
use crate::writer::{QueueError, WriterHandle};

const MAX_PENDING_REQUESTS: usize = 1024;

#[derive(Clone, Debug)]
pub struct ClientOptions {
    pub endpoint: RuntimeEndpoint,
    pub role: ClientRole,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Handshaking,
    Connected {
        daemon_instance_id: String,
        method_capabilities: Vec<String>,
        stream_capabilities: Vec<StreamKind>,
    },
    Degraded {
        code: String,
    },
    Reconnecting {
        attempt: u32,
        delay: Duration,
    },
    Shutdown,
}

#[derive(Clone)]
pub struct HomieClient {
    pub(crate) inner: Arc<ClientInner>,
    _lifetime: Arc<ClientLifetime>,
}

impl HomieClient {
    pub async fn connect(options: ClientOptions) -> Result<Self, ClientError> {
        if options.connect_timeout.is_zero() {
            return Err(ClientError::BadRequest(
                "connect timeout must be greater than zero".to_string(),
            ));
        }
        if options.request_timeout.is_zero() {
            return Err(ClientError::BadRequest(
                "request timeout must be greater than zero".to_string(),
            ));
        }

        let (state_tx, state_rx) = watch::channel(ConnectionState::Disconnected);
        let (shutdown_tx, _) = watch::channel(false);
        let inner = Arc::new(ClientInner {
            options: options.clone(),
            state_tx,
            state_rx,
            shutdown_tx: shutdown_tx.clone(),
            writer: RwLock::new(None),
            hello: RwLock::new(None),
            pending: PendingRequests::default(),
            streams: StreamRegistry::default(),
            manager: Mutex::new(None),
        });
        let (initial_tx, initial_rx) = oneshot::channel();
        let manager_inner = inner.clone();
        let manager = tokio::spawn(async move {
            connection::run(manager_inner, initial_tx).await;
        });
        *inner.manager.lock().expect("client manager lock poisoned") = Some(manager);

        tokio::task::yield_now().await;
        let initial = tokio::time::timeout(options.connect_timeout, initial_rx).await;
        match initial {
            Ok(Ok(Ok(()))) => Ok(Self {
                inner,
                _lifetime: Arc::new(ClientLifetime { shutdown_tx }),
            }),
            Ok(Ok(Err(error))) => {
                stop_failed_connect(&inner).await;
                Err(error)
            }
            Ok(Err(_)) => {
                stop_failed_connect(&inner).await;
                Err(ClientError::Unavailable)
            }
            Err(_) => {
                stop_failed_connect(&inner).await;
                Err(ClientError::Timeout)
            }
        }
    }

    #[must_use]
    pub fn connection_state(&self) -> watch::Receiver<ConnectionState> {
        self.inner.state_rx.clone()
    }

    #[must_use]
    pub fn hello(&self) -> Option<HelloResponse> {
        self.inner
            .hello
            .read()
            .expect("client hello lock poisoned")
            .clone()
    }

    pub async fn request<P, R>(&self, method: &str, params: P) -> Result<R, ClientError>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let value = self
            .request_value(
                method,
                serde_json::to_value(params)?,
                self.inner.options.request_timeout,
            )
            .await?;
        Ok(serde_json::from_value(value)?)
    }

    pub async fn subscribe_events(
        &self,
        request: EventStreamOpen,
    ) -> Result<EventStream, ClientError> {
        self.inner.ensure_stream_capability(StreamKind::EventsV1)?;
        let (stream, opened, frame) = self.inner.streams.insert_event(&self.inner, request)?;
        let stream_id = stream.stream_id;
        let writer = self.inner.writer().ok_or(ClientError::Unavailable)?;
        if let Err(error) = writer.try_send_high(frame) {
            self.inner.streams.remove(stream_id);
            return Err(error.into());
        }
        match tokio::time::timeout(self.inner.options.request_timeout, opened).await {
            Ok(Ok(Ok(()))) => Ok(stream),
            Ok(Ok(Err(error))) => Err(error),
            Ok(Err(_)) => Err(ClientError::Unavailable),
            Err(_) => {
                self.inner.streams.remove(stream_id);
                Err(ClientError::Timeout)
            }
        }
    }

    pub async fn open_terminal(
        &self,
        request: TerminalStreamOpen,
    ) -> Result<TerminalStream, ClientError> {
        self.inner
            .ensure_stream_capability(StreamKind::TerminalV1)?;
        let (stream, opened, frame) = self.inner.streams.insert_terminal(&self.inner, request)?;
        let stream_id = stream.stream_id;
        let writer = self.inner.writer().ok_or(ClientError::Unavailable)?;
        if let Err(error) = writer.try_send_high(frame) {
            self.inner.streams.remove(stream_id);
            return Err(error.into());
        }
        match tokio::time::timeout(self.inner.options.request_timeout, opened).await {
            Ok(Ok(Ok(()))) => Ok(stream),
            Ok(Ok(Err(error))) => Err(error),
            Ok(Err(_)) => Err(ClientError::Unavailable),
            Err(_) => {
                self.inner.streams.remove(stream_id);
                Err(ClientError::Timeout)
            }
        }
    }

    pub async fn spawn_shell(
        &self,
        cwd: &Path,
        title: Option<&str>,
    ) -> Result<homie_proto::model::SessionSummary, ClientError> {
        self.spawn_shell_with_parent(cwd, title, None).await
    }

    pub async fn spawn_shell_with_parent(
        &self,
        cwd: &Path,
        title: Option<&str>,
        parent_session_id: Option<&str>,
    ) -> Result<homie_proto::model::SessionSummary, ClientError> {
        self.request(
            Method::SESSION_SPAWN,
            homie_proto::SessionSpawnRequest {
                cwd: cwd.display().to_string(),
                title: title.map(str::to_string),
                parent_session_id: parent_session_id.map(homie_proto::SessionId::from),
            },
        )
        .await
    }

    pub async fn list_sessions(
        &self,
    ) -> Result<Vec<homie_proto::model::SessionSummary>, ClientError> {
        self.request(Method::SESSION_LIST, serde_json::json!({}))
            .await
    }

    pub async fn session_snapshot(
        &self,
        session_id: &str,
        output_offset: u64,
        max_bytes: usize,
    ) -> Result<homie_proto::model::SessionSnapshot, ClientError> {
        self.typed_request_with_timeout(
            Method::SESSION_SNAPSHOT,
            homie_proto::model::SessionSnapshotRequest {
                session_id: session_id.to_string(),
                output_offset,
                max_bytes,
            },
            Duration::from_secs(15),
        )
        .await
    }

    pub async fn read_output(&self, session_id: &str) -> Result<String, ClientError> {
        Ok(self
            .session_snapshot(session_id, 0, 4 * 1024 * 1024)
            .await?
            .output_text)
    }

    pub async fn status_report(
        &self,
        session_id: &str,
    ) -> Result<homie_proto::model::SessionStatusReport, ClientError> {
        self.typed_request_with_timeout(
            Method::SESSION_STATUS,
            homie_proto::model::SessionStatusRequest {
                session_id: session_id.to_string(),
            },
            Duration::from_secs(15),
        )
        .await
    }

    pub async fn scan_session_artifacts(
        &self,
        session_id: &str,
    ) -> Result<homie_proto::model::ArtifactScan, ClientError> {
        self.typed_request_with_timeout(
            Method::SESSION_ARTIFACTS,
            homie_proto::model::SessionArtifactsRequest {
                session_id: session_id.to_string(),
            },
            Duration::from_secs(15),
        )
        .await
    }

    pub async fn list_ports(&self) -> Result<Vec<homie_proto::model::PortListRow>, ClientError> {
        self.typed_request_with_timeout(
            Method::SESSION_PORTS,
            homie_proto::model::SessionPortsRequest { session_id: None },
            Duration::from_secs(15),
        )
        .await
    }

    pub async fn send_text(
        &self,
        session_id: &str,
        text: &str,
        submit: bool,
    ) -> Result<(), ClientError> {
        let _: homie_proto::transport::AckResult = self
            .request(
                Method::SESSION_SEND_TEXT,
                homie_proto::SessionSendTextRequest {
                    session_id: session_id.into(),
                    text: text.to_string(),
                    submit,
                },
            )
            .await?;
        Ok(())
    }

    pub async fn resize_session(
        &self,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), ClientError> {
        let _: homie_proto::transport::AckResult = self
            .request(
                Method::SESSION_RESIZE,
                homie_proto::SessionResizeRequest {
                    session_id: session_id.into(),
                    cols,
                    rows,
                },
            )
            .await?;
        Ok(())
    }

    pub async fn terminate_session(&self, session_id: &str) -> Result<(), ClientError> {
        let _: homie_proto::transport::AckResult = self
            .request(
                Method::SESSION_KILL,
                homie_proto::SessionKillRequest {
                    session_id: session_id.into(),
                },
            )
            .await?;
        Ok(())
    }

    pub async fn set_session_parent(
        &self,
        session_id: &str,
        parent_session_id: &str,
    ) -> Result<(), ClientError> {
        let _: homie_proto::transport::AckResult = self
            .request(
                Method::SESSION_SET_PARENT,
                homie_proto::model::SessionSetParentRequest {
                    session_id: session_id.to_string(),
                    parent_session_id: parent_session_id.to_string(),
                },
            )
            .await?;
        Ok(())
    }

    pub async fn list_child_sessions(
        &self,
        parent_session_id: &str,
    ) -> Result<Vec<homie_proto::model::SessionSummary>, ClientError> {
        self.request(
            Method::SESSION_LIST_CHILDREN,
            homie_proto::model::SessionChildrenRequest {
                parent_session_id: parent_session_id.to_string(),
            },
        )
        .await
    }

    pub async fn parent_session_id(&self, session_id: &str) -> Result<Option<String>, ClientError> {
        let result: homie_proto::model::SessionParentResult = self
            .request(
                Method::SESSION_PARENT,
                homie_proto::model::SessionParentRequest {
                    session_id: session_id.to_string(),
                },
            )
            .await?;
        Ok(result.parent_session_id)
    }

    pub async fn session_history(
        &self,
        request: homie_proto::SessionHistoryRequest,
    ) -> Result<Vec<homie_proto::model::ScannedHistoryEntry>, ClientError> {
        self.typed_request_with_timeout(Method::SESSION_HISTORY, request, Duration::from_secs(35))
            .await
    }

    pub async fn resume_from_history(
        &self,
        request: homie_proto::SessionResumeFromHistoryRequest,
    ) -> Result<homie_proto::model::SessionSummary, ClientError> {
        self.request(Method::SESSION_RESUME_FROM_HISTORY, request)
            .await
    }

    pub async fn read_diff(
        &self,
        session_id: &str,
        base: homie_proto::SessionDiffBase,
    ) -> Result<homie_proto::SessionReadDiffResult, ClientError> {
        self.typed_request_with_timeout(
            Method::SESSION_READ_DIFF,
            homie_proto::SessionReadDiffRequest {
                session_id: session_id.into(),
                base: Some(base),
            },
            Duration::from_secs(20),
        )
        .await
    }

    pub async fn locate_repo(
        &self,
        request: homie_proto::HostLocateRepoParams,
    ) -> Result<homie_proto::HostLocateRepoResult, ClientError> {
        self.typed_request_with_timeout(Method::HOST_LOCATE_REPO, request, Duration::from_secs(20))
            .await
    }

    pub async fn worktree_list(
        &self,
        request: homie_proto::WorktreeListRequest,
    ) -> Result<Vec<homie_proto::WorktreeInfo>, ClientError> {
        self.typed_request_with_timeout(Method::WORKTREE_LIST, request, Duration::from_secs(20))
            .await
    }

    pub async fn worktree_create(
        &self,
        request: homie_proto::WorktreeCreateRequest,
    ) -> Result<homie_proto::WorktreeInfo, ClientError> {
        self.typed_request_with_timeout(Method::WORKTREE_CREATE, request, Duration::from_secs(65))
            .await
    }

    pub async fn worktree_remove(
        &self,
        request: homie_proto::WorktreeRemoveRequest,
    ) -> Result<(), ClientError> {
        let _: homie_proto::transport::AckResult = self
            .typed_request_with_timeout(Method::WORKTREE_REMOVE, request, Duration::from_secs(65))
            .await?;
        Ok(())
    }

    pub async fn worktree_overview(
        &self,
    ) -> Result<homie_proto::model::WorktreeOverviewResult, ClientError> {
        self.typed_request_with_timeout(
            Method::WORKTREE_OVERVIEW,
            serde_json::json!({}),
            Duration::from_secs(20),
        )
        .await
    }

    pub async fn report_needs_input(
        &self,
        session_id: &str,
        detail: &homie_proto::NeedsInputDetail,
    ) -> Result<(), ClientError> {
        self.report_hook(session_id, "needs_input", Some(detail.clone()), false)
            .await
    }

    pub async fn report_turn_complete(&self, session_id: &str) -> Result<(), ClientError> {
        self.report_hook(session_id, "turn_completed", None, true)
            .await
    }

    async fn report_hook(
        &self,
        session_id: &str,
        event: &str,
        needs_input: Option<homie_proto::NeedsInputDetail>,
        turn_completed: bool,
    ) -> Result<(), ClientError> {
        let _: homie_proto::transport::AckResult = self
            .request(
                Method::HOOK_REPORT,
                homie_proto::model::HookReportRequest {
                    session_id: session_id.to_string(),
                    event: event.to_string(),
                    needs_input,
                    turn_completed,
                },
            )
            .await?;
        Ok(())
    }

    async fn typed_request_with_timeout<P, R>(
        &self,
        method: &str,
        params: P,
        timeout: Duration,
    ) -> Result<R, ClientError>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let value = self
            .request_value(method, serde_json::to_value(params)?, timeout)
            .await?;
        Ok(serde_json::from_value(value)?)
    }

    pub async fn close(&self) -> Result<(), ClientError> {
        let close_result = self.inner.close_all_streams();
        tokio::task::yield_now().await;
        let _ = self.inner.shutdown_tx.send(true);
        self.inner.set_state(ConnectionState::Shutdown);
        self.inner.clear_writer();
        self.inner.pending.fail_all_unavailable();
        let manager = self
            .inner
            .manager
            .lock()
            .expect("client manager lock poisoned")
            .take();
        if let Some(manager) = manager {
            manager.await.map_err(|_| ClientError::Internal)?;
        }
        close_result?;
        Ok(())
    }

    pub(crate) async fn request_value(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, ClientError> {
        self.inner.request_value(method, params, timeout).await
    }
}

impl ClientInner {
    pub(crate) async fn request_value(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, ClientError> {
        self.ensure_capability(method)?;
        let (message_id, receiver) = self.pending.insert()?;
        let mut guard = PendingGuard {
            message_id,
            pending: &self.pending,
            armed: true,
        };
        let payload = ControlMessage::request(RequestId::from(message_id), method, params);
        let frame = Frame {
            header: FrameHeader {
                version: WIRE_MAJOR,
                kind: FrameKind::Request,
                flags: 0,
                stream_id: 0,
                message_id,
                sequence: 0,
            },
            payload: serde_json::to_vec(&payload)?,
        };
        let writer = self.writer().ok_or(ClientError::Unavailable)?;
        if let Err(error) = writer.try_send_high(frame) {
            self.pending.remove(message_id);
            guard.disarm();
            return Err(error.into());
        }

        let response = tokio::time::timeout(timeout, receiver).await;
        match response {
            Ok(Ok(result)) => {
                guard.disarm();
                result
            }
            Ok(Err(_)) => {
                guard.disarm();
                Err(ClientError::Unavailable)
            }
            Err(_) => Err(ClientError::Timeout),
        }
    }
}

pub(crate) struct ClientInner {
    pub(crate) options: ClientOptions,
    pub(crate) state_tx: watch::Sender<ConnectionState>,
    state_rx: watch::Receiver<ConnectionState>,
    pub(crate) shutdown_tx: watch::Sender<bool>,
    writer: RwLock<Option<WriterHandle>>,
    hello: RwLock<Option<HelloResponse>>,
    pub(crate) pending: PendingRequests,
    pub(crate) streams: StreamRegistry,
    manager: Mutex<Option<JoinHandle<()>>>,
}

impl ClientInner {
    pub(crate) fn set_state(&self, state: ConnectionState) {
        self.state_tx.send_replace(state);
    }

    pub(crate) fn set_connected(&self, hello: HelloResponse, writer: WriterHandle) {
        *self.writer.write().expect("client writer lock poisoned") = Some(writer);
        *self.hello.write().expect("client hello lock poisoned") = Some(hello.clone());
        self.set_state(ConnectionState::Connected {
            daemon_instance_id: hello.daemon_instance_id,
            method_capabilities: hello.method_capabilities,
            stream_capabilities: hello.stream_capabilities,
        });
    }

    pub(crate) fn clear_writer(&self) {
        self.writer
            .write()
            .expect("client writer lock poisoned")
            .take();
    }

    pub(crate) fn writer(&self) -> Option<WriterHandle> {
        self.writer
            .read()
            .expect("client writer lock poisoned")
            .clone()
    }

    fn ensure_capability(&self, method: &str) -> Result<(), ClientError> {
        let hello = self.hello.read().expect("client hello lock poisoned");
        let Some(hello) = hello.as_ref() else {
            return Err(ClientError::Unavailable);
        };
        if hello
            .method_capabilities
            .iter()
            .any(|capability| capability == method)
        {
            return Ok(());
        }
        Err(ClientError::Remote(Box::new(ErrorEnvelope::new(
            StableErrorCode::MethodNotFound.as_str(),
            "method is not available from the connected daemon",
            false,
        ))))
    }

    fn ensure_stream_capability(&self, kind: StreamKind) -> Result<(), ClientError> {
        let hello = self.hello.read().expect("client hello lock poisoned");
        let Some(hello) = hello.as_ref() else {
            return Err(ClientError::Unavailable);
        };
        if hello.stream_capabilities.contains(&kind) {
            return Ok(());
        }
        Err(ClientError::Remote(Box::new(ErrorEnvelope::new(
            StableErrorCode::MethodNotFound.as_str(),
            "stream is not available from the connected daemon",
            false,
        ))))
    }

    pub(crate) fn close_stream(&self, stream_id: u32) {
        if self.streams.remove(stream_id)
            && let Some(writer) = self.writer()
        {
            writer.close_stream(stream_id);
            let _ = writer.try_send_high(stream_close_frame(stream_id));
        }
    }

    fn close_all_streams(&self) -> Result<(), ClientError> {
        let stream_ids = self.streams.close_all();
        let Some(writer) = self.writer() else {
            return Ok(());
        };
        let mut result = Ok(());
        for stream_id in stream_ids {
            writer.close_stream(stream_id);
            if let Err(error) = writer.try_send_high(stream_close_frame(stream_id))
                && result.is_ok()
            {
                result = Err(error.into());
            }
        }
        result
    }

    pub(crate) async fn recover_event(self: Arc<Self>, stream_id: u32) {
        let Some(close_remote) = self.streams.take_event_recovery_close(stream_id) else {
            return;
        };
        if close_remote {
            let Some(writer) = self.writer() else {
                return;
            };
            writer.close_stream(stream_id);
            if writer.try_send_high(stream_close_frame(stream_id)).is_err() {
                return;
            }
        }
        let mut retry_delay = Duration::from_millis(50);
        loop {
            if *self.shutdown_tx.borrow() || !self.streams.event_recovery_active(stream_id) {
                return;
            }
            let snapshot: Result<homie_proto::model::StateSnapshot, ClientError> = self
                .request_value(
                    Method::STATE_SNAPSHOT,
                    serde_json::json!({}),
                    self.options.request_timeout,
                )
                .await
                .and_then(|value| serde_json::from_value(value).map_err(ClientError::from));
            match snapshot {
                Ok(snapshot) => {
                    let Ok(frame) = self
                        .streams
                        .complete_event_recovery(stream_id, snapshot)
                        .await
                    else {
                        self.streams.remove(stream_id);
                        return;
                    };
                    if let Some(writer) = self.writer() {
                        let _ = writer.try_send_high(frame);
                    }
                    return;
                }
                Err(error) if event_recovery_is_retryable(&error) => {
                    tokio::time::sleep(retry_delay).await;
                    retry_delay = (retry_delay * 2).min(Duration::from_secs(1));
                }
                Err(_) => {
                    self.streams.remove(stream_id);
                    return;
                }
            }
        }
    }
}

fn event_recovery_is_retryable(error: &ClientError) -> bool {
    matches!(
        error,
        ClientError::Timeout | ClientError::Unavailable | ClientError::Backpressure
    ) || matches!(error, ClientError::Remote(error) if error.retryable)
}

struct ClientLifetime {
    shutdown_tx: watch::Sender<bool>,
}

impl Drop for ClientLifetime {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(true);
    }
}

#[derive(Default)]
pub(crate) struct PendingRequests {
    next_id: AtomicU64,
    waiters: Mutex<HashMap<u64, oneshot::Sender<Result<Value, ClientError>>>>,
}

impl PendingRequests {
    fn insert(
        &self,
    ) -> Result<
        (
            u64,
            oneshot::Receiver<Result<serde_json::Value, ClientError>>,
        ),
        ClientError,
    > {
        let mut waiters = self.waiters.lock().expect("pending request lock poisoned");
        if waiters.len() >= MAX_PENDING_REQUESTS {
            return Err(ClientError::Backpressure);
        }
        let mut message_id = self.next_id.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
        if message_id == 0 {
            message_id = self.next_id.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
        }
        let (sender, receiver) = oneshot::channel();
        waiters.insert(message_id, sender);
        Ok((message_id, receiver))
    }

    pub(crate) fn resolve(&self, message_id: u64, result: Result<Value, ClientError>) {
        if let Some(waiter) = self
            .waiters
            .lock()
            .expect("pending request lock poisoned")
            .remove(&message_id)
        {
            let _ = waiter.send(result);
        }
    }

    fn remove(&self, message_id: u64) {
        self.waiters
            .lock()
            .expect("pending request lock poisoned")
            .remove(&message_id);
    }

    pub(crate) fn fail_all_unavailable(&self) {
        let waiters =
            std::mem::take(&mut *self.waiters.lock().expect("pending request lock poisoned"));
        for (_, waiter) in waiters {
            let _ = waiter.send(Err(ClientError::Unavailable));
        }
    }
}

struct PendingGuard<'a> {
    message_id: u64,
    pending: &'a PendingRequests,
    armed: bool,
}

impl PendingGuard<'_> {
    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for PendingGuard<'_> {
    fn drop(&mut self) {
        if self.armed {
            self.pending.remove(self.message_id);
        }
    }
}

async fn stop_failed_connect(inner: &Arc<ClientInner>) {
    let _ = inner.shutdown_tx.send(true);
    let manager = inner
        .manager
        .lock()
        .expect("client manager lock poisoned")
        .take();
    if let Some(manager) = manager {
        let _ = manager.await;
    }
}

impl From<QueueError> for ClientError {
    fn from(error: QueueError) -> Self {
        match error {
            QueueError::Backpressure => Self::Backpressure,
            QueueError::Closed => Self::Unavailable,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("bad client request: {0}")]
    BadRequest(String),
    #[error("runtime method failed")]
    Remote(Box<ErrorEnvelope>),
    #[error("runtime request timed out")]
    Timeout,
    #[error("runtime endpoint is unavailable")]
    Unavailable,
    #[error("client transport is under backpressure")]
    Backpressure,
    #[error("runtime protocol version mismatch")]
    VersionMismatch,
    #[error("runtime endpoint rejected this client")]
    Unauthorized,
    #[error("runtime stream requires resynchronization")]
    ResyncRequired,
    #[error("runtime protocol error: {0}")]
    Protocol(String),
    #[error("client internal task failed")]
    Internal,
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl ClientError {
    #[must_use]
    pub fn code(&self) -> &str {
        match self {
            Self::BadRequest(_) => StableErrorCode::BadRequest.as_str(),
            Self::Remote(error) => &error.code,
            Self::Timeout => StableErrorCode::Timeout.as_str(),
            Self::Unavailable => StableErrorCode::Unavailable.as_str(),
            Self::Backpressure => StableErrorCode::Backpressure.as_str(),
            Self::VersionMismatch => StableErrorCode::VersionMismatch.as_str(),
            Self::Unauthorized => StableErrorCode::Unauthorized.as_str(),
            Self::ResyncRequired => StableErrorCode::ResyncRequired.as_str(),
            Self::Protocol(_) | Self::Internal | Self::Json(_) => {
                StableErrorCode::Internal.as_str()
            }
        }
    }
}
