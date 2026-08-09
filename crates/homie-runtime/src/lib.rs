pub mod capabilities;
pub mod connection;
#[cfg(unix)]
pub mod daemon;
pub mod dispatcher;
pub mod event_stream;
pub mod history;
pub mod holder;
pub mod long_running;
pub mod pr_monitor;
#[cfg(unix)]
pub mod process_tree;
pub mod pty;
pub mod runtime_actor;
pub mod screen;
pub mod server;
pub mod streams;
pub mod terminal_stream;
pub mod writer;

pub use connection::{
    ActiveStream, ActiveStreamFuture, ControlFuture, ControlHandler, StreamError, StreamFuture,
    StreamHandler,
};
#[cfg(unix)]
pub use daemon::{DaemonError, DaemonLease, daemon_instance_id, executable_sha256};
pub use dispatcher::RuntimeDispatcher;
pub use event_stream::{EventBounds, EventStore, RuntimeEventWaitHandler};
pub use holder::{HolderPaths, HolderRequest, HolderResponse};
pub use homie_proto::model::RuntimeEvent;
#[cfg(unix)]
pub use process_tree::{ProcessSample, kill_process_tree, process_tree};
pub use pty::{Exit, Pty, PtySpec, PtyStream};
pub use screen::{HeadlessScreen, ScreenSnapshot};
pub use server::{RuntimeServer, ServerConfig, ServerIdentity};
pub use streams::RuntimeStreamHandler;
pub use terminal_stream::{
    TerminalBackend, TerminalSourceDescriptor, TerminalSourceStats, TerminalStreamError,
};

use homie_agents::{
    Authority, ManifestState, ReducerTiming, ScreenObservation, StatusReducer, StatusSignal,
};
use homie_proto::{
    EventName, NeedsInputDetail, NeedsInputKind, SessionDiffBase, SessionReadDiffResult,
    SessionStatus, WorktreeInfo,
};
use homie_storage::{CreateSession, SessionSummary, Storage, StorageConfig, StorageError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::io::{Read, Seek};
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub data_dir: PathBuf,
}

pub struct RuntimeSupervisor {
    data_dir: PathBuf,
    storage: Storage,
    live_sessions: Mutex<HashMap<String, LiveSession>>,
    holder_binary: Option<PathBuf>,
    event_store: Arc<EventStore>,
}

struct LiveSession {
    holder: HolderPaths,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeOverviewEntry {
    pub project_root: String,
    pub path: String,
    pub branch: Option<String>,
    pub session_id: Option<String>,
    pub session_status: Option<String>,
    pub dirty: bool,
    pub merged: bool,
    pub age_days: i64,
    pub stale_suggestion: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeCleanupRequest {
    pub repo_path: String,
    pub worktree_path: String,
    pub force: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeCreateRequest {
    pub repo_path: PathBuf,
    pub branch: Option<String>,
    pub base: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeRemoveRequest {
    pub repo_path: PathBuf,
    pub worktree_path: PathBuf,
    pub force: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeSheet {
    entries: Vec<WorktreeOverviewEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactKind {
    PullRequest,
    Preview,
    Link,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionArtifact {
    pub kind: ArtifactKind,
    pub url: String,
    pub label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListeningPort {
    pub port: u16,
    pub url: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactScan {
    pub artifacts: Vec<SessionArtifact>,
    pub ports: Vec<ListeningPort>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionStatusReport {
    pub status: SessionStatus,
    pub needs_input: Option<NeedsInputDetail>,
    pub turn_completed: bool,
    pub screen_lines: Vec<String>,
    pub screen_observation: Option<RuntimeScreenObservation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionStatusPreparation {
    persisted_status: SessionStatus,
    holder_status: Option<String>,
    holder_exited: bool,
    persisted_needs_input: Option<NeedsInputDetail>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionSnapshot {
    pub session: SessionSummary,
    pub status: SessionStatusReport,
    pub output_offset: u64,
    pub output: Vec<u8>,
    pub holder: Option<HolderSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HolderSnapshot {
    pub pid: Option<u32>,
    pub status: Option<String>,
    pub tree_size: Option<usize>,
    pub cols: Option<u16>,
    pub rows: Option<u16>,
    pub log_offset: Option<u64>,
    pub epoch_offset: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenCheckpoint {
    pub session_id: String,
    pub output_offset: u64,
    pub content_seq: u64,
    pub lines: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeScreenObservation {
    pub state: ManifestState,
    pub matched_rule_id: String,
    pub content_seq: u64,
}

const MAX_DIFF_BYTES: usize = 16 * 1024 * 1024;
const MAX_UNTRACKED_FILES: usize = 200;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiffRowKind {
    File,
    Hunk,
    Context,
    Addition,
    Deletion,
    Meta,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffRow {
    pub kind: DiffRowKind,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub text: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiffSnapshot {
    pub repo_root: PathBuf,
    pub base_ref: Option<String>,
    pub rows: Vec<DiffRow>,
    pub files: usize,
    pub additions: usize,
    pub deletions: usize,
    pub max_text_columns: usize,
    pub truncated: bool,
    pub patch: Vec<u8>,
}

pub fn scan_artifacts(text: &str) -> ArtifactScan {
    let mut artifacts = Vec::new();
    let mut ports = Vec::new();
    for token in text.split_whitespace() {
        let cleaned = token.trim_matches(|ch: char| matches!(ch, ',' | ')' | ']' | '"' | '\''));
        if cleaned.starts_with("http://") || cleaned.starts_with("https://") {
            if let Some(port) = localhost_port(cleaned) {
                ports.push(ListeningPort {
                    port,
                    url: cleaned.to_string(),
                });
            }
            let kind = if cleaned.contains("/pull/") || cleaned.contains("/pr/") {
                ArtifactKind::PullRequest
            } else if localhost_port(cleaned).is_some() {
                ArtifactKind::Preview
            } else {
                ArtifactKind::Link
            };
            let label = match kind {
                ArtifactKind::PullRequest => "PR",
                ArtifactKind::Preview => "Preview",
                ArtifactKind::Link => "Link",
            }
            .to_string();
            artifacts.push(SessionArtifact {
                kind,
                url: cleaned.to_string(),
                label,
            });
        }
    }
    artifacts.sort_by(|left, right| left.url.cmp(&right.url));
    artifacts.dedup_by(|left, right| left.url == right.url && left.kind == right.kind);
    ports.sort_by_key(|port| port.port);
    ports.dedup_by_key(|port| port.port);
    ArtifactScan { artifacts, ports }
}

fn localhost_port(url: &str) -> Option<u16> {
    let rest = url
        .strip_prefix("http://localhost:")
        .or_else(|| url.strip_prefix("http://127.0.0.1:"))?;
    let digits = rest
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    digits.parse().ok()
}

impl WorktreeSheet {
    #[must_use]
    pub fn new(mut entries: Vec<WorktreeOverviewEntry>) -> Self {
        entries.sort_by(|left, right| {
            left.project_root
                .cmp(&right.project_root)
                .then_with(|| left.branch.cmp(&right.branch))
                .then_with(|| left.path.cmp(&right.path))
        });
        Self { entries }
    }

    #[must_use]
    pub fn entries(&self) -> &[WorktreeOverviewEntry] {
        &self.entries
    }

    #[must_use]
    pub fn request_cleanup(&mut self, path: &str) -> Option<WorktreeCleanupRequest> {
        let entry = self.entries.iter().find(|entry| entry.path == path)?;
        if !can_cleanup(entry) {
            return None;
        }
        Some(WorktreeCleanupRequest {
            repo_path: entry.project_root.clone(),
            worktree_path: entry.path.clone(),
            force: false,
        })
    }
}

fn can_cleanup(entry: &WorktreeOverviewEntry) -> bool {
    entry.stale_suggestion
        && !entry.dirty
        && entry.merged
        && !matches!(entry.branch.as_deref(), Some("main" | "master"))
}

pub fn load_git_diff(
    cwd: &Path,
    comparison: SessionDiffBase,
) -> Result<DiffSnapshot, RuntimeError> {
    let root_output = run_git_output(cwd, ["rev-parse", "--show-toplevel"])?;
    if !root_output.status.success() {
        return Err(std::io::Error::other("not inside a git repository").into());
    }
    let repo_root = PathBuf::from(String::from_utf8_lossy(&root_output.stdout).trim());
    if repo_root.as_os_str().is_empty() {
        return Err(std::io::Error::other("not inside a git repository").into());
    }

    let has_head = run_git_output(&repo_root, ["rev-parse", "--verify", "HEAD"])
        .is_ok_and(|output| output.status.success());
    let mut patch = Vec::new();
    let mut base_ref = None;
    if has_head {
        let (label, commit) = resolve_diff_base(&repo_root, comparison)?;
        base_ref = Some(label);
        append_diff_output(
            &mut patch,
            run_git_output(
                &repo_root,
                [
                    "diff",
                    "--no-ext-diff",
                    "--no-color",
                    "--unified=3",
                    commit.as_str(),
                    "--",
                ],
            )?,
        )?;
    } else {
        append_diff_output(
            &mut patch,
            run_git_output(
                &repo_root,
                [
                    "diff",
                    "--no-ext-diff",
                    "--no-color",
                    "--unified=3",
                    "--cached",
                    "--",
                ],
            )?,
        )?;
        append_diff_output(
            &mut patch,
            run_git_output(
                &repo_root,
                ["diff", "--no-ext-diff", "--no-color", "--unified=3", "--"],
            )?,
        )?;
    }

    let untracked = run_git_output(
        &repo_root,
        ["ls-files", "--others", "--exclude-standard", "-z"],
    )?;
    if !untracked.status.success() {
        return Err(git_output_error(&untracked).into());
    }
    for path in untracked
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .take(MAX_UNTRACKED_FILES)
    {
        if patch.len() >= MAX_DIFF_BYTES {
            break;
        }
        #[cfg(unix)]
        let path = OsString::from_vec(path.to_vec());
        #[cfg(not(unix))]
        let path = OsString::from(String::from_utf8_lossy(path).into_owned());
        let output = Command::new("/usr/bin/git")
            .current_dir(&repo_root)
            .args([
                "diff",
                "--no-index",
                "--no-color",
                "--unified=3",
                "--",
                "/dev/null",
            ])
            .arg(path)
            .stdin(Stdio::null())
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()?;
        if !output.status.success() && output.status.code() != Some(1) {
            return Err(git_output_error(&output).into());
        }
        append_diff_bytes(&mut patch, &output.stdout);
    }

    let truncated = patch.len() > MAX_DIFF_BYTES;
    patch.truncate(MAX_DIFF_BYTES);
    let mut snapshot = parse_unified_diff(&String::from_utf8_lossy(&patch));
    snapshot.repo_root = repo_root;
    snapshot.base_ref = base_ref;
    snapshot.truncated = truncated;
    snapshot.patch = patch;
    if truncated {
        snapshot.rows.push(DiffRow {
            kind: DiffRowKind::Meta,
            old_line: None,
            new_line: None,
            text: "Diff truncated at 16 MB".to_string(),
        });
    }
    Ok(snapshot)
}

pub fn read_git_diff(
    cwd: &Path,
    comparison: SessionDiffBase,
) -> Result<SessionReadDiffResult, RuntimeError> {
    let snapshot = load_git_diff(cwd, comparison)?;
    Ok(SessionReadDiffResult {
        patch: snapshot.patch,
        repo_root: snapshot.repo_root.display().to_string(),
        truncated: snapshot.truncated,
        base_ref: snapshot.base_ref,
    })
}

pub fn parse_unified_diff(patch: &str) -> DiffSnapshot {
    let mut snapshot = DiffSnapshot::default();
    let mut old_line = None;
    let mut new_line = None;

    for line in patch.lines() {
        let row = if let Some(header) = line.strip_prefix("diff --git ") {
            snapshot.files += 1;
            old_line = None;
            new_line = None;
            DiffRow {
                kind: DiffRowKind::File,
                old_line: None,
                new_line: None,
                text: diff_path(header),
            }
        } else if line.starts_with("--- ") || line.starts_with("+++ ") {
            continue;
        } else if line.starts_with("@@") {
            let (old, new) = parse_hunk_start(line);
            old_line = old;
            new_line = new;
            DiffRow {
                kind: DiffRowKind::Hunk,
                old_line: None,
                new_line: None,
                text: line.to_string(),
            }
        } else if let Some(text) = line.strip_prefix('+') {
            let current = new_line;
            new_line = new_line.map(|line: u32| line.saturating_add(1));
            snapshot.additions += 1;
            DiffRow {
                kind: DiffRowKind::Addition,
                old_line: None,
                new_line: current,
                text: text.to_string(),
            }
        } else if let Some(text) = line.strip_prefix('-') {
            let current = old_line;
            old_line = old_line.map(|line: u32| line.saturating_add(1));
            snapshot.deletions += 1;
            DiffRow {
                kind: DiffRowKind::Deletion,
                old_line: current,
                new_line: None,
                text: text.to_string(),
            }
        } else if let Some(text) = line.strip_prefix(' ') {
            let old = old_line;
            let new = new_line;
            old_line = old_line.map(|line: u32| line.saturating_add(1));
            new_line = new_line.map(|line: u32| line.saturating_add(1));
            DiffRow {
                kind: DiffRowKind::Context,
                old_line: old,
                new_line: new,
                text: text.to_string(),
            }
        } else {
            DiffRow {
                kind: DiffRowKind::Meta,
                old_line: None,
                new_line: None,
                text: line.to_string(),
            }
        };
        snapshot.max_text_columns = snapshot
            .max_text_columns
            .max(row.text.chars().count().min(500));
        snapshot.rows.push(row);
    }
    snapshot
}

fn resolve_diff_base(
    repo_root: &Path,
    comparison: SessionDiffBase,
) -> Result<(String, String), RuntimeError> {
    if comparison == SessionDiffBase::Head {
        return Ok(("HEAD".to_string(), "HEAD".to_string()));
    }
    if let Some(base_ref) = resolve_default_branch_ref(repo_root) {
        let output = run_git_output(repo_root, ["merge-base", base_ref.as_str(), "HEAD"])?;
        if output.status.success() {
            let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !commit.is_empty() {
                return Ok((base_ref, commit));
            }
        }
    }
    Ok(("HEAD".to_string(), "HEAD".to_string()))
}

fn resolve_default_branch_ref(repo_root: &Path) -> Option<String> {
    let origin_head = run_git_output(
        repo_root,
        [
            "symbolic-ref",
            "--quiet",
            "--short",
            "refs/remotes/origin/HEAD",
        ],
    )
    .ok()
    .filter(|output| output.status.success())
    .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
    .filter(|candidate| !candidate.is_empty());

    origin_head
        .into_iter()
        .chain(
            ["origin/main", "main", "origin/master", "master"]
                .into_iter()
                .map(str::to_string),
        )
        .find(|candidate| {
            let peeled = format!("{candidate}^{{commit}}");
            run_git_output(
                repo_root,
                ["rev-parse", "--verify", "--quiet", peeled.as_str()],
            )
            .is_ok_and(|output| output.status.success())
        })
}

fn run_git_output<I, S>(cwd: &Path, args: I) -> Result<std::process::Output, RuntimeError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Ok(Command::new("/usr/bin/git")
        .current_dir(cwd)
        .args(args)
        .stdin(Stdio::null())
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()?)
}

fn append_diff_output(
    patch: &mut Vec<u8>,
    output: std::process::Output,
) -> Result<(), RuntimeError> {
    if !output.status.success() {
        return Err(git_output_error(&output).into());
    }
    append_diff_bytes(patch, &output.stdout);
    Ok(())
}

fn append_diff_bytes(patch: &mut Vec<u8>, bytes: &[u8]) {
    let remaining = MAX_DIFF_BYTES.saturating_add(1).saturating_sub(patch.len());
    patch.extend_from_slice(&bytes[..bytes.len().min(remaining)]);
}

fn git_output_error(output: &std::process::Output) -> std::io::Error {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    std::io::Error::other(if stderr.is_empty() {
        format!("git exited with {}", output.status)
    } else {
        stderr
    })
}

fn diff_path(header: &str) -> String {
    header
        .rsplit_once(" b/")
        .map(|(_, path)| path)
        .or_else(|| header.rsplit_once(" \"b/").map(|(_, path)| path))
        .unwrap_or(header)
        .trim_matches('"')
        .to_string()
}

fn parse_hunk_start(header: &str) -> (Option<u32>, Option<u32>) {
    let mut fields = header.split_whitespace();
    let _ = fields.next();
    let old = fields.next().and_then(|field| range_start(field, '-'));
    let new = fields.next().and_then(|field| range_start(field, '+'));
    (old, new)
}

fn range_start(field: &str, prefix: char) -> Option<u32> {
    field.strip_prefix(prefix)?.split(',').next()?.parse().ok()
}

pub fn list_git_worktrees(repo: &Path) -> Result<Vec<WorktreeInfo>, RuntimeError> {
    let porcelain = run_git(["worktree", "list", "--porcelain"], repo)?;
    Ok(parse_worktree_porcelain(&porcelain))
}

pub(crate) fn list_git_worktrees_bounded(
    context: &crate::long_running::JobContext,
    repo: &Path,
) -> crate::runtime_actor::ServiceResult<Vec<WorktreeInfo>> {
    let output = run_git_bounded(
        context,
        repo,
        ["worktree", "list", "--porcelain"],
        4 * 1024 * 1024,
    )?;
    if !output.status.success() || output.stdout_truncated || output.stderr_truncated {
        return Err(crate::runtime_actor::ServiceError::Internal);
    }
    Ok(parse_worktree_porcelain(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

fn run_git_bounded<I, S>(
    context: &crate::long_running::JobContext,
    cwd: &Path,
    args: I,
    stdout_limit: usize,
) -> crate::runtime_actor::ServiceResult<crate::long_running::BoundedOutput>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    crate::long_running::run_bounded_command(
        context,
        crate::long_running::BoundedCommand::new("/usr/bin/git")
            .args(args)
            .current_dir(cwd)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", cwd.as_os_str())
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_OPTIONAL_LOCKS", "0")
            .stdout_limit(stdout_limit)
            .stderr_limit(256 * 1024),
    )
}

pub(crate) fn locate_repo_bounded(
    context: &crate::long_running::JobContext,
    request: homie_proto::HostLocateRepoParams,
    candidates: &[PathBuf],
) -> crate::runtime_actor::ServiceResult<homie_proto::HostLocateRepoResult> {
    for candidate in candidates {
        context.checkpoint()?;
        let root = run_git_bounded(
            context,
            candidate,
            ["rev-parse", "--show-toplevel"],
            64 * 1024,
        )?;
        if !root.status.success() || root.stdout_truncated || root.stderr_truncated {
            continue;
        }
        let root = PathBuf::from(String::from_utf8_lossy(&root.stdout).trim());
        if root.as_os_str().is_empty() {
            continue;
        }
        let origin = run_git_bounded(
            context,
            &root,
            ["config", "--get", "remote.origin.url"],
            64 * 1024,
        )?;
        let origin_url =
            if origin.status.success() && !origin.stdout_truncated && !origin.stderr_truncated {
                let value = String::from_utf8_lossy(&origin.stdout).trim().to_string();
                (!value.is_empty()).then_some(value)
            } else {
                None
            };
        if request.origin_url.as_deref().is_some_and(|wanted| {
            origin_url
                .as_deref()
                .is_none_or(|actual| normalize_git_origin(actual) != normalize_git_origin(wanted))
        }) {
            continue;
        }
        return Ok(homie_proto::HostLocateRepoResult {
            path: Some(canonical_display(&root)),
            origin_url,
        });
    }
    Ok(homie_proto::HostLocateRepoResult {
        path: None,
        origin_url: request.origin_url,
    })
}

fn normalize_git_origin(origin: &str) -> String {
    let trimmed = origin.trim().trim_end_matches('/').trim_end_matches(".git");
    if let Some((user_host, path)) = trimmed.split_once(':')
        && !user_host.contains("://")
        && let Some((_, host)) = user_host.rsplit_once('@')
    {
        return format!("{host}/{path}");
    }
    let without_scheme = trimmed
        .split_once("://")
        .map_or(trimmed, |(_, remainder)| remainder);
    without_scheme
        .split_once('@')
        .map_or(without_scheme, |(_, remainder)| remainder)
        .to_string()
}

pub(crate) fn worktree_overview_bounded(
    context: &crate::long_running::JobContext,
    projects: &[PathBuf],
    sessions: &[homie_proto::model::SessionSummary],
) -> crate::runtime_actor::ServiceResult<homie_proto::model::WorktreeOverviewResult> {
    let mut entries = Vec::new();
    for project in projects {
        context.checkpoint()?;
        let project_root = canonical_display(project);
        for worktree in list_git_worktrees_bounded(context, project)? {
            let path = PathBuf::from(&worktree.path);
            let status = run_git_bounded(
                context,
                &path,
                ["status", "--porcelain", "--untracked-files=normal"],
                4 * 1024 * 1024,
            )?;
            require_git_success(&status)?;
            let dirty = !status.stdout.is_empty();
            let merged = match worktree.branch.as_deref() {
                None | Some("main" | "master") => false,
                Some(branch) => {
                    let merge = run_git_bounded(
                        context,
                        project,
                        ["merge-base", "--is-ancestor", branch, "HEAD"],
                        64 * 1024,
                    )?;
                    match merge.status.code() {
                        Some(0) => true,
                        Some(1) => false,
                        _ => return Err(crate::runtime_actor::ServiceError::Internal),
                    }
                }
            };
            let age_days = path
                .metadata()
                .and_then(|metadata| metadata.modified())
                .ok()
                .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                .map_or(0, |age| {
                    i64::try_from(age.as_secs() / (24 * 60 * 60)).unwrap_or(i64::MAX)
                });
            let session = sessions
                .iter()
                .find(|session| same_path(Path::new(&session.workspace), &path));
            entries.push(homie_proto::model::WorktreeOverviewEntry {
                project_root: project_root.clone(),
                path: canonical_display(&path),
                branch: worktree.branch,
                session_id: session.map(|session| session.id.clone()),
                session_status: session.map(|session| session.status.clone()),
                dirty,
                merged,
                age_days,
                stale_suggestion: worktree.is_prunable || age_days >= 14,
            });
        }
    }
    entries.sort_by(|left, right| {
        left.project_root
            .cmp(&right.project_root)
            .then_with(|| left.branch.cmp(&right.branch))
            .then_with(|| left.path.cmp(&right.path))
    });
    Ok(homie_proto::model::WorktreeOverviewResult { entries })
}

pub(crate) fn read_git_diff_bounded(
    context: &crate::long_running::JobContext,
    cwd: &Path,
    comparison: SessionDiffBase,
) -> crate::runtime_actor::ServiceResult<SessionReadDiffResult> {
    let root_output = run_git_bounded(context, cwd, ["rev-parse", "--show-toplevel"], 64 * 1024)?;
    require_git_success(&root_output)?;
    let repo_root = PathBuf::from(String::from_utf8_lossy(&root_output.stdout).trim());
    if repo_root.as_os_str().is_empty() {
        return Err(crate::runtime_actor::ServiceError::BadRequest(
            "not inside a git repository".to_string(),
        ));
    }

    let has_head = run_git_bounded(
        context,
        &repo_root,
        ["rev-parse", "--verify", "HEAD"],
        64 * 1024,
    )?
    .status
    .success();
    let mut patch = Vec::new();
    let mut truncated = false;
    let mut base_ref = None;
    if has_head {
        let (label, commit) = resolve_diff_base_bounded(context, &repo_root, comparison)?;
        base_ref = Some(label);
        append_bounded_diff(
            &mut patch,
            &mut truncated,
            run_git_bounded(
                context,
                &repo_root,
                [
                    "diff",
                    "--no-ext-diff",
                    "--no-color",
                    "--unified=3",
                    commit.as_str(),
                    "--",
                ],
                MAX_DIFF_BYTES + 1,
            )?,
            false,
        )?;
    } else {
        append_bounded_diff(
            &mut patch,
            &mut truncated,
            run_git_bounded(
                context,
                &repo_root,
                [
                    "diff",
                    "--no-ext-diff",
                    "--no-color",
                    "--unified=3",
                    "--cached",
                    "--",
                ],
                MAX_DIFF_BYTES + 1,
            )?,
            false,
        )?;
        append_bounded_diff(
            &mut patch,
            &mut truncated,
            run_git_bounded(
                context,
                &repo_root,
                ["diff", "--no-ext-diff", "--no-color", "--unified=3", "--"],
                MAX_DIFF_BYTES + 1,
            )?,
            false,
        )?;
    }

    let untracked = run_git_bounded(
        context,
        &repo_root,
        ["ls-files", "--others", "--exclude-standard", "-z"],
        4 * 1024 * 1024,
    )?;
    if !untracked.status.success() || untracked.stderr_truncated {
        return Err(crate::runtime_actor::ServiceError::Internal);
    }
    truncated |= untracked.stdout_truncated;
    for path in untracked
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .take(MAX_UNTRACKED_FILES)
    {
        context.checkpoint()?;
        if patch.len() >= MAX_DIFF_BYTES {
            truncated = true;
            break;
        }
        #[cfg(unix)]
        let path = OsString::from_vec(path.to_vec());
        #[cfg(not(unix))]
        let path = OsString::from(String::from_utf8_lossy(path).into_owned());
        let args = vec![
            OsString::from("diff"),
            OsString::from("--no-index"),
            OsString::from("--no-color"),
            OsString::from("--unified=3"),
            OsString::from("--"),
            OsString::from("/dev/null"),
            path,
        ];
        append_bounded_diff(
            &mut patch,
            &mut truncated,
            run_git_bounded(context, &repo_root, args, MAX_DIFF_BYTES + 1)?,
            true,
        )?;
    }

    if patch.len() > MAX_DIFF_BYTES {
        patch.truncate(MAX_DIFF_BYTES);
        truncated = true;
    }
    Ok(SessionReadDiffResult {
        patch,
        repo_root: repo_root.display().to_string(),
        truncated,
        base_ref,
    })
}

fn resolve_diff_base_bounded(
    context: &crate::long_running::JobContext,
    repo_root: &Path,
    comparison: SessionDiffBase,
) -> crate::runtime_actor::ServiceResult<(String, String)> {
    if comparison == SessionDiffBase::Head {
        return Ok(("HEAD".to_string(), "HEAD".to_string()));
    }
    let origin_head = run_git_bounded(
        context,
        repo_root,
        [
            "symbolic-ref",
            "--quiet",
            "--short",
            "refs/remotes/origin/HEAD",
        ],
        64 * 1024,
    )?;
    let mut candidates = Vec::new();
    if origin_head.status.success()
        && !origin_head.stdout_truncated
        && !origin_head.stderr_truncated
    {
        let candidate = String::from_utf8_lossy(&origin_head.stdout)
            .trim()
            .to_string();
        if !candidate.is_empty() {
            candidates.push(candidate);
        }
    }
    candidates.extend(
        ["origin/main", "main", "origin/master", "master"]
            .into_iter()
            .map(str::to_string),
    );
    for candidate in candidates {
        context.checkpoint()?;
        let peeled = format!("{candidate}^{{commit}}");
        let verified = run_git_bounded(
            context,
            repo_root,
            ["rev-parse", "--verify", "--quiet", peeled.as_str()],
            64 * 1024,
        )?;
        if !verified.status.success() {
            continue;
        }
        let merge_base = run_git_bounded(
            context,
            repo_root,
            ["merge-base", candidate.as_str(), "HEAD"],
            64 * 1024,
        )?;
        if merge_base.status.success()
            && !merge_base.stdout_truncated
            && !merge_base.stderr_truncated
        {
            let commit = String::from_utf8_lossy(&merge_base.stdout)
                .trim()
                .to_string();
            if !commit.is_empty() {
                return Ok((candidate, commit));
            }
        }
    }
    Ok(("HEAD".to_string(), "HEAD".to_string()))
}

fn require_git_success(
    output: &crate::long_running::BoundedOutput,
) -> crate::runtime_actor::ServiceResult<()> {
    if output.status.success() && !output.stdout_truncated && !output.stderr_truncated {
        return Ok(());
    }
    let _safe_stderr = String::from_utf8_lossy(&output.stderr);
    Err(crate::runtime_actor::ServiceError::Internal)
}

fn append_bounded_diff(
    patch: &mut Vec<u8>,
    truncated: &mut bool,
    output: crate::long_running::BoundedOutput,
    allow_diff_exit_one: bool,
) -> crate::runtime_actor::ServiceResult<()> {
    let accepted =
        output.status.success() || (allow_diff_exit_one && output.status.code() == Some(1));
    if !accepted || output.stderr_truncated {
        let _safe_stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::runtime_actor::ServiceError::Internal);
    }
    *truncated |= output.stdout_truncated;
    append_diff_bytes(patch, &output.stdout);
    Ok(())
}

pub(crate) fn create_worktree_bounded(
    context: &crate::long_running::JobContext,
    request: homie_proto::WorktreeCreateRequest,
) -> crate::runtime_actor::ServiceResult<WorktreeInfo> {
    let repo_path = PathBuf::from(request.repo_path);
    let branch = request.branch.unwrap_or_else(generated_branch_name);
    let worktree_path = sibling_worktree_path(&repo_path, &branch)
        .map_err(|_| crate::runtime_actor::ServiceError::BadRequest("invalid repo path".into()))?;
    let mut args = vec![
        OsString::from("worktree"),
        OsString::from("add"),
        OsString::from("-b"),
        OsString::from(&branch),
        worktree_path.as_os_str().to_os_string(),
    ];
    if let Some(base) = request.base {
        args.push(OsString::from(base));
    }
    let output = run_git_bounded(context, &repo_path, args, 4 * 1024 * 1024)?;
    require_git_success(&output)?;
    Ok(list_git_worktrees_bounded(context, &repo_path)?
        .into_iter()
        .find(|entry| same_path(Path::new(&entry.path), &worktree_path))
        .unwrap_or(WorktreeInfo {
            path: canonical_display(&worktree_path),
            branch: Some(branch),
            is_bare: false,
            is_detached: false,
            is_prunable: false,
        }))
}

pub(crate) fn remove_worktree_bounded(
    context: &crate::long_running::JobContext,
    request: homie_proto::WorktreeRemoveRequest,
) -> crate::runtime_actor::ServiceResult<()> {
    let repo_path = PathBuf::from(request.repo_path);
    let mut args = vec![OsString::from("worktree"), OsString::from("remove")];
    if request.force {
        args.push(OsString::from("--force"));
    }
    args.push(OsString::from(request.worktree_path));
    let output = run_git_bounded(context, &repo_path, args, 4 * 1024 * 1024)?;
    require_git_success(&output)
}

pub fn create_worktree(request: WorktreeCreateRequest) -> Result<WorktreeInfo, RuntimeError> {
    let branch = request.branch.unwrap_or_else(generated_branch_name);
    let worktree_path = sibling_worktree_path(&request.repo_path, &branch)?;
    let mut args = vec![
        "worktree".to_string(),
        "add".to_string(),
        "-b".to_string(),
        branch.clone(),
        worktree_path.display().to_string(),
    ];
    if let Some(base) = request.base {
        args.push(base);
    }
    run_git(args.iter().map(String::as_str), &request.repo_path)?;
    Ok(list_git_worktrees(&request.repo_path)?
        .into_iter()
        .find(|entry| same_path(Path::new(&entry.path), &worktree_path))
        .unwrap_or(WorktreeInfo {
            path: canonical_display(&worktree_path),
            branch: Some(branch),
            is_bare: false,
            is_detached: false,
            is_prunable: false,
        }))
}

pub fn remove_worktree(request: WorktreeRemoveRequest) -> Result<(), RuntimeError> {
    let mut args = vec!["worktree".to_string(), "remove".to_string()];
    if request.force {
        args.push("--force".to_string());
    }
    args.push(request.worktree_path.display().to_string());
    run_git(args.iter().map(String::as_str), &request.repo_path)?;
    Ok(())
}

pub fn parse_worktree_porcelain(porcelain: &str) -> Vec<WorktreeInfo> {
    #[derive(Default)]
    struct Block {
        path: Option<String>,
        branch: Option<String>,
        is_bare: bool,
        is_detached: bool,
        is_prunable: bool,
    }

    fn flush(block: &mut Block, results: &mut Vec<WorktreeInfo>) {
        let Some(path) = block.path.take() else {
            return;
        };
        let finished = std::mem::take(block);
        results.push(WorktreeInfo {
            path,
            branch: finished.branch,
            is_bare: finished.is_bare,
            is_detached: finished.is_detached,
            is_prunable: finished.is_prunable,
        });
    }

    let mut block = Block::default();
    let mut results = Vec::new();
    for line in porcelain.split('\n') {
        if line.is_empty() {
            flush(&mut block, &mut results);
        } else if let Some(rest) = line.strip_prefix("worktree ") {
            flush(&mut block, &mut results);
            block.path = Some(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("branch ") {
            block.branch = Some(rest.strip_prefix("refs/heads/").unwrap_or(rest).to_string());
        } else if line == "bare" {
            block.is_bare = true;
        } else if line == "detached" {
            block.is_detached = true;
        } else if line == "prunable" || line.starts_with("prunable ") {
            block.is_prunable = true;
        }
    }
    flush(&mut block, &mut results);
    results
}

fn sibling_worktree_path(repo_path: &Path, branch: &str) -> Result<PathBuf, RuntimeError> {
    let parent = repo_path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("repo path has no parent: {}", repo_path.display()),
        )
    })?;
    let repo_name = repo_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("repo path has no file name: {}", repo_path.display()),
            )
        })?;
    Ok(parent.join(format!("{repo_name}-{}", branch_to_path_slug(branch))))
}

fn branch_to_path_slug(branch: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;
    for ch in branch.to_ascii_lowercase().chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            slug.push(ch);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn generated_branch_name() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("homie/worktree-{nanos:x}")
}

fn same_path(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn canonical_display(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

fn run_git<'a, I>(args: I, cwd: &Path) -> Result<String, RuntimeError>
where
    I: IntoIterator<Item = &'a str>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    let output = Command::new("/usr/bin/git")
        .args(&args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
        .into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("session {0} is not live")]
    SessionNotLive(String),
}

impl RuntimeSupervisor {
    pub fn open(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        Self::open_inner(config, None)
    }

    pub fn open_with_holder(
        config: RuntimeConfig,
        holder_binary: PathBuf,
    ) -> Result<Self, RuntimeError> {
        Self::open_inner(config, Some(holder_binary))
    }

    fn open_inner(
        config: RuntimeConfig,
        holder_binary: Option<PathBuf>,
    ) -> Result<Self, RuntimeError> {
        let storage = homie_storage::open_or_create(StorageConfig {
            data_dir: config.data_dir.clone(),
        })?;
        storage.migrate()?;
        storage.seed_defaults()?;
        storage.mark_interrupted_sessions_detached()?;
        let live_sessions = adopt_live_holders(&storage, &config.data_dir)?;
        let event_store = Arc::new(EventStore::open(config.data_dir.clone())?);
        Ok(Self {
            data_dir: config.data_dir,
            storage,
            live_sessions: Mutex::new(live_sessions),
            holder_binary,
            event_store,
        })
    }

    pub fn spawn_shell(
        &self,
        cwd: &Path,
        title: Option<&str>,
    ) -> Result<SessionSummary, RuntimeError> {
        self.spawn_shell_with_parent(cwd, title, None)
    }

    pub fn spawn_shell_with_parent(
        &self,
        cwd: &Path,
        title: Option<&str>,
        parent_session_id: Option<&str>,
    ) -> Result<SessionSummary, RuntimeError> {
        if !cwd.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("cwd does not exist: {}", cwd.display()),
            )
            .into());
        }
        let shell = Path::new("/bin/sh");
        if !shell.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "required shell not found: /bin/sh",
            )
            .into());
        }

        let session = self.storage.create_session_with_parent(
            CreateSession {
                workspace: cwd.to_path_buf(),
                title: title.map(str::to_string),
            },
            parent_session_id,
        )?;
        let holder = HolderPaths::new(&self.data_dir, &session.id);
        if let Err(error) = launch_holder(
            &holder,
            self.holder_binary.as_deref(),
            &self.output_log_path(&session.id),
            cwd,
            120,
            40,
        ) {
            self.storage.delete_session(&session.id)?;
            return Err(error.into());
        }
        self.live_sessions.lock().expect("live sessions").insert(
            session.id.clone(),
            LiveSession {
                holder: holder.clone(),
            },
        );
        let session = match self.storage.update_session_status(&session.id, "running") {
            Ok(session) => session,
            Err(error) => {
                self.stop_holder_if_live(&session.id)?;
                self.storage.delete_session(&session.id)?;
                return Err(error.into());
            }
        };
        self.emit_session_event(
            EventName::SESSION_SPAWNED,
            &session.id,
            Some(&session.status),
        );
        Ok(session)
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionSummary>, RuntimeError> {
        Ok(self.storage.list_sessions()?)
    }

    #[must_use]
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    #[must_use]
    pub fn event_store(&self) -> Arc<EventStore> {
        self.event_store.clone()
    }

    pub fn prepare_shutdown(&self) -> Result<(), RuntimeError> {
        self.event_store.sync()?;
        self.storage
            .connection()
            .execute_batch("PRAGMA optimize; PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(StorageError::from)?;
        Ok(())
    }

    pub fn send_text(
        &self,
        session_id: &str,
        text: &str,
        submit: bool,
    ) -> Result<(), RuntimeError> {
        let response = holder::request(
            &self.holder_socket(session_id),
            &HolderRequest::Write {
                text: text.to_string(),
                submit,
            },
        )
        .map_err(|_| RuntimeError::SessionNotLive(session_id.to_string()))?;
        if !response.ok {
            return Err(std::io::Error::other(
                response
                    .error
                    .unwrap_or_else(|| "holder write failed".into()),
            )
            .into());
        }
        let _ = self.storage.update_session_status(session_id, "running")?;
        self.emit_session_event(EventName::SESSION_OUTPUT, session_id, None);
        Ok(())
    }

    pub fn send_bytes(&self, session_id: &str, bytes: &[u8]) -> Result<(), RuntimeError> {
        let response = holder::request(
            &self.holder_socket(session_id),
            &HolderRequest::WriteBytes {
                bytes: bytes.to_vec(),
            },
        )
        .map_err(|_| RuntimeError::SessionNotLive(session_id.to_string()))?;
        if !response.ok {
            return Err(std::io::Error::other(
                response
                    .error
                    .unwrap_or_else(|| "holder byte write failed".into()),
            )
            .into());
        }
        let _ = self.storage.update_session_status(session_id, "running")?;
        self.emit_session_event(EventName::SESSION_OUTPUT, session_id, None);
        Ok(())
    }

    pub fn read_output(&self, session_id: &str) -> Result<String, RuntimeError> {
        let path = self.output_log_path(session_id);
        match std::fs::read_to_string(path) {
            Ok(output) => Ok(output),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn read_output_range(
        &self,
        session_id: &str,
        offset: u64,
        max_bytes: usize,
    ) -> Result<(u64, Vec<u8>), RuntimeError> {
        let path = self.output_log_path(session_id);
        let mut file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok((offset, Vec::new()));
            }
            Err(error) => return Err(error.into()),
        };
        let file_len = file.metadata()?.len();
        let offset = offset.min(file_len);
        file.seek(std::io::SeekFrom::Start(offset))?;
        let mut buffer = vec![0_u8; max_bytes];
        let read = file.read(&mut buffer)?;
        buffer.truncate(read);
        Ok((offset + read as u64, buffer))
    }

    pub fn read_screen_lines(&self, session_id: &str) -> Result<Vec<String>, RuntimeError> {
        let output = self.read_output(session_id)?;
        let mut screen = HeadlessScreen::new(120, 40);
        screen.feed(output.as_bytes());
        Ok(screen.lines())
    }

    pub fn session_status_report(
        &self,
        session_id: &str,
    ) -> Result<SessionStatusReport, RuntimeError> {
        let preparation = self.prepare_session_status(session_id)?;
        let output = self.read_output(session_id)?;
        Ok(status_report_from_output(&preparation, output.as_bytes()))
    }

    pub fn report_needs_input(
        &self,
        session_id: &str,
        detail: &NeedsInputDetail,
    ) -> Result<(), RuntimeError> {
        let kind = match detail.kind {
            NeedsInputKind::Approval => "approval",
            NeedsInputKind::Question => "question",
            NeedsInputKind::Error => "error",
            NeedsInputKind::Unknown => "unknown",
        };
        self.storage
            .set_session_needs_input(session_id, kind, &serde_json::to_value(detail)?)?;
        self.emit_session_event(
            EventName::SESSION_NEEDS_INPUT,
            session_id,
            Some("needs_input"),
        );
        Ok(())
    }

    pub fn report_turn_complete(&self, session_id: &str) -> Result<(), RuntimeError> {
        self.storage.update_session_status(session_id, "idle")?;
        self.emit_session_event(EventName::SESSION_STATUS, session_id, Some("idle"));
        Ok(())
    }

    pub fn session_status_projection(&self, session_id: &str) -> Result<String, RuntimeError> {
        let report = self.session_status_report(session_id)?;
        Ok(status_to_storage(&report.status).to_string())
    }

    pub fn session_snapshot(
        &self,
        session_id: &str,
        output_offset: u64,
        max_bytes: usize,
    ) -> Result<SessionSnapshot, RuntimeError> {
        let session = self
            .storage
            .list_sessions()?
            .into_iter()
            .find(|session| session.id == session_id)
            .ok_or_else(|| RuntimeError::SessionNotLive(session_id.to_string()))?;
        let status = self.session_status_report(session_id)?;
        let (output_offset, output) =
            self.read_output_range(session_id, output_offset, max_bytes)?;
        let holder = holder::request(
            &HolderPaths::new(&self.data_dir, session_id).socket,
            &HolderRequest::Stat,
        )
        .ok()
        .filter(|response| response.ok)
        .map(HolderSnapshot::from);
        Ok(SessionSnapshot {
            session,
            status,
            output_offset,
            output,
            holder,
        })
    }

    pub fn write_screen_checkpoint(
        &self,
        session_id: &str,
    ) -> Result<ScreenCheckpoint, RuntimeError> {
        let output = self.read_output(session_id)?;
        let mut screen = HeadlessScreen::new(120, 40);
        screen.feed(output.as_bytes());
        let snapshot = screen.snapshot();
        let checkpoint = ScreenCheckpoint {
            session_id: session_id.to_string(),
            output_offset: output.len() as u64,
            content_seq: snapshot.content_seq,
            lines: snapshot.lines,
        };
        let path = self.screen_checkpoint_path(session_id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_vec_pretty(&checkpoint)?)?;
        Ok(checkpoint)
    }

    pub fn read_screen_checkpoint(
        &self,
        session_id: &str,
    ) -> Result<Option<ScreenCheckpoint>, RuntimeError> {
        let path = self.screen_checkpoint_path(session_id);
        match std::fs::read(path) {
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn terminate_session(&self, session_id: &str) -> Result<(), RuntimeError> {
        let holder = self
            .live_sessions
            .lock()
            .expect("live sessions")
            .get(session_id)
            .map(|session| session.holder.clone())
            .unwrap_or_else(|| HolderPaths::new(&self.data_dir, session_id));
        let response = holder::request(&holder.socket, &HolderRequest::Terminate)
            .map_err(|_| RuntimeError::SessionNotLive(session_id.to_string()))?;
        if !response.ok {
            return Err(std::io::Error::other(
                response
                    .error
                    .unwrap_or_else(|| "holder terminate failed".into()),
            )
            .into());
        }
        wait_for_holder_shutdown(&holder.socket, &holder.pid_file, Duration::from_secs(3))?;
        self.live_sessions
            .lock()
            .expect("live sessions")
            .remove(session_id);
        self.storage.update_session_status(session_id, "exited")?;
        self.emit_session_event(EventName::SESSION_STATUS, session_id, Some("exited"));
        Ok(())
    }

    pub fn resize_session(
        &self,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), RuntimeError> {
        let response = holder::request(
            &self.holder_socket(session_id),
            &HolderRequest::Resize { cols, rows },
        )
        .map_err(|_| RuntimeError::SessionNotLive(session_id.to_string()))?;
        if !response.ok {
            return Err(std::io::Error::other(
                response
                    .error
                    .unwrap_or_else(|| "holder resize failed".into()),
            )
            .into());
        }
        self.emit_session_event(EventName::SESSION_UPDATED, session_id, None);
        Ok(())
    }

    pub fn archive(&self, session_id: &str) -> Result<SessionSummary, RuntimeError> {
        self.stop_holder_if_live(session_id)?;
        let session = self.storage.update_session_status(session_id, "archived")?;
        self.emit_session_event(EventName::SESSION_ARCHIVED, session_id, Some("archived"));
        Ok(session)
    }

    pub fn hibernate(&self, session_id: &str) -> Result<SessionSummary, RuntimeError> {
        self.stop_holder_if_live(session_id)?;
        let session = self
            .storage
            .update_session_status(session_id, "hibernated")?;
        self.emit_session_event(EventName::SESSION_STATUS, session_id, Some("hibernated"));
        Ok(session)
    }

    pub fn wake(&self, session_id: &str) -> Result<SessionSummary, RuntimeError> {
        let session = self.session_summary(session_id)?;
        let workspace = PathBuf::from(&session.workspace);
        if !workspace.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("workspace does not exist: {}", workspace.display()),
            )
            .into());
        }
        let holder = HolderPaths::new(&self.data_dir, session_id);
        if holder_runtime_status(&holder).as_deref() != Some("running") {
            launch_holder(
                &holder,
                self.holder_binary.as_deref(),
                &self.output_log_path(session_id),
                &workspace,
                120,
                40,
            )?;
        }
        self.live_sessions.lock().expect("live sessions").insert(
            session_id.to_string(),
            LiveSession {
                holder: holder.clone(),
            },
        );
        let session = self.storage.update_session_status(session_id, "running")?;
        self.emit_session_event(EventName::SESSION_STATUS, session_id, Some("running"));
        Ok(session)
    }

    pub fn events_after(&self, after_seq: u64, filter: &[String]) -> Vec<RuntimeEvent> {
        self.event_store
            .replay(after_seq)
            .events
            .into_iter()
            .filter(|event| filter.is_empty() || filter.iter().any(|wanted| wanted == &event.event))
            .collect()
    }

    pub(crate) fn output_log_path(&self, session_id: &str) -> PathBuf {
        self.data_dir
            .join("runtime")
            .join("output")
            .join(format!("{session_id}.log"))
    }

    fn holder_socket(&self, session_id: &str) -> PathBuf {
        self.live_sessions
            .lock()
            .expect("live sessions")
            .get(session_id)
            .map(|session| session.holder.socket.clone())
            .unwrap_or_else(|| HolderPaths::new(&self.data_dir, session_id).socket)
    }

    fn stop_holder_if_live(&self, session_id: &str) -> Result<(), RuntimeError> {
        let holder = self
            .live_sessions
            .lock()
            .expect("live sessions")
            .get(session_id)
            .map(|session| session.holder.clone())
            .unwrap_or_else(|| HolderPaths::new(&self.data_dir, session_id));
        match holder::request(&holder.socket, &HolderRequest::Terminate) {
            Ok(response) if response.ok => {
                wait_for_holder_shutdown(&holder.socket, &holder.pid_file, Duration::from_secs(3))?;
            }
            Ok(response) => {
                return Err(std::io::Error::other(
                    response
                        .error
                        .unwrap_or_else(|| "holder terminate failed".into()),
                )
                .into());
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => {}
            Err(error) => return Err(RuntimeError::Io(error)),
        }
        self.live_sessions
            .lock()
            .expect("live sessions")
            .remove(session_id);
        Ok(())
    }

    fn session_summary(&self, session_id: &str) -> Result<SessionSummary, RuntimeError> {
        self.storage
            .list_sessions()?
            .into_iter()
            .find(|session| session.id == session_id)
            .ok_or_else(|| RuntimeError::SessionNotLive(session_id.to_string()))
    }

    fn emit_session_event(&self, event: &str, session_id: &str, status: Option<&str>) {
        let _ = self.event_store.publish(
            event,
            Some(session_id.to_string()),
            status.map(str::to_string),
        );
    }

    fn screen_checkpoint_path(&self, session_id: &str) -> PathBuf {
        self.data_dir
            .join("runtime")
            .join("checkpoints")
            .join(format!("{session_id}.screen.json"))
    }

    pub(crate) fn prepare_session_status(
        &self,
        session_id: &str,
    ) -> Result<SessionStatusPreparation, RuntimeError> {
        let holder = HolderPaths::new(&self.data_dir, session_id);
        Ok(SessionStatusPreparation {
            persisted_status: persisted_projection_status(&self.storage, session_id)?,
            holder_status: holder_runtime_status(&holder),
            holder_exited: holder_status_is_exited(&holder),
            persisted_needs_input: self.persisted_needs_input(session_id)?,
        })
    }

    pub(crate) fn holder_snapshot(&self, session_id: &str) -> Option<HolderSnapshot> {
        holder::request(
            &HolderPaths::new(&self.data_dir, session_id).socket,
            &HolderRequest::Stat,
        )
        .ok()
        .filter(|response| response.ok)
        .map(HolderSnapshot::from)
    }

    fn persisted_needs_input(
        &self,
        session_id: &str,
    ) -> Result<Option<NeedsInputDetail>, RuntimeError> {
        let metadata = self.storage.session_core_metadata(session_id)?;
        if metadata.needs_input_kind.is_none() {
            return Ok(None);
        }
        Ok(Some(serde_json::from_value(metadata.needs_input_payload)?))
    }
}

pub(crate) fn status_report_from_output(
    preparation: &SessionStatusPreparation,
    output: &[u8],
) -> SessionStatusReport {
    let (lines, observation) = screen_observation_from_output(output);
    if matches!(
        preparation.persisted_status,
        SessionStatus::Archived | SessionStatus::Hibernated
    ) {
        return SessionStatusReport {
            status: preparation.persisted_status.clone(),
            needs_input: None,
            turn_completed: false,
            screen_lines: lines,
            screen_observation: observation.map(Into::into),
        };
    }
    if matches!(preparation.persisted_status, SessionStatus::Idle) {
        return SessionStatusReport {
            status: preparation.persisted_status.clone(),
            needs_input: None,
            turn_completed: true,
            screen_lines: lines,
            screen_observation: observation.map(Into::into),
        };
    }
    if preparation
        .holder_status
        .as_deref()
        .is_some_and(|status| status.starts_with("exited"))
        || preparation.holder_exited
    {
        return SessionStatusReport {
            status: SessionStatus::Exited,
            needs_input: None,
            turn_completed: false,
            screen_lines: lines,
            screen_observation: observation.map(Into::into),
        };
    }

    let mut reducer = StatusReducer::new(Authority::ScreenPrimary, std::time::UNIX_EPOCH)
        .with_timing(runtime_reducer_timing());
    let mut outcome = None;
    if let Some(observation) = observation.clone() {
        outcome = Some(reducer.reduce(
            StatusSignal::Screen(observation.clone()),
            std::time::UNIX_EPOCH + Duration::from_secs(10),
        ));
    } else if preparation.holder_status.as_deref() == Some("running") && !lines.is_empty() {
        outcome = Some(reducer.reduce(
            StatusSignal::PtyOutputActivity,
            std::time::UNIX_EPOCH + Duration::from_secs(10),
        ));
    }
    if preparation.holder_status.as_deref() == Some("running") {
        let tick = reducer.reduce(
            StatusSignal::Tick,
            std::time::UNIX_EPOCH + Duration::from_secs(11),
        );
        if let Some(outcome) = outcome.as_mut() {
            if outcome.status_change.is_none() {
                outcome.status_change = tick.status_change;
            }
            if outcome.needs_input.is_none() {
                outcome.needs_input = tick.needs_input;
            }
            outcome.turn_completed |= tick.turn_completed;
        } else {
            outcome = Some(tick);
        }
    }

    let status = match preparation.holder_status.as_deref() {
        Some("running") if *reducer.status() == SessionStatus::Starting => SessionStatus::Running,
        Some("running") => reducer.status().clone(),
        _ => preparation.persisted_status.clone(),
    };
    let outcome = outcome.unwrap_or_default();
    let needs_input = outcome
        .needs_input
        .or_else(|| preparation.persisted_needs_input.clone());
    let status = if needs_input.is_some() && !matches!(status, SessionStatus::Exited) {
        SessionStatus::NeedsInput
    } else {
        status
    };
    SessionStatusReport {
        status,
        needs_input,
        turn_completed: outcome.turn_completed,
        screen_lines: lines,
        screen_observation: observation.map(Into::into),
    }
}

fn screen_observation_from_output(output: &[u8]) -> (Vec<String>, Option<ScreenObservation>) {
    let mut screen = HeadlessScreen::new(120, 40);
    screen.feed(output);
    let snapshot = screen.snapshot();
    let lines = snapshot.lines.clone();
    let observation = classify_screen(&snapshot);
    (lines, observation)
}

pub(crate) fn read_file_range_bounded(
    context: &crate::long_running::JobContext,
    path: &Path,
    offset: u64,
    max_bytes: usize,
) -> crate::runtime_actor::ServiceResult<(u64, Vec<u8>)> {
    context.checkpoint()?;
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((offset, Vec::new()));
        }
        Err(_) => return Err(crate::runtime_actor::ServiceError::Internal),
    };
    let file_len = file
        .metadata()
        .map_err(|_| crate::runtime_actor::ServiceError::Internal)?
        .len();
    let offset = offset.min(file_len);
    file.seek(std::io::SeekFrom::Start(offset))
        .map_err(|_| crate::runtime_actor::ServiceError::Internal)?;
    let mut output = Vec::with_capacity(max_bytes.min(64 * 1024));
    let mut buffer = [0_u8; 64 * 1024];
    while output.len() < max_bytes {
        context.checkpoint()?;
        let remaining = max_bytes - output.len();
        let chunk_len = remaining.min(buffer.len());
        let read = file
            .read(&mut buffer[..chunk_len])
            .map_err(|_| crate::runtime_actor::ServiceError::Internal)?;
        if read == 0 {
            break;
        }
        output.extend_from_slice(&buffer[..read]);
    }
    Ok((offset + output.len() as u64, output))
}

pub(crate) fn read_file_bounded(
    context: &crate::long_running::JobContext,
    path: &Path,
    max_bytes: usize,
) -> crate::runtime_actor::ServiceResult<Vec<u8>> {
    read_file_range_bounded(context, path, 0, max_bytes).map(|(_, output)| output)
}

fn launch_holder(
    paths: &HolderPaths,
    binary: Option<&Path>,
    log_path: &Path,
    cwd: &Path,
    cols: u16,
    rows: u16,
) -> std::io::Result<()> {
    if let Some(parent) = paths.socket.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let _ = std::fs::remove_file(&paths.socket);
    let holder = match binary {
        Some(path) => path.to_path_buf(),
        None => holder_binary()?,
    };
    let mut child = Command::new(holder)
        .arg("--socket")
        .arg(&paths.socket)
        .arg("--pid-file")
        .arg(&paths.pid_file)
        .arg("--status-file")
        .arg(&paths.status_file)
        .arg("--log-path")
        .arg(log_path)
        .arg("--cwd")
        .arg(cwd)
        .arg("--cols")
        .arg(cols.to_string())
        .arg("--rows")
        .arg(rows.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    if let Err(error) = wait_for_holder(&paths.socket, Duration::from_secs(3)) {
        let _ = holder::request(&paths.socket, &HolderRequest::Terminate);
        let _ = child.kill();
        let _ = child.wait();
        let _ = std::fs::remove_file(&paths.socket);
        let _ = std::fs::remove_file(&paths.pid_file);
        return Err(error);
    }
    Ok(())
}

fn holder_binary() -> std::io::Result<PathBuf> {
    if let Some(path) = std::env::var_os("HOMIE_HOLDER_BIN") {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = option_env!("CARGO_BIN_EXE_homie-runtime-holder") {
        return Ok(PathBuf::from(path));
    }
    let current = std::env::current_exe()?;
    let sibling = current.with_file_name("homie-runtime-holder");
    if sibling.is_file() {
        return Ok(sibling);
    }
    if let Some(debug_dir) = current.parent().and_then(Path::parent) {
        let debug_binary = debug_dir.join("homie-runtime-holder");
        if debug_binary.is_file() {
            return Ok(debug_binary);
        }
    }
    let resources = current.parent().and_then(Path::parent).map(|contents| {
        contents
            .join("Resources")
            .join("bin")
            .join("homie-runtime-holder")
    });
    if let Some(path) = resources
        && path.is_file()
    {
        return Ok(path);
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "homie-runtime-holder binary not found",
    ))
}

fn wait_for_holder(socket: &Path, timeout: Duration) -> std::io::Result<()> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Ok(response) = holder::request(socket, &HolderRequest::Stat)
            && response.ok
            && response.status.as_deref() == Some("running")
        {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        format!("holder did not become ready: {}", socket.display()),
    ))
}

fn wait_for_holder_shutdown(
    socket: &Path,
    pid_file: &Path,
    timeout: Duration,
) -> std::io::Result<()> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !socket.exists() && !pid_file.exists() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        format!(
            "holder did not shut down: socket={}, pid_file={}",
            socket.display(),
            pid_file.display()
        ),
    ))
}

fn adopt_live_holders(
    storage: &Storage,
    data_dir: &Path,
) -> Result<HashMap<String, LiveSession>, RuntimeError> {
    let mut live = HashMap::new();
    for session in storage.list_sessions()? {
        let holder = HolderPaths::new(data_dir, &session.id);
        if let Ok(response) = holder::request(&holder.socket, &HolderRequest::Stat)
            && response.ok
        {
            match response.status.as_deref() {
                Some("running") => {
                    if matches!(session.status.as_str(), "created" | "starting" | "running") {
                        let _ = storage.mark_session_running_if_exists(&session.id)?;
                    }
                    live.insert(session.id, LiveSession { holder });
                }
                Some(status) if status.starts_with("exited") => {
                    let _ = storage.update_session_status(&session.id, "exited")?;
                }
                _ => {}
            }
        } else if holder_status_is_exited(&holder) {
            let _ = storage.update_session_status(&session.id, "exited")?;
        }
    }
    Ok(live)
}

fn holder_status_is_exited(holder: &HolderPaths) -> bool {
    std::fs::read_to_string(&holder.status_file)
        .map(|status| status.trim_start().starts_with("exited"))
        .unwrap_or(false)
}

fn holder_runtime_status(holder: &HolderPaths) -> Option<String> {
    holder::request(&holder.socket, &HolderRequest::Stat)
        .ok()
        .filter(|response| response.ok)
        .and_then(|response| response.status)
}

fn persisted_projection_status(
    storage: &Storage,
    session_id: &str,
) -> Result<SessionStatus, RuntimeError> {
    let session = storage
        .list_sessions()?
        .into_iter()
        .find(|session| session.id == session_id)
        .ok_or_else(|| RuntimeError::SessionNotLive(session_id.to_string()))?;
    Ok(match session.status.as_str() {
        "created" | "starting" | "running" => SessionStatus::Unknown("detached".to_string()),
        "needs_input" => SessionStatus::NeedsInput,
        "idle" => SessionStatus::Idle,
        "hibernated" => SessionStatus::Hibernated,
        "archived" => SessionStatus::Archived,
        "exited" => SessionStatus::Exited,
        other => SessionStatus::Unknown(other.to_string()),
    })
}

fn status_to_storage(status: &SessionStatus) -> &str {
    match status {
        SessionStatus::Created => "created",
        SessionStatus::Starting => "starting",
        SessionStatus::Running => "running",
        SessionStatus::NeedsInput => "needs_input",
        SessionStatus::Idle => "idle",
        SessionStatus::Hibernated => "hibernated",
        SessionStatus::Archived => "archived",
        SessionStatus::Exited => "exited",
        SessionStatus::Unknown(value) if value == "detached" => "detached",
        SessionStatus::Unknown(_) => "unknown",
    }
}

fn runtime_reducer_timing() -> ReducerTiming {
    ReducerTiming {
        idle_confirmations: 1,
        startup_grace: Duration::ZERO,
        ..ReducerTiming::default()
    }
}

fn classify_screen(snapshot: &ScreenSnapshot) -> Option<ScreenObservation> {
    let text = snapshot.lines.join("\n");
    let lower = text.to_ascii_lowercase();
    let idle_pos = latest_of(
        &lower,
        &["homie-status:idle"],
        if snapshot
            .lines
            .iter()
            .rev()
            .take(5)
            .any(|line| line.trim_start().starts_with(['❯', '›']))
        {
            Some(lower.len())
        } else {
            None
        },
    )
    .or_else(|| (snapshot.osc_progress_state == Some(0)).then_some(lower.len()));
    let working_pos = latest_of(
        &lower,
        &["homie-status:working", "working..."],
        snapshot
            .osc_title
            .as_ref()
            .is_some_and(|title| looks_like_spinner(title))
            .then_some(lower.len()),
    )
    .or_else(|| (snapshot.osc_progress_state == Some(1)).then_some(lower.len()));
    let blocker_pos = latest_of(
        &lower,
        &[
            "press enter to confirm or esc to cancel",
            "allow command?",
            "permission required",
        ],
        None,
    );
    let question_pos = latest_of(&lower, &["enter to submit answer"], None);

    let (_, state) = [
        idle_pos.map(|position| (position, ManifestState::Idle)),
        working_pos.map(|position| (position, ManifestState::Working)),
        blocker_pos.map(|position| (position, ManifestState::BlockedPermission)),
        question_pos.map(|position| (position, ManifestState::BlockedQuestion)),
    ]
    .into_iter()
    .flatten()
    .max_by_key(|(position, _)| *position)?;

    let observation = match state {
        ManifestState::Idle => ScreenObservation {
            state: ManifestState::Idle,
            matched_rule_id: "runtime-idle-text".to_string(),
            priority: 500,
            content_seq: snapshot.content_seq,
            prompt_excerpt: None,
            options: None,
        },
        ManifestState::Working => ScreenObservation {
            state: ManifestState::Working,
            matched_rule_id: "runtime-working-text".to_string(),
            priority: 900,
            content_seq: snapshot.content_seq,
            prompt_excerpt: None,
            options: None,
        },
        ManifestState::BlockedPermission => ScreenObservation {
            state: ManifestState::BlockedPermission,
            matched_rule_id: "runtime-permission-text".to_string(),
            priority: 1000,
            content_seq: snapshot.content_seq,
            prompt_excerpt: excerpt_from(
                &snapshot.lines,
                ["allow command?", "permission required"],
            ),
            options: None,
        },
        ManifestState::BlockedQuestion => ScreenObservation {
            state: ManifestState::BlockedQuestion,
            matched_rule_id: "runtime-question-text".to_string(),
            priority: 950,
            content_seq: snapshot.content_seq,
            prompt_excerpt: excerpt_from(&snapshot.lines, ["enter to submit answer"]),
            options: None,
        },
        ManifestState::Skip => return None,
    };
    Some(observation)
}

fn latest_of(text: &str, needles: &[&str], fallback: Option<usize>) -> Option<usize> {
    needles
        .iter()
        .filter_map(|needle| text.rfind(needle))
        .chain(fallback)
        .max()
}

fn looks_like_spinner(title: &str) -> bool {
    title
        .chars()
        .next()
        .is_some_and(|ch| ('⠀'..='⣿').contains(&ch))
}

fn recent_excerpt(lines: &[String]) -> Option<String> {
    let excerpt = lines
        .iter()
        .rev()
        .filter(|line| !line.trim().is_empty())
        .take(5)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");
    if excerpt.is_empty() {
        None
    } else {
        Some(excerpt.chars().take(400).collect())
    }
}

fn excerpt_from<const N: usize>(lines: &[String], needles: [&str; N]) -> Option<String> {
    let lower_needles = needles.map(str::to_ascii_lowercase);
    let start = lines
        .iter()
        .position(|line| {
            let lower = line.to_ascii_lowercase();
            lower_needles
                .iter()
                .any(|needle| lower.contains(needle.as_str()))
        })
        .unwrap_or_else(|| lines.len().saturating_sub(5));
    let excerpt = lines[start..]
        .iter()
        .filter(|line| !line.trim().is_empty())
        .take(5)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    if excerpt.is_empty() {
        recent_excerpt(lines)
    } else {
        Some(excerpt.chars().take(400).collect())
    }
}

impl From<RuntimeScreenObservation> for ScreenObservation {
    fn from(value: RuntimeScreenObservation) -> Self {
        Self {
            state: value.state,
            matched_rule_id: value.matched_rule_id,
            priority: 0,
            content_seq: value.content_seq,
            prompt_excerpt: None,
            options: None,
        }
    }
}

impl From<ScreenObservation> for RuntimeScreenObservation {
    fn from(value: ScreenObservation) -> Self {
        Self {
            state: value.state,
            matched_rule_id: value.matched_rule_id,
            content_seq: value.content_seq,
        }
    }
}

impl From<HolderResponse> for HolderSnapshot {
    fn from(value: HolderResponse) -> Self {
        Self {
            pid: value.pid,
            status: value.status,
            tree_size: value.tree_size,
            cols: value.cols,
            rows: value.rows,
            log_offset: value.log_offset,
            epoch_offset: value.epoch_offset,
        }
    }
}

#[cfg(test)]
mod event_store_tests {
    use super::{RuntimeConfig, RuntimeEvent, RuntimeSupervisor};

    #[test]
    fn runtime_event_is_the_wire_owned_type() {
        let wire_event = homie_proto::model::RuntimeEvent {
            seq: 1,
            event: "runtime.ready".to_string(),
            session_id: None,
            status: None,
        };
        let runtime_event: RuntimeEvent = wire_event;

        assert_eq!(runtime_event.seq, 1);
    }

    #[test]
    fn runtime_supervisor_event_store_drives_cursor_and_persistence() {
        let temp = tempfile::tempdir().expect("tempdir");
        let supervisor = RuntimeSupervisor::open(RuntimeConfig {
            data_dir: temp.path().to_path_buf(),
        })
        .expect("runtime");

        supervisor.emit_session_event("session.status", "session-1", Some("running"));
        let store = supervisor.event_store();
        let replayed = store.replay(0).events;
        let snapshot_cursor = store.bounds().latest_seq;
        let latest = supervisor
            .events_after(0, &[])
            .last()
            .map(|event| event.seq);
        drop(supervisor);
        drop(store);
        let reopened = RuntimeSupervisor::open(RuntimeConfig {
            data_dir: temp.path().to_path_buf(),
        })
        .expect("reopen runtime");

        assert_eq!(
            (
                replayed.last().map(|event| event.seq),
                snapshot_cursor,
                latest,
                reopened.events_after(0, &[]).last().map(|event| event.seq),
            ),
            (Some(1), 1, Some(1), Some(1))
        );
    }
}
