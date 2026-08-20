//! Headless application state and the thin asynchronous daemon adapter.

mod prefs;
mod projection;
mod residency;

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use homie_client::{ClientError, ConnectionState, DaemonClient, EventEnvelope};
use homie_proto::paths::HomiePaths;
use homie_proto::remote_pty::DirectoryListResult;
use homie_proto::{
    AgentDescriptor, AgentKind, AgentReadinessResult, AttentionLevel, DateMillis, EventName,
    ExitReason, GovernorConfigureParams, HostEntry, HostsConfig, Project, ProjectId, Resumability,
    SessionId, SessionListResult, SessionRecord, SessionSpawnParams, SessionStatus,
};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

use crate::notifications::{
    SendTextCommand, StatusTransition, migration_transition, prefs_sync_transition,
    reach_failure_transition, transitions_for_update,
};
use crate::switcher::{
    OverviewArrow, OverviewFilter, OverviewMode, OverviewOutcome, SessionOverviewState,
    SessionSwitcherState, SwitcherKey, SwitcherOutcome,
};

pub use prefs::{DefaultAgent, InspectorTab, Prefs, WindowMode, WindowPlacement};
pub use projection::{SidebarProject, SidebarProjection, SidebarRow};
pub use residency::{ResidencyUpdate, TerminalResidency};

pub const AUXILIARY_TERMINAL_TITLE: &str = "Terminal";

pub(crate) fn is_auxiliary_terminal(session: &SessionRecord) -> bool {
    // `parent` was previously written to the wire but had no UI semantics.
    // Shell children now belong to their parent's workbench. Do not key this
    // off the title: shells can update their title through terminal escape
    // sequences while they are running.
    session.kind == AgentKind::SHELL && session.parent.is_some()
}

// Session titles, badges, and sidebar metadata change at human speed. Publishing
// daemon bursts at display refresh rate rebuilt the whole sidebar 60 times/sec
// while several agents were working; terminal grids retain their independent
// 20fps direct path.
const UI_PUBLISH_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StoreEventChange {
    None,
    Resources,
    Model,
}

impl StoreEventChange {
    fn merge(self, other: Self) -> Self {
        if self == Self::Model || other == Self::Model {
            Self::Model
        } else if self == Self::Resources || other == Self::Resources {
            Self::Resources
        } else {
            Self::None
        }
    }
}

fn event_publication_policy(change: StoreEventChange, active: bool) -> (bool, bool) {
    let publish_snapshot = active || change == StoreEventChange::Model;
    let notify_views = active;
    (publish_snapshot, notify_views)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DaemonState {
    Connecting,
    Connected,
    Unreachable(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum StoreEffect {
    /// Repaint subscribers after a purely local navigation change.
    UiChanged,
    /// Push one fresh snapshot to watch subscribers. The menu-bar panel skips
    /// rebuilds while hidden, so opening it asks for a current snapshot.
    PublishSnapshot,
    MarkSeen(SessionId),
    Remove(SessionId),
    Resume {
        id: SessionId,
        automatic: bool,
    },
    Archive(SessionId),
    Unarchive(SessionId),
    Rename {
        id: SessionId,
        title: String,
    },
    RefreshAgentCatalog,
    Spawn(SessionSpawnParams),
    /// A shell owned by a workbench pane. Unlike a top-level spawn, its
    /// response must not replace the selected sidebar session.
    SpawnAuxiliary(SessionSpawnParams),
    /// `session.migrate` — move a Claude session between local and a host.
    Migrate {
        id: SessionId,
        target_host: Option<String>,
    },
    /// `host.sync_prefs` — push agent preferences to a remote host.
    SyncPrefs {
        host: String,
        host_name: String,
    },
    /// `host.locate_repo` — resolve the reference session's repo on a host;
    /// the answer lands back in the store as a `RepoTarget`.
    LocateRepo {
        key: String,
        host: Option<String>,
        session_id: SessionId,
    },
    /// One bounded level for the New Agent folder picker. Results are keyed by
    /// generation so a slow host cannot overwrite a newer navigation click.
    ListDirectories {
        request_id: u64,
        host: Option<String>,
        path: String,
    },
    ReopenLast,
    SetActive(bool),
    ConfigureGovernor(GovernorConfigureParams),
    /// T11 consumes this by closing and dropping the corresponding attachment.
    DetachAttachment(SessionId),
    /// T15 consumes this without involving daemon lifecycle operations.
    StatusTransition(StatusTransition),
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoreSnapshot {
    /// Shared, not owned: a snapshot publish happens up to every 50ms while
    /// events flow, and its consumers only read.
    pub sessions: Vec<Arc<SessionRecord>>,
    pub projects: Vec<Project>,
    pub selected_session_id: Option<SessionId>,
    pub global_attention: AttentionLevel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingClose {
    pub ids: Vec<SessionId>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClickModifiers {
    pub command: bool,
    pub shift: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorktreeSpawn {
    pub create: bool,
    pub branch: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpawnOptions {
    pub cwd: Option<String>,
    pub worktree: Option<WorktreeSpawn>,
    pub title: Option<String>,
    pub initial_prompt: Option<String>,
    pub parent: Option<SessionId>,
    pub initial_cols: Option<i64>,
    pub initial_rows: Option<i64>,
    /// `HostEntry.id` from hosts.json — spawn on that remote host. When set,
    /// `cwd` (or the host's `defaultCwd`) is a REMOTE path, and worktree
    /// options are ignored (a local-git feature).
    pub host: Option<String>,
    /// Repo-preserving spawn: the daemon opens the session in the checkout of
    /// the SAME repository as this session on the target host (matched by
    /// origin URL), falling back to `cwd` / defaultCwd when not cloned there.
    pub same_repo_as: Option<SessionId>,
}

/// Async resolution state for "the active session's repo on host X" — drives
/// the new-agent popover's target path + fallback subtitle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepoTarget {
    Pending,
    /// Absolute checkout path on the target host.
    Resolved(String),
    /// The repo has an origin but no clone on the target host — spawns fall
    /// back to the host's default directory (and the popover says so).
    NotCloned,
    /// The reference session isn't in a git repo with an origin (or the
    /// lookup failed): plain default-directory behavior, no subtitle.
    NoOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirectoryListingState {
    Loading,
    Ready(DirectoryListResult),
    Error(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryListing {
    pub request_id: u64,
    pub host: Option<String>,
    pub requested_path: String,
    pub state: DirectoryListingState,
}

fn repo_target_key(host: Option<&str>) -> String {
    host.unwrap_or("local").to_owned()
}

/// Pure application model. Side effects are emitted onto a channel for the daemon adapter.
pub struct SessionStore {
    daemon_state: DaemonState,
    sessions: HashMap<SessionId, Arc<SessionRecord>>,
    projects: HashMap<ProjectId, Project>,
    selected_session_id: Option<SessionId>,
    sidebar_selection: HashSet<SessionId>,
    pending_close: Option<PendingClose>,
    /// Sessions with a `session.remove` in flight. The daemon terminates the
    /// process tree before it publishes `session.removed`, which takes upwards
    /// of half a second; rows hide the moment the close is dispatched so the
    /// click reads as done instead of inviting a second one.
    closing: HashSet<SessionId>,
    auto_resuming: HashSet<SessionId>,
    /// Sessions with a `session.migrate` in flight — rows render busy and
    /// migration actions are suppressed until the daemon answers.
    migrating: HashSet<SessionId>,
    /// Host ids with a `host.sync_prefs` in flight.
    syncing_prefs: HashSet<String>,
    /// Popover repo resolution: host key → state (see `RepoTarget`).
    repo_targets: HashMap<String, RepoTarget>,
    /// The session whose repo the popover preserves (selected at open time).
    repo_target_session: Option<SessionId>,
    directory_request_seq: u64,
    directory_listing: Option<DirectoryListing>,
    prefs: Prefs,
    terminal_residency: TerminalResidency,
    app_is_active: bool,
    last_action_error: Option<String>,
    sidebar_selection_anchor: Option<SessionId>,
    mru_order: Vec<SessionId>,
    switcher: SessionSwitcherState,
    overview: SessionOverviewState,
    auto_resume_attempted: HashSet<SessionId>,
    revision: u64,
    cached_projection: Option<(u64, Arc<SidebarProjection>)>,
    prefs_path: Option<PathBuf>,
    /// Remote host catalog from hosts.json. Empty when the file is absent or
    /// invalid (pickers show Local only). Reloaded on picker open.
    hosts: Vec<HostEntry>,
    /// Agent catalog from the daemon's `agent.readiness`: what agents exist,
    /// whether their CLI is installed, and each one's manifest descriptor.
    /// Empty until the first successful connect, and empty forever against a
    /// daemon too old to send descriptors — every reader must have a fallback.
    agents: AgentReadinessResult,
    effects: mpsc::UnboundedSender<StoreEffect>,
}

mod events;
mod hosts;
mod lifecycle;
mod navigation;
mod ordering;
mod runtime;
mod sessions;
mod switcher;

pub use runtime::StoreRuntime;

fn attention_rank(attention: &AttentionLevel) -> u8 {
    match attention {
        AttentionLevel::None | AttentionLevel::Unknown => 0,
        AttentionLevel::IdleSeen => 1,
        AttentionLevel::Working => 2,
        AttentionLevel::DoneUnseen => 3,
        AttentionLevel::NeedsInput => 4,
    }
}

/// Drops ids that no longer exist and appends the ones the order has not seen,
/// in `arriving` order. Returns `None` when the order already agreed, so a
/// caller can skip a prefs write on the overwhelmingly common no-op.
fn reconcile_order<'a, T>(
    order: &[T],
    live: &HashSet<&T>,
    arriving: impl Iterator<Item = &'a T>,
) -> Option<Vec<T>>
where
    T: Clone + Eq + std::hash::Hash + 'a,
{
    let mut next: Vec<T> = order
        .iter()
        .filter(|id| live.contains(id))
        .cloned()
        .collect();
    let known: HashSet<&T> = order.iter().collect();
    let appended: Vec<T> = arriving
        .filter(|id| !known.contains(*id))
        .cloned()
        .collect();
    if next.len() == order.len() && appended.is_empty() {
        return None;
    }
    next.extend(appended);
    Some(next)
}

/// Prunes ids that no longer exist, or `None` when there was nothing to prune.
fn retain_live<T: Clone + Eq + std::hash::Hash>(
    values: &[T],
    live: &HashSet<&T>,
) -> Option<Vec<T>> {
    let kept: Vec<T> = values
        .iter()
        .filter(|id| live.contains(id))
        .cloned()
        .collect();
    (kept.len() != values.len()).then_some(kept)
}

fn toggle_vec_member<T: PartialEq>(values: &mut Vec<T>, value: T) {
    if let Some(index) = values.iter().position(|candidate| candidate == &value) {
        values.remove(index);
    } else {
        values.push(value);
    }
}

fn now_millis() -> DateMillis {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |duration| duration.as_secs_f64() * 1000.0);
    DateMillis(millis)
}

pub fn prefs_path_in_home(home: &Path) -> PathBuf {
    Prefs::path_in_home(home)
}

#[cfg(test)]
mod tests;
