use std::collections::HashSet;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use homie_proto::model::{
    ArtifactScan, HolderSnapshot as ProtoHolderSnapshot, HookReportRequest, PortListRow,
    RuntimeScreenObservation as ProtoScreenObservation, ScannedHistoryEntry,
    SessionArtifactsRequest, SessionChildrenRequest, SessionParentRequest, SessionParentResult,
    SessionPortsRequest, SessionSetParentRequest, SessionSnapshot, SessionSnapshotRequest,
    SessionStatusReport, SessionStatusRequest, SessionSummary, StateSnapshot,
    WorktreeOverviewResult,
};
use homie_proto::{
    EventsWaitRequest, HostLocateRepoParams, HostLocateRepoResult, Method, SessionHistoryRequest,
    SessionKillRequest, SessionReadDiffRequest, SessionReadDiffResult, SessionResizeRequest,
    SessionResumeFromHistoryRequest, SessionSendTextRequest, SessionSpawnRequest,
    WorktreeCreateRequest, WorktreeInfo, WorktreeListRequest, WorktreeRemoveRequest,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, to_value};

use crate::long_running::{CancellationToken, JobContext, JobOptions, LongRunningLaneHandle};
use crate::runtime_actor::{RuntimeActorHandle, RuntimeCall, ServiceError, ServiceResult};

const READ_DEADLINE: Duration = Duration::from_secs(10);
const GIT_DEADLINE: Duration = Duration::from_secs(15);
const HISTORY_DEADLINE: Duration = Duration::from_secs(30);
const WORKTREE_MUTATION_DEADLINE: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandlerClass {
    Actor,
    LongRunning,
    AsyncWait,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HandlerRegistration {
    pub method: &'static str,
    pub class: HandlerClass,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActorRequest {
    StateSnapshot,
    PrepareShutdown,
    Shutdown,
    SessionSpawn(SessionSpawnRequest),
    SessionList,
    SessionSetParent(SessionSetParentRequest),
    SessionListChildren(SessionChildrenRequest),
    SessionParent(SessionParentRequest),
    SessionResumeFromHistory(SessionResumeFromHistoryRequest),
    SessionSendText(SessionSendTextRequest),
    SessionResize(SessionResizeRequest),
    SessionKill(SessionKillRequest),
    HookReport(HookReportRequest),
}

impl ActorRequest {
    #[must_use]
    pub const fn method(&self) -> &'static str {
        match self {
            Self::StateSnapshot => Method::STATE_SNAPSHOT,
            Self::PrepareShutdown => Method::DAEMON_PREPARE_SHUTDOWN,
            Self::Shutdown => Method::DAEMON_SHUTDOWN,
            Self::SessionSpawn(_) => Method::SESSION_SPAWN,
            Self::SessionList => Method::SESSION_LIST,
            Self::SessionSetParent(_) => Method::SESSION_SET_PARENT,
            Self::SessionListChildren(_) => Method::SESSION_LIST_CHILDREN,
            Self::SessionParent(_) => Method::SESSION_PARENT,
            Self::SessionResumeFromHistory(_) => Method::SESSION_RESUME_FROM_HISTORY,
            Self::SessionSendText(_) => Method::SESSION_SEND_TEXT,
            Self::SessionResize(_) => Method::SESSION_RESIZE,
            Self::SessionKill(_) => Method::SESSION_KILL,
            Self::HookReport(_) => Method::HOOK_REPORT,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LongRunningRequest {
    SessionSnapshot(SessionSnapshotRequest),
    SessionStatus(SessionStatusRequest),
    SessionArtifacts(SessionArtifactsRequest),
    SessionPorts(SessionPortsRequest),
    SessionHistory(SessionHistoryRequest),
    SessionReadDiff(SessionReadDiffRequest),
    HostLocateRepo(HostLocateRepoParams),
    WorktreeList(WorktreeListRequest),
    WorktreeCreate(WorktreeCreateRequest),
    WorktreeRemove(WorktreeRemoveRequest),
    WorktreeOverview,
}

impl LongRunningRequest {
    #[must_use]
    pub const fn method(&self) -> &'static str {
        match self {
            Self::SessionSnapshot(_) => Method::SESSION_SNAPSHOT,
            Self::SessionStatus(_) => Method::SESSION_STATUS,
            Self::SessionArtifacts(_) => Method::SESSION_ARTIFACTS,
            Self::SessionPorts(_) => Method::SESSION_PORTS,
            Self::SessionHistory(_) => Method::SESSION_HISTORY,
            Self::SessionReadDiff(_) => Method::SESSION_READ_DIFF,
            Self::HostLocateRepo(_) => Method::HOST_LOCATE_REPO,
            Self::WorktreeList(_) => Method::WORKTREE_LIST,
            Self::WorktreeCreate(_) => Method::WORKTREE_CREATE,
            Self::WorktreeRemove(_) => Method::WORKTREE_REMOVE,
            Self::WorktreeOverview => Method::WORKTREE_OVERVIEW,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeResponse {
    StateSnapshot(StateSnapshot),
    Ack(homie_proto::transport::AckResult),
    Shutdown(homie_proto::transport::ShutdownResult),
    Session(SessionSummary),
    Sessions(Vec<SessionSummary>),
    SessionSnapshot(Box<SessionSnapshot>),
    SessionStatus(SessionStatusReport),
    SessionArtifacts(ArtifactScan),
    SessionPorts(Vec<PortListRow>),
    SessionParent(SessionParentResult),
    SessionHistory(Vec<ScannedHistoryEntry>),
    SessionReadDiff(SessionReadDiffResult),
    HostLocateRepo(HostLocateRepoResult),
    WorktreeList(Vec<WorktreeInfo>),
    Worktree(WorktreeInfo),
    WorktreeOverview(WorktreeOverviewResult),
}

impl RuntimeResponse {
    fn into_value(self) -> ServiceResult<Value> {
        match self {
            Self::StateSnapshot(value) => encode_params(value),
            Self::Ack(value) => encode_params(value),
            Self::Shutdown(value) => encode_params(value),
            Self::Session(value) => encode_params(value),
            Self::Sessions(value) => encode_params(value),
            Self::SessionSnapshot(value) => encode_params(value),
            Self::SessionStatus(value) => encode_params(value),
            Self::SessionArtifacts(value) => encode_params(value),
            Self::SessionPorts(value) => encode_params(value),
            Self::SessionParent(value) => encode_params(value),
            Self::SessionHistory(value) => encode_params(value),
            Self::SessionReadDiff(value) => encode_params(value),
            Self::HostLocateRepo(value) => encode_params(value),
            Self::WorktreeList(value) => encode_params(value),
            Self::Worktree(value) => encode_params(value),
            Self::WorktreeOverview(value) => encode_params(value),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PreparedLongRunning {
    Request(LongRunningRequest),
    SessionSnapshot {
        request: SessionSnapshotRequest,
        session: Box<SessionSummary>,
        output_path: PathBuf,
        status: crate::SessionStatusPreparation,
        holder: Option<ProtoHolderSnapshot>,
    },
    SessionStatus {
        output_path: PathBuf,
        status: crate::SessionStatusPreparation,
    },
    SessionArtifacts {
        output_path: PathBuf,
    },
    SessionPorts {
        sessions: Vec<PreparedPortSession>,
    },
    HostLocateRepo {
        request: HostLocateRepoParams,
        candidates: Vec<PathBuf>,
    },
    WorktreeOverview {
        projects: Vec<PathBuf>,
        sessions: Vec<SessionSummary>,
    },
    SessionReadDiff {
        cwd: PathBuf,
        comparison: homie_proto::SessionDiffBase,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreparedPortSession {
    pub session_id: String,
    pub session_title: String,
    pub output_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LongRunningResult {
    Response(Box<RuntimeResponse>),
    HistoryScan(Vec<ScannedHistoryEntry>),
}

#[derive(Clone, Debug, PartialEq)]
enum DispatchRequest {
    Actor(ActorRequest),
    LongRunning(LongRunningRequest),
    AsyncWait(EventsWaitRequest),
}

impl DispatchRequest {
    const fn class(&self) -> HandlerClass {
        match self {
            Self::Actor(_) => HandlerClass::Actor,
            Self::LongRunning(_) => HandlerClass::LongRunning,
            Self::AsyncWait(_) => HandlerClass::AsyncWait,
        }
    }
}

impl HandlerRegistration {
    fn decode(&self, params: Value) -> ServiceResult<DispatchRequest> {
        let request = match self.method {
            Method::STATE_SNAPSHOT => DispatchRequest::Actor(ActorRequest::StateSnapshot),
            Method::EVENTS_WAIT => DispatchRequest::AsyncWait(decode_params(params)?),
            Method::DAEMON_PREPARE_SHUTDOWN => {
                DispatchRequest::Actor(ActorRequest::PrepareShutdown)
            }
            Method::DAEMON_SHUTDOWN => DispatchRequest::Actor(ActorRequest::Shutdown),
            Method::SESSION_SPAWN => {
                DispatchRequest::Actor(ActorRequest::SessionSpawn(decode_params(params)?))
            }
            Method::SESSION_LIST => DispatchRequest::Actor(ActorRequest::SessionList),
            Method::SESSION_SNAPSHOT => DispatchRequest::LongRunning(
                LongRunningRequest::SessionSnapshot(decode_params(params)?),
            ),
            Method::SESSION_STATUS => DispatchRequest::LongRunning(
                LongRunningRequest::SessionStatus(decode_params(params)?),
            ),
            Method::SESSION_ARTIFACTS => DispatchRequest::LongRunning(
                LongRunningRequest::SessionArtifacts(decode_params(params)?),
            ),
            Method::SESSION_PORTS => DispatchRequest::LongRunning(
                LongRunningRequest::SessionPorts(decode_params(params)?),
            ),
            Method::SESSION_SET_PARENT => {
                DispatchRequest::Actor(ActorRequest::SessionSetParent(decode_params(params)?))
            }
            Method::SESSION_LIST_CHILDREN => {
                DispatchRequest::Actor(ActorRequest::SessionListChildren(decode_params(params)?))
            }
            Method::SESSION_PARENT => {
                DispatchRequest::Actor(ActorRequest::SessionParent(decode_params(params)?))
            }
            Method::SESSION_HISTORY => DispatchRequest::LongRunning(
                LongRunningRequest::SessionHistory(decode_params(params)?),
            ),
            Method::SESSION_RESUME_FROM_HISTORY => DispatchRequest::Actor(
                ActorRequest::SessionResumeFromHistory(decode_params(params)?),
            ),
            Method::SESSION_READ_DIFF => DispatchRequest::LongRunning(
                LongRunningRequest::SessionReadDiff(decode_params(params)?),
            ),
            Method::SESSION_SEND_TEXT => {
                DispatchRequest::Actor(ActorRequest::SessionSendText(decode_params(params)?))
            }
            Method::SESSION_RESIZE => {
                DispatchRequest::Actor(ActorRequest::SessionResize(decode_params(params)?))
            }
            Method::SESSION_KILL => {
                DispatchRequest::Actor(ActorRequest::SessionKill(decode_params(params)?))
            }
            Method::HOST_LOCATE_REPO => DispatchRequest::LongRunning(
                LongRunningRequest::HostLocateRepo(decode_params(params)?),
            ),
            Method::WORKTREE_LIST => DispatchRequest::LongRunning(
                LongRunningRequest::WorktreeList(decode_params(params)?),
            ),
            Method::WORKTREE_CREATE => DispatchRequest::LongRunning(
                LongRunningRequest::WorktreeCreate(decode_params(params)?),
            ),
            Method::WORKTREE_REMOVE => DispatchRequest::LongRunning(
                LongRunningRequest::WorktreeRemove(decode_params(params)?),
            ),
            Method::WORKTREE_OVERVIEW => {
                DispatchRequest::LongRunning(LongRunningRequest::WorktreeOverview)
            }
            Method::HOOK_REPORT => {
                DispatchRequest::Actor(ActorRequest::HookReport(decode_params(params)?))
            }
            _ => return Err(ServiceError::MethodNotFound(self.method.to_string())),
        };
        if request.class() != self.class {
            return Err(ServiceError::Internal);
        }
        Ok(request)
    }
}

const REQUEST_HANDLERS: [HandlerRegistration; 25] = [
    handler(Method::STATE_SNAPSHOT, HandlerClass::Actor),
    handler(Method::EVENTS_WAIT, HandlerClass::AsyncWait),
    handler(Method::DAEMON_PREPARE_SHUTDOWN, HandlerClass::Actor),
    handler(Method::DAEMON_SHUTDOWN, HandlerClass::Actor),
    handler(Method::SESSION_SPAWN, HandlerClass::Actor),
    handler(Method::SESSION_LIST, HandlerClass::Actor),
    handler(Method::SESSION_SNAPSHOT, HandlerClass::LongRunning),
    handler(Method::SESSION_STATUS, HandlerClass::LongRunning),
    handler(Method::SESSION_ARTIFACTS, HandlerClass::LongRunning),
    handler(Method::SESSION_PORTS, HandlerClass::LongRunning),
    handler(Method::SESSION_SET_PARENT, HandlerClass::Actor),
    handler(Method::SESSION_LIST_CHILDREN, HandlerClass::Actor),
    handler(Method::SESSION_PARENT, HandlerClass::Actor),
    handler(Method::SESSION_HISTORY, HandlerClass::LongRunning),
    handler(Method::SESSION_RESUME_FROM_HISTORY, HandlerClass::Actor),
    handler(Method::SESSION_READ_DIFF, HandlerClass::LongRunning),
    handler(Method::SESSION_SEND_TEXT, HandlerClass::Actor),
    handler(Method::SESSION_RESIZE, HandlerClass::Actor),
    handler(Method::SESSION_KILL, HandlerClass::Actor),
    handler(Method::HOST_LOCATE_REPO, HandlerClass::LongRunning),
    handler(Method::WORKTREE_LIST, HandlerClass::LongRunning),
    handler(Method::WORKTREE_CREATE, HandlerClass::LongRunning),
    handler(Method::WORKTREE_REMOVE, HandlerClass::LongRunning),
    handler(Method::WORKTREE_OVERVIEW, HandlerClass::LongRunning),
    handler(Method::HOOK_REPORT, HandlerClass::Actor),
];

const fn handler(method: &'static str, class: HandlerClass) -> HandlerRegistration {
    HandlerRegistration { method, class }
}

#[must_use]
pub fn request_handlers() -> &'static [HandlerRegistration] {
    &REQUEST_HANDLERS
}

#[must_use]
pub fn find_handler(method: &str) -> Option<&'static HandlerRegistration> {
    REQUEST_HANDLERS
        .iter()
        .find(|handler| handler.method == method)
}

pub trait LongRunningExecutor: Send + Sync + 'static {
    fn execute(
        &self,
        context: &JobContext,
        prepared: PreparedLongRunning,
    ) -> ServiceResult<LongRunningResult>;
}

pub struct RuntimeLongRunningExecutor;

impl LongRunningExecutor for RuntimeLongRunningExecutor {
    fn execute(
        &self,
        context: &JobContext,
        prepared: PreparedLongRunning,
    ) -> ServiceResult<LongRunningResult> {
        match prepared {
            PreparedLongRunning::SessionSnapshot {
                request,
                session,
                output_path,
                status,
                holder,
            } => {
                let max_bytes = request.max_bytes.min(4 * 1024 * 1024);
                let (output_offset, output) = crate::read_file_range_bounded(
                    context,
                    &output_path,
                    request.output_offset,
                    max_bytes,
                )?;
                let status_output =
                    crate::read_file_bounded(context, &output_path, 4 * 1024 * 1024)?;
                Ok(long_running_response(RuntimeResponse::SessionSnapshot(
                    Box::new(SessionSnapshot {
                        session: *session,
                        status: status_to_proto(crate::status_report_from_output(
                            &status,
                            &status_output,
                        )),
                        output_offset,
                        output_text: String::from_utf8_lossy(&output).into_owned(),
                        holder,
                    }),
                )))
            }
            PreparedLongRunning::SessionStatus {
                output_path,
                status,
            } => {
                let output = crate::read_file_bounded(context, &output_path, 4 * 1024 * 1024)?;
                Ok(long_running_response(RuntimeResponse::SessionStatus(
                    status_to_proto(crate::status_report_from_output(&status, &output)),
                )))
            }
            PreparedLongRunning::SessionArtifacts { output_path } => {
                let output = crate::read_file_bounded(context, &output_path, 4 * 1024 * 1024)?;
                Ok(long_running_response(RuntimeResponse::SessionArtifacts(
                    artifacts_to_proto(crate::scan_artifacts(&String::from_utf8_lossy(&output))),
                )))
            }
            PreparedLongRunning::SessionPorts { sessions } => {
                let mut rows = Vec::new();
                for session in sessions {
                    let output =
                        crate::read_file_bounded(context, &session.output_path, 4 * 1024 * 1024)?;
                    rows.extend(
                        crate::scan_artifacts(&String::from_utf8_lossy(&output))
                            .ports
                            .into_iter()
                            .map(|port| PortListRow {
                                port: port.port,
                                url: port.url,
                                session_id: session.session_id.clone(),
                                session_title: session.session_title.clone(),
                            }),
                    );
                }
                rows.sort_by(|left, right| {
                    left.port
                        .cmp(&right.port)
                        .then_with(|| left.session_id.cmp(&right.session_id))
                });
                Ok(long_running_response(RuntimeResponse::SessionPorts(rows)))
            }
            PreparedLongRunning::HostLocateRepo {
                request,
                candidates,
            } => {
                let result = crate::locate_repo_bounded(context, request, &candidates)?;
                Ok(long_running_response(RuntimeResponse::HostLocateRepo(
                    result,
                )))
            }
            PreparedLongRunning::WorktreeOverview { projects, sessions } => {
                let result = crate::worktree_overview_bounded(context, &projects, &sessions)?;
                Ok(long_running_response(RuntimeResponse::WorktreeOverview(
                    result,
                )))
            }
            PreparedLongRunning::Request(LongRunningRequest::SessionHistory(request)) => {
                execute_history_scan(context, request)
            }
            PreparedLongRunning::Request(LongRunningRequest::WorktreeList(request)) => {
                let repo_path = PathBuf::from(request.repo_path);
                let worktrees = crate::list_git_worktrees_bounded(context, &repo_path)?;
                Ok(long_running_response(RuntimeResponse::WorktreeList(
                    worktrees,
                )))
            }
            PreparedLongRunning::Request(LongRunningRequest::WorktreeCreate(request)) => {
                let worktree = crate::create_worktree_bounded(context, request)?;
                Ok(long_running_response(RuntimeResponse::Worktree(worktree)))
            }
            PreparedLongRunning::Request(LongRunningRequest::WorktreeRemove(request)) => {
                crate::remove_worktree_bounded(context, request)?;
                Ok(long_running_response(RuntimeResponse::Ack(
                    homie_proto::transport::AckResult { ok: true },
                )))
            }
            PreparedLongRunning::Request(_) => Err(ServiceError::Internal),
            PreparedLongRunning::SessionReadDiff { cwd, comparison } => {
                let result = crate::read_git_diff_bounded(context, &cwd, comparison)?;
                Ok(long_running_response(RuntimeResponse::SessionReadDiff(
                    result,
                )))
            }
        }
    }
}

fn long_running_response(response: RuntimeResponse) -> LongRunningResult {
    LongRunningResult::Response(Box::new(response))
}

fn execute_history_scan(
    context: &JobContext,
    request: SessionHistoryRequest,
) -> ServiceResult<LongRunningResult> {
    let roots = crate::history::HistoryRoots {
        claude: PathBuf::from(request.claude_root),
        codex: PathBuf::from(request.codex_root),
    };
    let tracked = request.tracked.into_iter().collect::<HashSet<_>>();
    let mut interrupted = None;
    let entries = crate::history::scan_history_bounded(
        &roots,
        &tracked,
        crate::history::HistoryScanLimits::default(),
        || match context.checkpoint() {
            Ok(()) => Ok(()),
            Err(error) => {
                interrupted = Some(error);
                Err(crate::history::HistoryScanError::Interrupted)
            }
        },
    )
    .map_err(|error| {
        interrupted.unwrap_or(match error {
            crate::history::HistoryScanError::Interrupted => ServiceError::Cancelled,
            crate::history::HistoryScanError::Storage(_) => ServiceError::Internal,
        })
    })?;
    Ok(LongRunningResult::HistoryScan(
        entries.into_iter().map(history_to_proto).collect(),
    ))
}

fn history_to_proto(entry: crate::history::ScannedHistoryEntry) -> ScannedHistoryEntry {
    ScannedHistoryEntry {
        agent_kind: entry.agent_kind,
        external_id: entry.external_id,
        cwd: entry.cwd.display().to_string(),
        title: entry.title,
        title_source: entry.title_source,
        transcript_path: entry.transcript_path.display().to_string(),
        last_active_at: entry.last_active_at,
        created_at: entry.created_at,
        cwd_exists: entry.cwd_exists,
    }
}

fn status_to_proto(report: crate::SessionStatusReport) -> SessionStatusReport {
    SessionStatusReport {
        status: report.status,
        needs_input: report.needs_input,
        turn_completed: report.turn_completed,
        screen_lines: report.screen_lines,
        screen_observation: report
            .screen_observation
            .map(|observation| ProtoScreenObservation {
                state: format!("{:?}", observation.state).to_ascii_lowercase(),
                matched_rule_id: observation.matched_rule_id,
                content_seq: observation.content_seq,
            }),
    }
}

fn artifacts_to_proto(scan: crate::ArtifactScan) -> ArtifactScan {
    ArtifactScan {
        artifacts: scan
            .artifacts
            .into_iter()
            .map(|artifact| homie_proto::model::SessionArtifact {
                kind: match artifact.kind {
                    crate::ArtifactKind::PullRequest => {
                        homie_proto::model::ArtifactKind::PullRequest
                    }
                    crate::ArtifactKind::Preview => homie_proto::model::ArtifactKind::Preview,
                    crate::ArtifactKind::Link => homie_proto::model::ArtifactKind::Link,
                },
                url: artifact.url,
                label: artifact.label,
            })
            .collect(),
        ports: scan
            .ports
            .into_iter()
            .map(|port| homie_proto::model::ListeningPort {
                port: port.port,
                url: port.url,
            })
            .collect(),
    }
}

pub trait AsyncWaitHandler: Send + Sync + 'static {
    fn wait(
        &self,
        request: EventsWaitRequest,
    ) -> Pin<Box<dyn Future<Output = ServiceResult<Value>> + Send + '_>>;
}

#[derive(Clone)]
pub struct RuntimeDispatcher {
    actor: RuntimeActorHandle,
    lane: LongRunningLaneHandle,
    long_running: Arc<dyn LongRunningExecutor>,
    async_wait: Arc<dyn AsyncWaitHandler>,
}

impl RuntimeDispatcher {
    #[must_use]
    pub fn new(
        actor: RuntimeActorHandle,
        lane: LongRunningLaneHandle,
        long_running: Arc<dyn LongRunningExecutor>,
        async_wait: Arc<dyn AsyncWaitHandler>,
    ) -> Self {
        Self {
            actor,
            lane,
            long_running,
            async_wait,
        }
    }

    pub async fn dispatch(&self, method: &str, params: Value) -> ServiceResult<Value> {
        let handler =
            find_handler(method).ok_or_else(|| ServiceError::MethodNotFound(method.to_string()))?;
        match handler.decode(params)? {
            DispatchRequest::Actor(request) => self
                .call_actor(RuntimeCall::Invoke(request))
                .await?
                .into_response()?
                .into_value(),
            DispatchRequest::LongRunning(request) => self.dispatch_long_running(request).await,
            DispatchRequest::AsyncWait(request) => self.async_wait.wait(request).await,
        }
    }

    async fn dispatch_long_running(&self, request: LongRunningRequest) -> ServiceResult<Value> {
        let method = request.method();
        let is_mutation = matches!(method, Method::WORKTREE_CREATE | Method::WORKTREE_REMOVE);
        let prepared = self
            .call_actor(RuntimeCall::PrepareLongRunning(request))
            .await?
            .into_prepared()?;
        let executor = self.long_running.clone();
        let cancellation = CancellationToken::new();
        let guard = DispatchCancellation::new(cancellation.clone());
        let options = job_options(method).with_cancellation(cancellation);
        let receiver = self
            .lane
            .try_submit(options, move |context| executor.execute(context, prepared))?;
        if is_mutation {
            let actor = self.actor.clone();
            let coordinator = tokio::spawn(async move {
                let result = receiver.await.map_err(|_| ServiceError::Unavailable)??;
                actor
                    .try_call(RuntimeCall::CommitLongRunning(Box::new(result)))?
                    .await
                    .map_err(|_| ServiceError::Unavailable)??
                    .into_response()?
                    .into_value()
            });
            let response = coordinator.await.map_err(|_| ServiceError::Unavailable)?;
            guard.disarm();
            return response;
        }
        let result = receiver.await.map_err(|_| ServiceError::Unavailable)?;
        guard.disarm();
        let result = result?;
        self.call_actor(RuntimeCall::CommitLongRunning(Box::new(result)))
            .await?
            .into_response()?
            .into_value()
    }

    async fn call_actor(
        &self,
        request: RuntimeCall,
    ) -> ServiceResult<crate::runtime_actor::RuntimeReply> {
        self.actor
            .try_call(request)?
            .await
            .map_err(|_| ServiceError::Unavailable)?
    }
}

struct DispatchCancellation {
    token: CancellationToken,
    armed: bool,
}

impl DispatchCancellation {
    fn new(token: CancellationToken) -> Self {
        Self { token, armed: true }
    }

    fn disarm(mut self) {
        self.armed = false;
    }
}

impl Drop for DispatchCancellation {
    fn drop(&mut self) {
        if self.armed {
            self.token.cancel();
        }
    }
}

fn decode_params<T: DeserializeOwned>(params: Value) -> ServiceResult<T> {
    serde_json::from_value(params).map_err(|error| ServiceError::BadRequest(error.to_string()))
}

fn encode_params(params: impl Serialize) -> ServiceResult<Value> {
    to_value(params).map_err(|_| ServiceError::Internal)
}

fn job_options(method: &str) -> JobOptions {
    match method {
        Method::WORKTREE_CREATE | Method::WORKTREE_REMOVE => {
            JobOptions::mutation(WORKTREE_MUTATION_DEADLINE)
        }
        Method::SESSION_HISTORY => JobOptions::read_only(HISTORY_DEADLINE),
        Method::SESSION_READ_DIFF
        | Method::HOST_LOCATE_REPO
        | Method::WORKTREE_LIST
        | Method::WORKTREE_OVERVIEW => JobOptions::read_only(GIT_DEADLINE),
        _ => JobOptions::read_only(READ_DEADLINE),
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;
    use std::time::Duration;

    use serde_json::{Value, json};

    use super::*;
    use crate::long_running::{JobContext, LongRunningLane};
    use crate::runtime_actor::{
        RuntimeActor, RuntimeBackend, RuntimeCall, RuntimeReply, ServiceError, ServiceResult,
    };

    struct FakeBackend {
        calls: Arc<Mutex<Vec<&'static str>>>,
    }

    impl RuntimeBackend for FakeBackend {
        fn call(&mut self, request: RuntimeCall) -> ServiceResult<RuntimeReply> {
            match request {
                RuntimeCall::Invoke(request) => {
                    self.calls.lock().expect("calls").push(request.method());
                    Ok(RuntimeReply::Response(RuntimeResponse::Ack(
                        homie_proto::transport::AckResult { ok: true },
                    )))
                }
                RuntimeCall::PrepareLongRunning(request) => {
                    self.calls.lock().expect("calls").push("prepare");
                    Ok(RuntimeReply::Prepared(PreparedLongRunning::Request(
                        request,
                    )))
                }
                RuntimeCall::CommitLongRunning(result) => match *result {
                    LongRunningResult::Response(response) => {
                        self.calls.lock().expect("calls").push("commit");
                        Ok(RuntimeReply::Response(*response))
                    }
                    LongRunningResult::HistoryScan(entries) => {
                        self.calls.lock().expect("calls").push("commit");
                        Ok(RuntimeReply::Response(RuntimeResponse::SessionHistory(
                            entries,
                        )))
                    }
                },
                RuntimeCall::TerminalDescribe { .. }
                | RuntimeCall::TerminalInput { .. }
                | RuntimeCall::TerminalResize { .. } => Err(ServiceError::Internal),
            }
        }
    }

    struct FakeLongRunning {
        result: ServiceResult<LongRunningResult>,
        gate: Option<Arc<(Mutex<bool>, Condvar)>>,
    }

    impl LongRunningExecutor for FakeLongRunning {
        fn execute(
            &self,
            _context: &JobContext,
            _prepared: PreparedLongRunning,
        ) -> ServiceResult<LongRunningResult> {
            if let Some(gate) = &self.gate {
                let (open, changed) = &**gate;
                let mut open = open.lock().expect("gate");
                while !*open {
                    open = changed.wait(open).expect("wait");
                }
            }
            self.result.clone()
        }
    }

    fn long_running_ack() -> ServiceResult<LongRunningResult> {
        Ok(long_running_response(RuntimeResponse::Ack(
            homie_proto::transport::AckResult { ok: true },
        )))
    }

    struct CancellationProbe {
        started: Arc<AtomicBool>,
        cancelled: Arc<AtomicBool>,
        release: Arc<AtomicBool>,
    }

    impl LongRunningExecutor for CancellationProbe {
        fn execute(
            &self,
            context: &JobContext,
            _prepared: PreparedLongRunning,
        ) -> ServiceResult<LongRunningResult> {
            self.started.store(true, Ordering::SeqCst);
            loop {
                if let Err(error) = context.checkpoint() {
                    if error == ServiceError::Cancelled {
                        self.cancelled.store(true, Ordering::SeqCst);
                    }
                    return Err(error);
                }
                if self.release.load(Ordering::SeqCst) {
                    return Err(ServiceError::Cancelled);
                }
                thread::sleep(Duration::from_millis(5));
            }
        }
    }

    struct BlockingMutation {
        started: Arc<AtomicBool>,
        release: Arc<AtomicBool>,
    }

    impl LongRunningExecutor for BlockingMutation {
        fn execute(
            &self,
            context: &JobContext,
            prepared: PreparedLongRunning,
        ) -> ServiceResult<LongRunningResult> {
            assert!(matches!(
                prepared,
                PreparedLongRunning::Request(LongRunningRequest::WorktreeCreate(_))
            ));
            self.started.store(true, Ordering::SeqCst);
            while !self.release.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(5));
            }
            context.checkpoint()?;
            long_running_ack()
        }
    }

    struct QueuedMutationProbe {
        blocker_started: Arc<AtomicBool>,
        release_blocker: Arc<AtomicBool>,
        mutation_executed: Arc<AtomicBool>,
    }

    impl LongRunningExecutor for QueuedMutationProbe {
        fn execute(
            &self,
            context: &JobContext,
            prepared: PreparedLongRunning,
        ) -> ServiceResult<LongRunningResult> {
            match prepared {
                PreparedLongRunning::Request(LongRunningRequest::SessionHistory(_)) => {
                    self.blocker_started.store(true, Ordering::SeqCst);
                    while !self.release_blocker.load(Ordering::SeqCst) {
                        thread::sleep(Duration::from_millis(5));
                    }
                    context.checkpoint()?;
                    long_running_ack()
                }
                PreparedLongRunning::Request(LongRunningRequest::WorktreeRemove(_)) => {
                    self.mutation_executed.store(true, Ordering::SeqCst);
                    long_running_ack()
                }
                _ => panic!("unexpected prepared request"),
            }
        }
    }

    struct FakeAsyncWait;

    impl AsyncWaitHandler for FakeAsyncWait {
        fn wait(
            &self,
            _request: EventsWaitRequest,
        ) -> Pin<Box<dyn Future<Output = ServiceResult<Value>> + Send + '_>> {
            Box::pin(async { Ok(json!({"source": "async_wait"})) })
        }
    }

    struct Harness {
        dispatcher: RuntimeDispatcher,
        actor: RuntimeActor,
        lane: LongRunningLane,
        calls: Arc<Mutex<Vec<&'static str>>>,
    }

    impl Harness {
        fn new(executor: impl LongRunningExecutor) -> Self {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let actor = RuntimeActor::spawn(FakeBackend {
                calls: calls.clone(),
            })
            .expect("actor");
            let lane = LongRunningLane::spawn().expect("lane");
            let dispatcher = RuntimeDispatcher::new(
                actor.handle(),
                lane.handle(),
                Arc::new(executor),
                Arc::new(FakeAsyncWait),
            );
            Self {
                dispatcher,
                actor,
                lane,
                calls,
            }
        }

        async fn shutdown(self) {
            self.actor.shutdown_async().await.expect("actor shutdown");
            self.lane.shutdown_async().await.expect("lane shutdown");
        }
    }

    #[tokio::test]
    async fn actor_handlers_are_dispatched_to_the_actor() {
        let harness = Harness::new(FakeLongRunning {
            result: long_running_ack(),
            gate: None,
        });

        let result = harness
            .dispatcher
            .dispatch(Method::SESSION_LIST, json!({}))
            .await
            .expect("dispatch");

        assert_eq!(result, json!({"ok": true}));
        harness.shutdown().await;
    }

    #[tokio::test]
    async fn history_runs_prepare_lane_and_commit_in_order() {
        let harness = Harness::new(FakeLongRunning {
            result: long_running_ack(),
            gate: None,
        });

        let result = harness
            .dispatcher
            .dispatch(
                Method::SESSION_HISTORY,
                json!({"claudeRoot": "/tmp/claude", "codexRoot": "/tmp/codex"}),
            )
            .await
            .expect("dispatch");

        assert_eq!(result, json!({"ok": true}));
        assert_eq!(
            harness.calls.lock().expect("calls").as_slice(),
            ["prepare", "commit"]
        );
        harness.shutdown().await;
    }

    #[tokio::test]
    async fn failed_long_running_work_never_commits() {
        let harness = Harness::new(FakeLongRunning {
            result: Err(ServiceError::Timeout),
            gate: None,
        });

        let result = harness
            .dispatcher
            .dispatch(
                Method::SESSION_HISTORY,
                json!({"claudeRoot": "/tmp/claude", "codexRoot": "/tmp/codex"}),
            )
            .await;

        assert_eq!(result, Err(ServiceError::Timeout));
        assert_eq!(harness.calls.lock().expect("calls").as_slice(), ["prepare"]);
        harness.shutdown().await;
    }

    #[tokio::test]
    async fn async_wait_handlers_stay_outside_the_actor() {
        let harness = Harness::new(FakeLongRunning {
            result: long_running_ack(),
            gate: None,
        });

        let result = harness
            .dispatcher
            .dispatch(Method::EVENTS_WAIT, json!({}))
            .await
            .expect("dispatch");

        assert_eq!(result, json!({"source": "async_wait"}));
        assert!(harness.calls.lock().expect("calls").is_empty());
        harness.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_long_running_job_does_not_block_actor_requests() {
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let harness = Harness::new(FakeLongRunning {
            result: long_running_ack(),
            gate: Some(gate.clone()),
        });
        let dispatcher = harness.dispatcher.clone();
        let long = tokio::spawn(async move {
            dispatcher
                .dispatch(
                    Method::SESSION_HISTORY,
                    json!({"claudeRoot": "/tmp/claude", "codexRoot": "/tmp/codex"}),
                )
                .await
        });
        tokio::time::sleep(Duration::from_millis(20)).await;

        let actor_result = tokio::time::timeout(
            Duration::from_millis(100),
            harness.dispatcher.dispatch(Method::SESSION_LIST, json!({})),
        )
        .await
        .expect("actor stayed responsive")
        .expect("actor dispatch");

        assert_eq!(actor_result, json!({"ok": true}));
        *gate.0.lock().expect("gate") = true;
        gate.1.notify_all();
        long.await.expect("join").expect("long dispatch");
        harness.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dropping_a_read_only_dispatch_cancels_lane_work_without_commit() {
        let started = Arc::new(AtomicBool::new(false));
        let cancelled = Arc::new(AtomicBool::new(false));
        let release = Arc::new(AtomicBool::new(false));
        let harness = Harness::new(CancellationProbe {
            started: started.clone(),
            cancelled: cancelled.clone(),
            release: release.clone(),
        });
        let dispatcher = harness.dispatcher.clone();
        let task = tokio::spawn(async move {
            dispatcher
                .dispatch(
                    Method::SESSION_HISTORY,
                    json!({"claudeRoot": "/tmp/claude", "codexRoot": "/tmp/codex"}),
                )
                .await
        });
        tokio::time::timeout(Duration::from_secs(1), async {
            while !started.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("job started");

        task.abort();
        let _ = task.await;
        let observed = tokio::time::timeout(Duration::from_millis(150), async {
            while !cancelled.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .is_ok();
        release.store(true, Ordering::SeqCst);
        let calls = harness.calls.clone();
        harness.shutdown().await;

        assert!(observed, "lane context did not observe caller cancellation");
        assert_eq!(calls.lock().expect("calls").as_slice(), ["prepare"]);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dropping_a_started_mutation_dispatch_still_commits_lane_result() {
        let started = Arc::new(AtomicBool::new(false));
        let release = Arc::new(AtomicBool::new(false));
        let harness = Harness::new(BlockingMutation {
            started: started.clone(),
            release: release.clone(),
        });
        let dispatcher = harness.dispatcher.clone();
        let task = tokio::spawn(async move {
            dispatcher
                .dispatch(Method::WORKTREE_CREATE, json!({"repoPath": "/tmp/repo"}))
                .await
        });
        tokio::time::timeout(Duration::from_secs(1), async {
            while !started.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("mutation started");

        task.abort();
        let _ = task.await;
        release.store(true, Ordering::SeqCst);
        let committed = tokio::time::timeout(Duration::from_secs(1), async {
            while !harness.calls.lock().expect("calls").contains(&"commit") {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .is_ok();
        let calls = harness.calls.clone();
        harness.shutdown().await;

        assert!(committed, "started mutation result was not committed");
        assert_eq!(
            calls.lock().expect("calls").as_slice(),
            ["prepare", "commit"]
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dropping_a_queued_mutation_dispatch_skips_execution_and_commit() {
        let blocker_started = Arc::new(AtomicBool::new(false));
        let release_blocker = Arc::new(AtomicBool::new(false));
        let mutation_executed = Arc::new(AtomicBool::new(false));
        let harness = Harness::new(QueuedMutationProbe {
            blocker_started: blocker_started.clone(),
            release_blocker: release_blocker.clone(),
            mutation_executed: mutation_executed.clone(),
        });
        let blocker_dispatcher = harness.dispatcher.clone();
        let blocker = tokio::spawn(async move {
            blocker_dispatcher
                .dispatch(
                    Method::SESSION_HISTORY,
                    json!({"claudeRoot": "/tmp/claude", "codexRoot": "/tmp/codex"}),
                )
                .await
        });
        tokio::time::timeout(Duration::from_secs(1), async {
            while !blocker_started.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("blocker started");

        let mutation_dispatcher = harness.dispatcher.clone();
        let mutation = tokio::spawn(async move {
            mutation_dispatcher
                .dispatch(
                    Method::WORKTREE_REMOVE,
                    json!({
                        "repoPath": "/tmp/repo",
                        "worktreePath": "/tmp/worktree"
                    }),
                )
                .await
        });
        tokio::time::timeout(Duration::from_secs(1), async {
            while harness.calls.lock().expect("calls").len() < 2 {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("mutation prepared");
        tokio::time::sleep(Duration::from_millis(20)).await;

        mutation.abort();
        let _ = mutation.await;
        release_blocker.store(true, Ordering::SeqCst);
        blocker
            .await
            .expect("blocker join")
            .expect("blocker dispatch");
        let calls = harness.calls.clone();
        harness.shutdown().await;

        assert!(
            !mutation_executed.load(Ordering::SeqCst),
            "queued mutation executed after caller cancellation"
        );
        assert_eq!(
            calls.lock().expect("calls").as_slice(),
            ["prepare", "prepare", "commit"]
        );
    }

    #[tokio::test]
    async fn unknown_methods_fail_without_entering_any_backend() {
        let harness = Harness::new(FakeLongRunning {
            result: long_running_ack(),
            gate: None,
        });

        let result = harness
            .dispatcher
            .dispatch("future.method", json!({}))
            .await;

        assert_eq!(
            result,
            Err(ServiceError::MethodNotFound("future.method".to_string()))
        );
        assert!(harness.calls.lock().expect("calls").is_empty());
        harness.shutdown().await;
    }

    #[test]
    fn every_registered_handler_decodes_to_its_declared_execution_class() {
        let cases = [
            (Method::STATE_SNAPSHOT, json!({})),
            (Method::EVENTS_WAIT, json!({})),
            (Method::DAEMON_PREPARE_SHUTDOWN, json!({})),
            (Method::DAEMON_SHUTDOWN, json!({})),
            (Method::SESSION_SPAWN, json!({"cwd": "/tmp"})),
            (Method::SESSION_LIST, json!({})),
            (
                Method::SESSION_SNAPSHOT,
                json!({"sessionId": "session-1", "maxBytes": 1024}),
            ),
            (Method::SESSION_STATUS, json!({"sessionId": "session-1"})),
            (Method::SESSION_ARTIFACTS, json!({"sessionId": "session-1"})),
            (Method::SESSION_PORTS, json!({})),
            (
                Method::SESSION_SET_PARENT,
                json!({"sessionId": "session-1", "parentSessionId": "parent-1"}),
            ),
            (
                Method::SESSION_LIST_CHILDREN,
                json!({"parentSessionId": "parent-1"}),
            ),
            (Method::SESSION_PARENT, json!({"sessionId": "session-1"})),
            (
                Method::SESSION_HISTORY,
                json!({"claudeRoot": "/tmp/claude", "codexRoot": "/tmp/codex"}),
            ),
            (
                Method::SESSION_RESUME_FROM_HISTORY,
                json!({
                    "agentKind": "codex",
                    "externalId": "thread-1",
                    "cwd": "/tmp"
                }),
            ),
            (Method::SESSION_READ_DIFF, json!({"sessionID": "session-1"})),
            (
                Method::SESSION_SEND_TEXT,
                json!({"sessionId": "session-1", "text": "status", "submit": true}),
            ),
            (
                Method::SESSION_RESIZE,
                json!({"sessionId": "session-1", "cols": 120, "rows": 40}),
            ),
            (Method::SESSION_KILL, json!({"sessionId": "session-1"})),
            (Method::HOST_LOCATE_REPO, json!({})),
            (Method::WORKTREE_LIST, json!({"repoPath": "/tmp/repo"})),
            (Method::WORKTREE_CREATE, json!({"repoPath": "/tmp/repo"})),
            (
                Method::WORKTREE_REMOVE,
                json!({"repoPath": "/tmp/repo", "worktreePath": "/tmp/worktree"}),
            ),
            (Method::WORKTREE_OVERVIEW, json!({})),
            (
                Method::HOOK_REPORT,
                json!({"sessionId": "session-1", "event": "turn_completed"}),
            ),
        ];

        for (method, params) in cases {
            let handler = find_handler(method).expect("registered handler");
            let decoded = handler.decode(params).expect("typed request");
            assert_eq!(decoded.class(), handler.class, "{method}");
        }
    }
}
