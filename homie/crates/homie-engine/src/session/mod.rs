//! A live session: one child process on a PTY, watched.
//!
//! This is where the previous layers meet. A session appends everything the
//! child writes to its [`OutputLog`], feeds the same bytes to a
//! [`HeadlessScreen`], evaluates the screen against the agent's manifest, and
//! folds the result through a [`StatusReducer`]. The current status and the
//! output log are what everything else in the product reads.
//!
//! Who owns the PTY is a transport choice. A *direct* session owns it in
//! process — simple, and gone when this process is. A *held* session's PTY
//! belongs to a holder (see [`crate::holder`]): the session is then only a
//! client and a log tail, and the child survives this process dying. Held is
//! what the daemon uses; direct remains for tests and embedded callers.
//!
//! The pump runs on its own thread rather than the async runtime, because the
//! PTY read is a blocking syscall — the same reasoning that moved the test
//! servers off the cooperative pool earlier tonight.

use std::io::Read;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime};

use homie_proto::frames::FrameType;
use homie_proto::remote_pty::{
    FullSnapshot, GridDelta, LaunchRequest, ProcessExit, RemoteCodec, RemoteMessage,
    RemoteProcessState,
};
use homie_proto::{NeedsInputDetail, SessionStatus};
use homie_terminal_state::GridMirror;

use crate::detect::ManifestEngine;
use crate::holder::{
    HolderClient, HolderExitMarker, HolderExitStatus, HolderLaunchSpec, HolderLauncher,
    HolderPaths, HolderStat,
};
use crate::log::OutputLog;
use crate::pty::{Exit, Pty, PtySpec};
use crate::remote::binding::{RemoteBinding, RemoteBindingStore};
use crate::remote::client::RemoteSessionClient;
use crate::remote::manager::{InstalledHelper, RemoteManager};
use crate::screen::HeadlessScreen;
use crate::status::{Authority, ClaudeHook, ReducerOutcome, StatusReducer, StatusSignal};

/// How often the pump ticks when the child is quiet, so debounce timers still
/// advance and staleness is noticed.
const TICK_INTERVAL: Duration = Duration::from_millis(100);

/// Quiet-tick interval for a session that is neither attached, recently
/// touched, nor Working. Reducer ticks are no-ops outside Working, so with 30
/// idle background sessions this is the difference between ~300 wakeups plus
/// ~900 log syscalls a second and ~30.
const IDLE_TICK_INTERVAL: Duration = Duration::from_secs(1);

/// How long an attach poll or input write keeps a session on the fast tick.
const HOT_WINDOW_SECS: u64 = 30;

/// Maximum raw log tail replayed when starting or adopting a held session.
/// The same hard startup-work bound the reference implementation enforced.
const REPLAY_BUDGET: usize = 256 << 10;
const MAX_REPLAY_BUDGET: usize = 32 << 20;

/// The normal startup bound is intentionally small, but an old checkpoint
/// format can require a one-time raw-log migration to rebuild scrollback.
/// Operators can raise the bound for that restart without changing the
/// steady-state cost; malformed values fall back to the default.
fn replay_budget() -> usize {
    std::env::var("HOMIE_REPLAY_BUDGET_BYTES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map_or(REPLAY_BUDGET, |value| {
            value.clamp(REPLAY_BUDGET, MAX_REPLAY_BUDGET)
        })
}

/// Quiet time after the last output before a screen checkpoint is written,
/// the reference implementation's `checkpointSettleDelay`. Bursts coalesce into one
/// write; an idle screen is checkpointed within about a second.
const CHECKPOINT_SETTLE: Duration = Duration::from_secs(1);

/// How long a deferred spawn waits for the first client size before
/// launching at the estimated size anyway — an MCP-spawned agent may never
/// get a view. The reference implementation's 400ms fallback window.
const LAUNCH_FALLBACK: Duration = Duration::from_millis(400);

/// While unlaunched, each client resize pushes the exec back this far, so
/// the agent starts at the SETTLED viewport rather than a transient
/// first-layout size — otherwise its one-shot banner bakes at the wrong
/// width. The reference implementation's `scheduleDebouncedLaunch` delay.
const LAUNCH_DEBOUNCE: Duration = Duration::from_millis(120);

/// Quiet time between holder liveness probes: a holder that died markerless
/// (SIGKILL, machine issues) must not leave a forever-live session behind.
/// Elapsed-based so the probe cadence is the same on fast and idle ticks.
const LIVENESS_INTERVAL: Duration = Duration::from_secs(2);

/// What a session looks like from the outside.
#[derive(Clone, Debug)]
pub struct SessionView {
    pub id: String,
    pub status: SessionStatus,
    pub needs_input: Option<NeedsInputDetail>,
    pub title: Option<String>,
    pub title_source: Option<homie_proto::TitleSource>,
    pub tail_offset: u64,
    pub exited: bool,
}

/// Small input-side composer mirror used only until the first real prompt is
/// submitted. It avoids parsing an Agent's rendered screen or reading remote
/// transcript files, and disappears from the hot path after the title exists.
#[derive(Default)]
struct PromptInputState {
    draft: String,
}

/// The state the pump thread and the outside world share.
pub(crate) struct Shared {
    id: String,
    status: Mutex<SessionStatus>,
    needs_input: Mutex<Option<NeedsInputDetail>>,
    title: Mutex<Option<String>>,
    prompt_title: Mutex<Option<String>>,
    prompt_input: Mutex<PromptInputState>,
    log: Mutex<OutputLog>,
    screen: Mutex<HeadlessScreen>,
    reducer: Mutex<StatusReducer>,
    /// How the child ended, once known (from `wait` or the exit marker).
    exit: Mutex<Option<Exit>>,
    exited: AtomicBool,
    stop: AtomicBool,
    /// Bumped whenever status, needs-input, or title actually change. The
    /// registry watcher compares this instead of cloning and JSON-serializing
    /// every record on every poll.
    state_version: AtomicU64,
    /// Seconds since UNIX_EPOCH of the last attach-pump poll or input write.
    /// Keeps interactive sessions on the fast quiet-tick.
    last_hot: AtomicU64,
    /// URLs scanned off the visible screen (PRs, previews, links).
    artifacts: Mutex<Vec<homie_proto::SessionArtifact>>,
    /// True while the child tree is SIGSTOPped. Writing into a stopped
    /// tree's PTY wedges (nobody drains the slave; the buffer fills), so
    /// input is queued instead and flushed right after SIGCONT.
    hibernated: AtomicBool,
    /// Input received while hibernated, in arrival order.
    queued_input: Mutex<Vec<u8>>,
    /// The child's pid, for tree enumeration by the resource governor.
    child_pid: std::sync::atomic::AtomicI32,
    /// The remote Holder's grid is display-authoritative. Raw output still
    /// feeds `screen` for local status reduction and artifact detection.
    remote_grid: Mutex<Option<RemoteGridState>>,
    remote_output_offset: AtomicU64,
    grid_wake: GridWake,
}

struct RemoteGridState {
    mirror: GridMirror,
    revision: u64,
    pending: Option<homie_proto::grid::GridUpdate>,
}

impl Shared {
    fn bump_state_version(&self) {
        self.state_version.fetch_add(1, Ordering::SeqCst);
    }

    fn note_hot(&self) {
        self.last_hot.store(unix_secs(), Ordering::Relaxed);
    }

    /// Fast quiet-tick while the session is attached/touched or Working;
    /// everything else can wait a second.
    fn wants_fast_tick(&self) -> bool {
        if unix_secs().saturating_sub(self.last_hot.load(Ordering::Relaxed)) <= HOT_WINDOW_SECS {
            return true;
        }
        matches!(
            *self.status.lock().expect("status"),
            SessionStatus::Working | SessionStatus::Starting
        )
    }

    fn quiet_tick(&self) -> Duration {
        if self.wants_fast_tick() {
            TICK_INTERVAL
        } else {
            IDLE_TICK_INTERVAL
        }
    }
}

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// What the grid pump compares between ticks to decide whether anything
/// observable changed. Default is "never seen anything".
#[derive(Clone, Copy, Debug, Default, PartialEq)]

pub struct GridSignature {
    pub content_seq: u64,
    pub size: (usize, usize),
    pub cursor: (u16, u16, bool),
    pub alt_screen: bool,
    pub mouse_reporting: bool,
}

/// Event source for the attachment writer. PTY readers advance it only after
/// the authoritative grid changes, so a quiet attached terminal has no
/// frame-rate polling cost.
#[derive(Clone)]
pub(crate) struct GridWake {
    inner: Arc<GridWakeInner>,
}

struct GridWakeInner {
    state: Mutex<GridWakeState>,
    changed: Condvar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GridWakeEvent {
    pub generation: u64,
    pub interactive: bool,
}

struct GridWakeState {
    generation: u64,
    interactive_budget: u8,
}

const INTERACTIVE_GRID_BUDGET: u8 = 2;

pub(crate) struct AttachmentSeed {
    pub grid: homie_proto::grid::GridUpdate,
    pub modes: (bool, bool),
    pub signature: GridSignature,
    pub wake: GridWake,
    pub wake_generation: u64,
}

/// Who owns the PTY.
enum Transport {
    /// This process does; dropping the session kills the child.
    Direct(Arc<Mutex<Pty>>),
    /// A holder process does; this session is a socket client and a log
    /// tail, and the child outlives it.
    Held(HolderClient),
    /// A remote Holder owns the PTY. Dropping this transport closes only the
    /// SSH Bridge; explicit termination is the only path that kills Agent.
    Remote(Arc<RemoteSessionClient>),
}

pub struct Session {
    shared: Arc<Shared>,
    transport: Transport,
    pump: Option<JoinHandle<()>>,
    manifest_id: String,
    /// Present while the exec is deferred to the first settled client size.
    deferred: Option<Arc<DeferredLaunch>>,
}

/// Deferred-launch state: the agent is not exec'd until the attaching client
/// reports its real terminal size, so a TUI's one-shot banner renders at the
/// exact width (no post-spawn reflow). Ported from the reference implementation's
/// `scheduleDebouncedLaunch`.
struct DeferredLaunch {
    state: Mutex<DeferredState>,
    cond: std::sync::Condvar,
}

/// What [`DeferredLaunch::finish_launch`] hands back: the input queued while
/// unlaunched, and a size proposed after the launch size was taken.
struct LaunchHandoff {
    queued_input: Vec<u8>,
    late_size: Option<(u16, u16)>,
}

struct DeferredState {
    /// The latest client-proposed size, if any arrived before launch.
    pending: Option<(u16, u16)>,
    /// When the launch fires: pushed back by each new size proposal.
    deadline: Instant,
    /// Input typed before the child exists, flushed right after exec.
    queued_input: Vec<u8>,
    launched: bool,
    cancelled: bool,
}

/// Where holders live and what binary hosts them. Present on a spec, it makes
/// the spawn holder-backed.
#[derive(Clone, Debug)]
pub struct HolderConfig {
    pub holders_dir: PathBuf,
    pub executable: PathBuf,
}

/// Everything needed to launch one structured command through an installed
/// remote Helper. Secrets stay in Engine memory and are never written into a
/// public [`homie_proto::SessionRecord`].
#[derive(Clone)]
pub struct RemoteSessionSpec {
    pub manager: Arc<RemoteManager>,
    pub helper: InstalledHelper,
    pub launch: LaunchRequest,
    pub host_id: String,
    pub binding_store: RemoteBindingStore,
}

#[derive(Clone)]
pub struct RemoteAdoptSpec {
    pub manager: Arc<RemoteManager>,
    pub helper: InstalledHelper,
    pub token: homie_proto::remote_pty::SessionToken,
    pub incarnation: String,
    pub binding_store: RemoteBindingStore,
    pub output_offset: u64,
}

struct RemoteLaunchCleanup {
    manager: Arc<RemoteManager>,
    helper: InstalledHelper,
    binding_store: RemoteBindingStore,
    selector: homie_proto::remote_pty::SessionSelector,
    armed: bool,
}

/// How to start a session.
pub struct SessionSpec {
    pub id: String,
    pub pty: PtySpec,
    /// Which manifest drives detection ("claude-code", "codex", …).
    pub manifest_id: String,
    pub authority: Authority,
    pub logs_dir: PathBuf,
    /// `Some` spawns through a holder so the child survives this process.
    pub holder: Option<HolderConfig>,
    /// Present for a remote Holder-backed session. It is mutually exclusive
    /// with the local `holder` transport.
    pub remote: Option<RemoteSessionSpec>,
    /// Defer the exec until the first client size settles (holder spawns
    /// only), so the agent's banner renders at the real viewport width.
    pub defer_launch: bool,
}

/// Everything `session_spawn` / `session_spawn_remote` assemble before the
/// handler registers the project, spawns the session, publishes the update,
/// and types the initial prompt. Bundling it here keeps the handler a thin
/// adapter while leaving the registry mutation (and therefore the mutex
/// discipline) in the transport layer.
pub struct SpawnPlan {
    pub spec: SessionSpec,
    pub record: homie_proto::SessionRecord,
    pub prompt: Option<String>,
    /// The project root to register with `ensure_session_project`: the
    /// *caller's* cwd for a local spawn (a linked worktree is an execution
    /// cwd, not a new first-level project) and the captured remote cwd
    /// otherwise.
    pub project_root: String,
    /// Remote host id, `Some` only for remote spawns.
    pub host_id: Option<String>,
}

/// Convenience for tests and callers that just want the shipped rules.
pub fn load_engine(manifests: &Path) -> std::io::Result<(Arc<ManifestEngine>, Vec<String>)> {
    let (engine, failed) = ManifestEngine::load_dir(manifests)?;
    Ok((Arc::new(engine), failed))
}

/// The reducer authority for an agent, as its manifest declares it.
///
/// This used to special-case "claude-code" in code. It is data: each manifest
/// carries `agent.statusAuthority`, so a new agent gets the right behavior by
/// existing as a file.
pub fn authority_for(manifest_id: &str, engine: &ManifestEngine) -> Authority {
    engine
        .manifest(manifest_id)
        .and_then(|manifest| manifest.agent.as_ref())
        .map_or(Authority::ProcessOnly, |agent| agent.authority())
}

impl Session {
    pub fn id(&self) -> &str {
        &self.shared.id
    }

    pub fn manifest_id(&self) -> &str {
        &self.manifest_id
    }
}

mod launch;
mod lifecycle;
mod pty;
mod pump;
mod remote;
mod screen;
mod status;

pub(crate) use launch::{
    LaunchContext, remote_resume_spec, remote_spawn_spec, resolve_host, resume_spec, spawn_spec,
};
pub(crate) use lifecycle::holder_io_error;
pub(crate) use pump::{evaluate_if_screen_changed, pump_held, pump_loop, scan_artifacts_if_due};
pub(crate) use remote::pump_remote;
pub(crate) use status::apply;
