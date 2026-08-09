use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};

use homie_proto::model::{
    HolderSnapshot as ProtoHolderSnapshot, SessionParentResult,
    SessionSummary as ProtoSessionSummary, StateSnapshot,
};
use homie_storage::SessionSummary;
use thiserror::Error;
use tokio::sync::oneshot;

use crate::RuntimeSupervisor;
use crate::dispatcher::{
    ActorRequest, LongRunningRequest, LongRunningResult, PreparedLongRunning, PreparedPortSession,
    RuntimeResponse,
};
use crate::terminal_stream::TerminalSourceDescriptor;

pub const ACTOR_QUEUE_CAPACITY: usize = 256;

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ServiceError {
    #[error("runtime actor queue is full")]
    Backpressure,
    #[error("runtime actor is unavailable")]
    Unavailable,
    #[error("runtime operation timed out")]
    Timeout,
    #[error("runtime operation was cancelled")]
    Cancelled,
    #[error("invalid runtime request: {0}")]
    BadRequest(String),
    #[error("runtime method not found: {0}")]
    MethodNotFound(String),
    #[error("runtime operation failed")]
    Internal,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeCall {
    Invoke(ActorRequest),
    PrepareLongRunning(LongRunningRequest),
    CommitLongRunning(Box<LongRunningResult>),
    TerminalDescribe {
        session_id: String,
    },
    TerminalInput {
        session_id: String,
        bytes: Vec<u8>,
    },
    TerminalResize {
        session_id: String,
        cols: u16,
        rows: u16,
    },
}

fn allow_during_drain(request: &RuntimeCall) -> bool {
    matches!(
        request,
        RuntimeCall::Invoke(ActorRequest::Shutdown) | RuntimeCall::CommitLongRunning(_)
    )
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeReply {
    Response(RuntimeResponse),
    Prepared(PreparedLongRunning),
    TerminalDescriptor(TerminalSourceDescriptor),
}

impl RuntimeReply {
    pub(crate) fn into_response(self) -> ServiceResult<RuntimeResponse> {
        match self {
            Self::Response(response) => Ok(response),
            Self::Prepared(_) | Self::TerminalDescriptor(_) => Err(ServiceError::Internal),
        }
    }

    pub(crate) fn into_prepared(self) -> ServiceResult<PreparedLongRunning> {
        match self {
            Self::Prepared(prepared) => Ok(prepared),
            Self::Response(_) | Self::TerminalDescriptor(_) => Err(ServiceError::Internal),
        }
    }

    pub(crate) fn into_terminal_descriptor(self) -> ServiceResult<TerminalSourceDescriptor> {
        match self {
            Self::TerminalDescriptor(descriptor) => Ok(descriptor),
            Self::Response(_) | Self::Prepared(_) => Err(ServiceError::Internal),
        }
    }
}

pub trait RuntimeBackend: Send + 'static {
    fn call(&mut self, request: RuntimeCall) -> ServiceResult<RuntimeReply>;

    fn prepare_shutdown(&mut self) -> ServiceResult<()> {
        Ok(())
    }

    fn shutdown(&mut self) {}
}

pub struct RuntimeSupervisorBackend {
    supervisor: RuntimeSupervisor,
}

impl RuntimeSupervisorBackend {
    #[must_use]
    pub fn new(supervisor: RuntimeSupervisor) -> Self {
        Self { supervisor }
    }
}

impl RuntimeBackend for RuntimeSupervisorBackend {
    fn call(&mut self, request: RuntimeCall) -> ServiceResult<RuntimeReply> {
        match request {
            RuntimeCall::Invoke(ActorRequest::StateSnapshot) => {
                let sessions = self
                    .supervisor
                    .list_sessions()
                    .map_err(|_| ServiceError::Internal)?
                    .into_iter()
                    .map(session_to_proto)
                    .collect();
                let event_cursor = self
                    .supervisor
                    .events_after(0, &[])
                    .last()
                    .map_or(0, |event| event.seq);
                Ok(RuntimeReply::Response(RuntimeResponse::StateSnapshot(
                    StateSnapshot {
                        sessions,
                        event_cursor,
                    },
                )))
            }
            RuntimeCall::Invoke(ActorRequest::PrepareShutdown) => {
                self.prepare_shutdown()?;
                Ok(ack())
            }
            RuntimeCall::Invoke(ActorRequest::Shutdown) => {
                self.prepare_shutdown()?;
                Ok(RuntimeReply::Response(RuntimeResponse::Shutdown(
                    homie_proto::transport::ShutdownResult { acknowledged: true },
                )))
            }
            RuntimeCall::Invoke(ActorRequest::SessionSpawn(request)) => {
                let session = self
                    .supervisor
                    .spawn_shell_with_parent(
                        PathBuf::from(request.cwd).as_path(),
                        request.title.as_deref(),
                        request
                            .parent_session_id
                            .as_ref()
                            .map(homie_proto::SessionId::as_str),
                    )
                    .map_err(|_| ServiceError::Internal)?;
                Ok(RuntimeReply::Response(RuntimeResponse::Session(
                    session_to_proto(session),
                )))
            }
            RuntimeCall::Invoke(ActorRequest::SessionList) => {
                let sessions = self
                    .supervisor
                    .list_sessions()
                    .map_err(|_| ServiceError::Internal)?
                    .into_iter()
                    .map(session_to_proto)
                    .collect::<Vec<_>>();
                Ok(RuntimeReply::Response(RuntimeResponse::Sessions(sessions)))
            }
            RuntimeCall::Invoke(ActorRequest::SessionSetParent(request)) => {
                self.supervisor
                    .storage()
                    .set_session_parent(&request.session_id, &request.parent_session_id)
                    .map_err(|_| ServiceError::Internal)?;
                Ok(ack())
            }
            RuntimeCall::Invoke(ActorRequest::SessionListChildren(request)) => {
                let sessions = self
                    .supervisor
                    .storage()
                    .list_child_sessions(&request.parent_session_id)
                    .map_err(|_| ServiceError::Internal)?
                    .into_iter()
                    .map(session_to_proto)
                    .collect();
                Ok(RuntimeReply::Response(RuntimeResponse::Sessions(sessions)))
            }
            RuntimeCall::Invoke(ActorRequest::SessionParent(request)) => {
                let parent_session_id = self
                    .supervisor
                    .storage()
                    .session_core_metadata(&request.session_id)
                    .map_err(|_| ServiceError::Internal)?
                    .parent_session_id;
                Ok(RuntimeReply::Response(RuntimeResponse::SessionParent(
                    SessionParentResult { parent_session_id },
                )))
            }
            RuntimeCall::Invoke(ActorRequest::SessionResumeFromHistory(request)) => {
                let entry = crate::history::ScannedHistoryEntry {
                    agent_kind: request.agent_kind.clone(),
                    external_id: request.external_id.clone(),
                    cwd: PathBuf::from(&request.cwd),
                    title: request.title.clone(),
                    title_source: "history".to_string(),
                    transcript_path: PathBuf::new(),
                    last_active_at: 0,
                    created_at: None,
                    cwd_exists: PathBuf::from(&request.cwd).is_dir(),
                };
                let command = crate::history::resume_command(&entry).ok_or_else(|| {
                    ServiceError::BadRequest("history entry cannot be resumed".to_string())
                })?;
                let session = self
                    .supervisor
                    .spawn_shell(&entry.cwd, entry.title.as_deref())
                    .map_err(|_| ServiceError::Internal)?;
                if self
                    .supervisor
                    .send_text(&session.id, &command, true)
                    .is_err()
                {
                    let _ = self.supervisor.terminate_session(&session.id);
                    return Err(ServiceError::Internal);
                }
                if self
                    .supervisor
                    .storage()
                    .mark_history_entry_tracked(
                        &request.agent_kind,
                        &request.external_id,
                        &session.id,
                    )
                    .is_err()
                {
                    let _ = self.supervisor.terminate_session(&session.id);
                    return Err(ServiceError::Internal);
                }
                Ok(RuntimeReply::Response(RuntimeResponse::Session(
                    session_to_proto(session),
                )))
            }
            RuntimeCall::Invoke(ActorRequest::SessionSendText(request)) => {
                self.supervisor
                    .send_text(request.session_id.as_str(), &request.text, request.submit)
                    .map_err(|_| ServiceError::Internal)?;
                Ok(ack())
            }
            RuntimeCall::Invoke(ActorRequest::SessionResize(request)) => {
                self.supervisor
                    .resize_session(request.session_id.as_str(), request.cols, request.rows)
                    .map_err(|_| ServiceError::Internal)?;
                Ok(ack())
            }
            RuntimeCall::Invoke(ActorRequest::SessionKill(request)) => {
                self.supervisor
                    .terminate_session(request.session_id.as_str())
                    .map_err(|_| ServiceError::Internal)?;
                Ok(ack())
            }
            RuntimeCall::Invoke(ActorRequest::HookReport(request)) => {
                let mut handled = false;
                if let Some(detail) = request.needs_input.as_ref() {
                    self.supervisor
                        .report_needs_input(&request.session_id, detail)
                        .map_err(|_| ServiceError::Internal)?;
                    handled = true;
                }
                if request.turn_completed {
                    self.supervisor
                        .report_turn_complete(&request.session_id)
                        .map_err(|_| ServiceError::Internal)?;
                    handled = true;
                }
                if !handled {
                    return Err(ServiceError::BadRequest(format!(
                        "unsupported hook event: {}",
                        request.event
                    )));
                }
                Ok(ack())
            }
            RuntimeCall::PrepareLongRunning(LongRunningRequest::SessionSnapshot(request)) => {
                let session = find_session(&self.supervisor, &request.session_id)?;
                let output_path = self.supervisor.output_log_path(&request.session_id);
                let status = self
                    .supervisor
                    .prepare_session_status(&request.session_id)
                    .map_err(|_| ServiceError::Internal)?;
                let holder = self
                    .supervisor
                    .holder_snapshot(&request.session_id)
                    .map(holder_to_proto);
                Ok(RuntimeReply::Prepared(
                    PreparedLongRunning::SessionSnapshot {
                        request,
                        session: Box::new(session_to_proto(session)),
                        output_path,
                        status,
                        holder,
                    },
                ))
            }
            RuntimeCall::PrepareLongRunning(LongRunningRequest::SessionStatus(request)) => {
                find_session(&self.supervisor, &request.session_id)?;
                Ok(RuntimeReply::Prepared(PreparedLongRunning::SessionStatus {
                    output_path: self.supervisor.output_log_path(&request.session_id),
                    status: self
                        .supervisor
                        .prepare_session_status(&request.session_id)
                        .map_err(|_| ServiceError::Internal)?,
                }))
            }
            RuntimeCall::PrepareLongRunning(LongRunningRequest::SessionArtifacts(request)) => {
                find_session(&self.supervisor, &request.session_id)?;
                Ok(RuntimeReply::Prepared(
                    PreparedLongRunning::SessionArtifacts {
                        output_path: self.supervisor.output_log_path(&request.session_id),
                    },
                ))
            }
            RuntimeCall::PrepareLongRunning(LongRunningRequest::SessionPorts(request)) => {
                let sessions = self
                    .supervisor
                    .list_sessions()
                    .map_err(|_| ServiceError::Internal)?
                    .into_iter()
                    .filter(|session| {
                        request
                            .session_id
                            .as_deref()
                            .is_none_or(|session_id| session.id == session_id)
                    })
                    .map(|session| PreparedPortSession {
                        output_path: self.supervisor.output_log_path(&session.id),
                        session_id: session.id,
                        session_title: session.title,
                    })
                    .collect();
                Ok(RuntimeReply::Prepared(PreparedLongRunning::SessionPorts {
                    sessions,
                }))
            }
            RuntimeCall::PrepareLongRunning(LongRunningRequest::HostLocateRepo(request)) => {
                let mut candidates = Vec::new();
                if let Some(session_id) = request.session_id.as_ref() {
                    candidates.push(PathBuf::from(
                        find_session(&self.supervisor, session_id.as_str())?.workspace,
                    ));
                }
                candidates.extend(
                    self.supervisor
                        .storage()
                        .list_projects()
                        .map_err(|_| ServiceError::Internal)?
                        .into_iter()
                        .map(|project| PathBuf::from(project.root_path)),
                );
                candidates.sort();
                candidates.dedup();
                Ok(RuntimeReply::Prepared(
                    PreparedLongRunning::HostLocateRepo {
                        request,
                        candidates,
                    },
                ))
            }
            RuntimeCall::PrepareLongRunning(LongRunningRequest::WorktreeOverview) => {
                let projects = self
                    .supervisor
                    .storage()
                    .list_projects()
                    .map_err(|_| ServiceError::Internal)?
                    .into_iter()
                    .map(|project| PathBuf::from(project.root_path))
                    .collect();
                let sessions = self
                    .supervisor
                    .list_sessions()
                    .map_err(|_| ServiceError::Internal)?
                    .into_iter()
                    .map(session_to_proto)
                    .collect();
                Ok(RuntimeReply::Prepared(
                    PreparedLongRunning::WorktreeOverview { projects, sessions },
                ))
            }
            RuntimeCall::PrepareLongRunning(LongRunningRequest::SessionReadDiff(request)) => {
                let cwd = self
                    .supervisor
                    .list_sessions()
                    .map_err(|_| ServiceError::Internal)?
                    .into_iter()
                    .find(|session| session.id == request.session_id.as_str())
                    .map(|session| PathBuf::from(session.workspace))
                    .ok_or_else(|| {
                        ServiceError::BadRequest(format!("unknown session: {}", request.session_id))
                    })?;
                Ok(RuntimeReply::Prepared(
                    PreparedLongRunning::SessionReadDiff {
                        cwd,
                        comparison: request.base.unwrap_or_default(),
                    },
                ))
            }
            RuntimeCall::PrepareLongRunning(request) => Ok(RuntimeReply::Prepared(
                PreparedLongRunning::Request(request),
            )),
            RuntimeCall::CommitLongRunning(result) => match *result {
                LongRunningResult::Response(response) => Ok(RuntimeReply::Response(*response)),
                LongRunningResult::HistoryScan(entries) => {
                    let scanned = entries
                        .iter()
                        .map(|entry| crate::history::ScannedHistoryEntry {
                            agent_kind: entry.agent_kind.clone(),
                            external_id: entry.external_id.clone(),
                            cwd: PathBuf::from(&entry.cwd),
                            title: entry.title.clone(),
                            title_source: entry.title_source.clone(),
                            transcript_path: PathBuf::from(&entry.transcript_path),
                            last_active_at: entry.last_active_at,
                            created_at: entry.created_at,
                            cwd_exists: entry.cwd_exists,
                        })
                        .collect::<Vec<_>>();
                    crate::history::write_history_to_storage(self.supervisor.storage(), &scanned)
                        .map_err(|_| ServiceError::Internal)?;
                    Ok(RuntimeReply::Response(RuntimeResponse::SessionHistory(
                        entries,
                    )))
                }
            },
            RuntimeCall::TerminalDescribe { session_id } => {
                find_session(&self.supervisor, &session_id)?;
                let output_path = self.supervisor.output_log_path(&session_id);
                let geometry = self
                    .supervisor
                    .holder_snapshot(&session_id)
                    .filter(|holder| holder.status.as_deref() == Some("running"))
                    .and_then(|holder| holder.cols.zip(holder.rows))
                    .filter(|(cols, rows)| *cols > 0 && *rows > 0)
                    .unwrap_or((120, 40));
                Ok(RuntimeReply::TerminalDescriptor(TerminalSourceDescriptor {
                    session_id,
                    output_path,
                    cols: geometry.0,
                    rows: geometry.1,
                    modes: Vec::new(),
                }))
            }
            RuntimeCall::TerminalInput { session_id, bytes } => {
                self.supervisor
                    .send_bytes(&session_id, &bytes)
                    .map_err(|_| ServiceError::Internal)?;
                Ok(ack())
            }
            RuntimeCall::TerminalResize {
                session_id,
                cols,
                rows,
            } => {
                self.supervisor
                    .resize_session(&session_id, cols, rows)
                    .map_err(|_| ServiceError::Internal)?;
                Ok(ack())
            }
        }
    }

    fn prepare_shutdown(&mut self) -> ServiceResult<()> {
        self.supervisor
            .prepare_shutdown()
            .map_err(|_| ServiceError::Internal)
    }
}

fn session_to_proto(session: SessionSummary) -> ProtoSessionSummary {
    ProtoSessionSummary {
        id: session.id,
        title: session.title,
        status: session.status,
        workspace: session.workspace,
        agent_profile_id: session.agent_profile_id,
        runtime_id: session.runtime_id,
        llm_profile_id: session.llm_profile_id,
        permission_profile_id: session.permission_profile_id,
    }
}

fn find_session(supervisor: &RuntimeSupervisor, session_id: &str) -> ServiceResult<SessionSummary> {
    supervisor
        .list_sessions()
        .map_err(|_| ServiceError::Internal)?
        .into_iter()
        .find(|session| session.id == session_id)
        .ok_or_else(|| ServiceError::BadRequest(format!("unknown session: {session_id}")))
}

fn holder_to_proto(holder: crate::HolderSnapshot) -> ProtoHolderSnapshot {
    ProtoHolderSnapshot {
        pid: holder.pid,
        status: holder.status,
        tree_size: holder.tree_size,
        cols: holder.cols,
        rows: holder.rows,
        log_offset: holder.log_offset,
        epoch_offset: holder.epoch_offset,
    }
}

fn ack() -> RuntimeReply {
    RuntimeReply::Response(RuntimeResponse::Ack(homie_proto::transport::AckResult {
        ok: true,
    }))
}

enum ActorCommand {
    Call {
        request: RuntimeCall,
        reply: oneshot::Sender<ServiceResult<RuntimeReply>>,
    },
    PrepareShutdown {
        reply: oneshot::Sender<ServiceResult<()>>,
    },
    Shutdown {
        reply: oneshot::Sender<()>,
    },
}

#[derive(Clone)]
pub struct RuntimeActorHandle {
    sender: mpsc::SyncSender<ActorCommand>,
    accepting: Arc<AtomicBool>,
}

impl RuntimeActorHandle {
    pub fn try_call(
        &self,
        request: RuntimeCall,
    ) -> ServiceResult<oneshot::Receiver<ServiceResult<RuntimeReply>>> {
        if !self.accepting.load(Ordering::Acquire) && !allow_during_drain(&request) {
            return Err(ServiceError::Unavailable);
        }
        let (reply, receiver) = oneshot::channel();
        self.sender
            .try_send(ActorCommand::Call { request, reply })
            .map_err(map_try_send_error)?;
        Ok(receiver)
    }

    pub fn prepare_shutdown(&self) -> ServiceResult<oneshot::Receiver<ServiceResult<()>>> {
        if !self.accepting.load(Ordering::Acquire) {
            return Err(ServiceError::Unavailable);
        }
        let (reply, receiver) = oneshot::channel();
        self.sender
            .try_send(ActorCommand::PrepareShutdown { reply })
            .map_err(map_try_send_error)?;
        Ok(receiver)
    }
}

pub struct RuntimeActor {
    handle: RuntimeActorHandle,
    join: JoinHandle<()>,
}

impl RuntimeActor {
    pub fn spawn(backend: impl RuntimeBackend) -> io::Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(ACTOR_QUEUE_CAPACITY);
        let accepting = Arc::new(AtomicBool::new(true));
        let thread_accepting = accepting.clone();
        let join = thread::Builder::new()
            .name("homie-runtime-actor".to_string())
            .spawn(move || run_actor(backend, receiver, thread_accepting))?;
        Ok(Self {
            handle: RuntimeActorHandle { sender, accepting },
            join,
        })
    }

    #[must_use]
    pub fn handle(&self) -> RuntimeActorHandle {
        self.handle.clone()
    }

    pub fn shutdown(self) -> ServiceResult<()> {
        self.handle.accepting.store(false, Ordering::Release);
        let (reply, receiver) = oneshot::channel();
        self.handle
            .sender
            .send(ActorCommand::Shutdown { reply })
            .map_err(|_| ServiceError::Unavailable)?;
        receiver
            .blocking_recv()
            .map_err(|_| ServiceError::Unavailable)?;
        self.join.join().map_err(|_| ServiceError::Internal)
    }

    pub async fn shutdown_async(self) -> ServiceResult<()> {
        tokio::task::spawn_blocking(move || self.shutdown())
            .await
            .map_err(|_| ServiceError::Internal)?
    }
}

fn run_actor(
    mut backend: impl RuntimeBackend,
    receiver: mpsc::Receiver<ActorCommand>,
    accepting: Arc<AtomicBool>,
) {
    while let Ok(command) = receiver.recv() {
        match command {
            ActorCommand::Call { request, reply } => {
                let stops_admission = matches!(
                    &request,
                    RuntimeCall::Invoke(ActorRequest::PrepareShutdown | ActorRequest::Shutdown)
                );
                let result = if accepting.load(Ordering::Acquire) || allow_during_drain(&request) {
                    backend.call(request)
                } else {
                    Err(ServiceError::Unavailable)
                };
                if stops_admission && result.is_ok() {
                    accepting.store(false, Ordering::Release);
                }
                let _ = reply.send(result);
            }
            ActorCommand::PrepareShutdown { reply } => {
                accepting.store(false, Ordering::Release);
                let _ = reply.send(backend.prepare_shutdown());
            }
            ActorCommand::Shutdown { reply } => {
                accepting.store(false, Ordering::Release);
                backend.shutdown();
                let _ = reply.send(());
                break;
            }
        }
    }
    accepting.store(false, Ordering::Release);
}

fn map_try_send_error<T>(error: mpsc::TrySendError<T>) -> ServiceError {
    match error {
        mpsc::TrySendError::Full(_) => ServiceError::Backpressure,
        mpsc::TrySendError::Disconnected(_) => ServiceError::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;
    use std::time::Duration;

    use super::*;

    struct RecordingBackend {
        calls: Arc<Mutex<Vec<String>>>,
        gate: Option<Arc<(Mutex<bool>, Condvar)>>,
    }

    impl RuntimeBackend for RecordingBackend {
        fn call(&mut self, request: RuntimeCall) -> ServiceResult<RuntimeReply> {
            if let Some(gate) = &self.gate {
                let (open, changed) = &**gate;
                let mut open = open.lock().expect("gate");
                while !*open {
                    open = changed.wait(open).expect("gate wait");
                }
            }
            let thread_name = thread::current().name().unwrap_or_default().to_string();
            self.calls.lock().expect("calls").push(thread_name.clone());
            match request {
                RuntimeCall::Invoke(ActorRequest::SessionList) => Ok(RuntimeReply::Response(
                    RuntimeResponse::Ack(homie_proto::transport::AckResult { ok: true }),
                )),
                RuntimeCall::Invoke(ActorRequest::Shutdown) => {
                    Ok(RuntimeReply::Response(RuntimeResponse::Shutdown(
                        homie_proto::transport::ShutdownResult { acknowledged: true },
                    )))
                }
                RuntimeCall::CommitLongRunning(result) => match *result {
                    LongRunningResult::Response(response) => Ok(RuntimeReply::Response(*response)),
                    LongRunningResult::HistoryScan(_) => panic!("unexpected history scan"),
                },
                _ => panic!("unexpected request"),
            }
        }
    }

    #[test]
    fn calls_run_in_order_on_the_named_owner_thread() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let actor = RuntimeActor::spawn(RecordingBackend {
            calls: calls.clone(),
            gate: None,
        })
        .expect("spawn actor");
        let handle = actor.handle();

        let first = handle
            .try_call(RuntimeCall::Invoke(ActorRequest::SessionList))
            .expect("first");
        let second = handle
            .try_call(RuntimeCall::Invoke(ActorRequest::SessionList))
            .expect("second");

        assert_eq!(
            first
                .blocking_recv()
                .expect("first reply")
                .expect("first call"),
            RuntimeReply::Response(RuntimeResponse::Ack(homie_proto::transport::AckResult {
                ok: true
            }))
        );
        assert_eq!(
            second
                .blocking_recv()
                .expect("second reply")
                .expect("second call"),
            RuntimeReply::Response(RuntimeResponse::Ack(homie_proto::transport::AckResult {
                ok: true
            }))
        );
        assert_eq!(
            calls.lock().expect("calls").as_slice(),
            ["homie-runtime-actor", "homie-runtime-actor"]
        );
        actor.shutdown().expect("shutdown");
    }

    #[test]
    fn the_257th_pending_call_is_rejected_with_backpressure() {
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let actor = RuntimeActor::spawn(RecordingBackend {
            calls: Arc::new(Mutex::new(Vec::new())),
            gate: Some(gate.clone()),
        })
        .expect("spawn actor");
        let handle = actor.handle();

        let running = handle
            .try_call(RuntimeCall::Invoke(ActorRequest::SessionList))
            .expect("running");
        thread::sleep(Duration::from_millis(20));
        let pending = (0..ACTOR_QUEUE_CAPACITY)
            .map(|_| {
                handle
                    .try_call(RuntimeCall::Invoke(ActorRequest::SessionList))
                    .expect("pending")
            })
            .collect::<Vec<_>>();

        assert_eq!(
            handle
                .try_call(RuntimeCall::Invoke(ActorRequest::SessionList))
                .expect_err("full"),
            ServiceError::Backpressure
        );

        let (open, changed) = &*gate;
        *open.lock().expect("gate") = true;
        changed.notify_all();
        running
            .blocking_recv()
            .expect("running reply")
            .expect("call");
        for reply in pending {
            reply.blocking_recv().expect("pending reply").expect("call");
        }
        actor.shutdown().expect("shutdown");
    }

    #[test]
    fn prepare_shutdown_rejects_later_calls() {
        let actor = RuntimeActor::spawn(RecordingBackend {
            calls: Arc::new(Mutex::new(Vec::new())),
            gate: None,
        })
        .expect("spawn actor");
        let handle = actor.handle();

        handle
            .prepare_shutdown()
            .expect("prepare command")
            .blocking_recv()
            .expect("prepare reply")
            .expect("prepare");

        assert_eq!(
            handle
                .try_call(RuntimeCall::Invoke(ActorRequest::SessionList))
                .expect_err("rejected"),
            ServiceError::Unavailable
        );
        actor.shutdown().expect("shutdown");
    }

    #[test]
    fn prepare_shutdown_allows_shutdown_request() {
        let actor = RuntimeActor::spawn(RecordingBackend {
            calls: Arc::new(Mutex::new(Vec::new())),
            gate: None,
        })
        .expect("spawn actor");
        let handle = actor.handle();

        handle
            .prepare_shutdown()
            .expect("prepare command")
            .blocking_recv()
            .expect("prepare reply")
            .expect("prepare");

        assert_eq!(
            handle
                .try_call(RuntimeCall::Invoke(ActorRequest::Shutdown))
                .expect("shutdown request")
                .blocking_recv()
                .expect("shutdown reply")
                .expect("shutdown call"),
            RuntimeReply::Response(RuntimeResponse::Shutdown(
                homie_proto::transport::ShutdownResult { acknowledged: true }
            ))
        );
        actor.shutdown().expect("shutdown");
    }

    #[test]
    fn prepare_shutdown_allows_long_running_commit() {
        let actor = RuntimeActor::spawn(RecordingBackend {
            calls: Arc::new(Mutex::new(Vec::new())),
            gate: None,
        })
        .expect("spawn actor");
        let handle = actor.handle();

        handle
            .prepare_shutdown()
            .expect("prepare command")
            .blocking_recv()
            .expect("prepare reply")
            .expect("prepare");

        let response = RuntimeResponse::Ack(homie_proto::transport::AckResult { ok: true });
        assert_eq!(
            handle
                .try_call(RuntimeCall::CommitLongRunning(Box::new(
                    LongRunningResult::Response(Box::new(response.clone())),
                )))
                .expect("commit request")
                .blocking_recv()
                .expect("commit reply")
                .expect("commit call"),
            RuntimeReply::Response(response)
        );
        actor.shutdown().expect("shutdown");
    }

    #[test]
    fn stopped_actor_rejects_all_calls() {
        let actor = RuntimeActor::spawn(RecordingBackend {
            calls: Arc::new(Mutex::new(Vec::new())),
            gate: None,
        })
        .expect("spawn actor");
        let handle = actor.handle();

        actor.shutdown().expect("shutdown");

        let calls = [
            RuntimeCall::Invoke(ActorRequest::SessionList),
            RuntimeCall::Invoke(ActorRequest::Shutdown),
            RuntimeCall::PrepareLongRunning(LongRunningRequest::WorktreeOverview),
            RuntimeCall::CommitLongRunning(Box::new(LongRunningResult::Response(Box::new(
                RuntimeResponse::Ack(homie_proto::transport::AckResult { ok: true }),
            )))),
            RuntimeCall::TerminalDescribe {
                session_id: "session".to_string(),
            },
            RuntimeCall::TerminalInput {
                session_id: "session".to_string(),
                bytes: Vec::new(),
            },
            RuntimeCall::TerminalResize {
                session_id: "session".to_string(),
                cols: 120,
                rows: 40,
            },
        ];
        for call in calls {
            assert_eq!(
                handle.try_call(call).expect_err("rejected"),
                ServiceError::Unavailable
            );
        }
        assert_eq!(
            handle.prepare_shutdown().expect_err("rejected"),
            ServiceError::Unavailable
        );
    }
}
