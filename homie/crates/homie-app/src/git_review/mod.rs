//! Native Git review operations.
//!
//! The review cockpit crosses one deep module: discover a repository, read a
//! structured snapshot, then perform path- or patch-scoped mutations. Git's
//! porcelain formats, literal pathspec rules, subprocess hardening, timeouts,
//! and output limits stay local to this implementation.

use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

mod paths;
mod process;
mod status;
#[cfg(test)]
mod tests;

use paths::{literal_path_command, patch_creates_file, patch_rejected, validate_paths};
use process::{ensure_success, run_git};
use status::{parse_status, path_from_output_line, trim_line_ending};

const MAX_COMMIT_MESSAGE_BYTES: usize = 64 * 1024;
const MAX_PATCH_BYTES: usize = 4 * 1024 * 1024;

/// A discovered, non-bare Git worktree.
///
/// Holding the root makes subsequent operations insensitive to changes in the
/// process working directory and gives every mutation the same path-safety
/// rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitRepository {
    root: PathBuf,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReviewStatus {
    pub repo_root: PathBuf,
    pub branch: BranchInfo,
    pub staged: Vec<FileChange>,
    pub unstaged: Vec<FileChange>,
    pub untracked: Vec<FileChange>,
    pub conflicted: Vec<FileChange>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BranchInfo {
    /// `None` means detached HEAD. An unborn branch still has a name.
    pub name: Option<String>,
    /// The full HEAD object id, or `None` for an unborn branch.
    pub oid: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u64,
    pub behind: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileChange {
    /// Repository-relative, lossless on Unix, and safe to pass back to a
    /// mutation method as a literal path.
    pub path: PathBuf,
    pub original_path: Option<PathBuf>,
    pub kind: ChangeKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
    Unmerged,
    Unknown(char),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatchMutation {
    Stage,
    Unstage,
    Discard,
}

impl PatchMutation {
    const fn label(self) -> &'static str {
        match self {
            Self::Stage => "staging",
            Self::Unstage => "unstaging",
            Self::Discard => "discarding",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitResult {
    pub oid: String,
    pub summary: String,
}

#[derive(Debug)]
pub enum GitReviewError {
    NotRepository(PathBuf),
    CouldNotRunGit {
        operation: &'static str,
        source: io::Error,
    },
    GitFailed {
        operation: &'static str,
        exit_code: Option<i32>,
        message: String,
    },
    TimedOut {
        operation: &'static str,
        timeout: Duration,
    },
    OutputTooLarge {
        operation: &'static str,
        limit: usize,
    },
    InvalidPath {
        path: PathBuf,
        reason: &'static str,
    },
    EmptySelection,
    EmptyPatch,
    InvalidPatch {
        reason: &'static str,
    },
    PatchTooLarge {
        size: usize,
        limit: usize,
    },
    PatchDoesNotApply {
        mutation: PatchMutation,
        message: String,
    },
    EmptyCommitMessage,
    CommitMessageTooLarge {
        size: usize,
        limit: usize,
    },
    MalformedStatus(String),
}

impl fmt::Display for GitReviewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRepository(path) => write!(
                formatter,
                "{} is not inside a non-bare Git repository",
                path.display()
            ),
            Self::CouldNotRunGit { operation, source } => {
                write!(formatter, "could not run Git while {operation}: {source}")
            }
            Self::GitFailed {
                operation,
                exit_code,
                message,
            } => {
                let exit = exit_code
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "signal".to_owned());
                if message.is_empty() {
                    write!(formatter, "Git failed while {operation} (exit {exit})")
                } else {
                    write!(
                        formatter,
                        "Git failed while {operation} (exit {exit}): {message}"
                    )
                }
            }
            Self::TimedOut { operation, timeout } => write!(
                formatter,
                "Git timed out after {:.0}s while {operation}",
                timeout.as_secs_f32()
            ),
            Self::OutputTooLarge { operation, limit } => write!(
                formatter,
                "Git produced more than {limit} bytes while {operation}"
            ),
            Self::InvalidPath { path, reason } => {
                write!(
                    formatter,
                    "unsafe repository path {}: {reason}",
                    path.display()
                )
            }
            Self::EmptySelection => formatter.write_str("select at least one changed path"),
            Self::EmptyPatch => formatter.write_str("review patch cannot be empty"),
            Self::InvalidPatch { reason } => write!(formatter, "invalid review patch: {reason}"),
            Self::PatchTooLarge { size, limit } => {
                write!(formatter, "review patch is {size} bytes (limit {limit})")
            }
            Self::PatchDoesNotApply { mutation, message } => {
                write!(
                    formatter,
                    "review patch is stale or no longer applies while {}: {message}",
                    mutation.label()
                )
            }
            Self::EmptyCommitMessage => formatter.write_str("commit message cannot be empty"),
            Self::CommitMessageTooLarge { size, limit } => {
                write!(formatter, "commit message is {size} bytes (limit {limit})")
            }
            Self::MalformedStatus(message) => {
                write!(formatter, "Git returned malformed status data: {message}")
            }
        }
    }
}

impl std::error::Error for GitReviewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CouldNotRunGit { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl GitRepository {
    /// Resolves the worktree root containing `cwd`.
    pub fn discover(cwd: &Path) -> Result<Self, GitReviewError> {
        let output = run_git(
            cwd,
            ["rev-parse", "--show-toplevel"],
            None,
            "finding repository",
        )?;
        if !output.status.success() {
            let message = output.stderr_message();
            if message.contains("not a git repository") || message.contains("not a git work tree") {
                return Err(GitReviewError::NotRepository(cwd.to_path_buf()));
            }
            return Err(output.failure("finding repository"));
        }

        let root = path_from_output_line(&output.stdout);
        if root.as_os_str().is_empty() {
            return Err(GitReviewError::NotRepository(cwd.to_path_buf()));
        }
        let bare = run_git(
            &root,
            ["rev-parse", "--is-bare-repository"],
            None,
            "checking repository",
        )?;
        let bare = ensure_success(bare, "checking repository")?;
        if String::from_utf8_lossy(&bare.stdout).trim() == "true" {
            return Err(GitReviewError::NotRepository(cwd.to_path_buf()));
        }

        Ok(Self { root })
    }

    /// Reads porcelain-v2 status. Conflicts are reported only in
    /// `conflicted`; an ordinary path can appear in both `staged` and
    /// `unstaged` when its index and worktree versions both changed.
    pub fn status(&self) -> Result<ReviewStatus, GitReviewError> {
        let output = run_git(
            &self.root,
            [
                "status",
                "--porcelain=v2",
                "--branch",
                "-z",
                "--untracked-files=all",
                "--ignore-submodules=none",
            ],
            None,
            "reading status",
        )?;
        ensure_success(output, "reading status")
            .and_then(|output| parse_status(&self.root, &output.stdout))
    }

    /// Every requested path, plus the source of any staged rename whose
    /// destination was requested. Order is preserved and sources are appended
    /// without duplicating a path the caller already named.
    fn with_rename_sources(&self, paths: &[PathBuf]) -> Result<Vec<PathBuf>, GitReviewError> {
        let mut resolved = paths.to_vec();
        let renames: Vec<_> = self
            .status()?
            .staged
            .into_iter()
            .filter(|change| matches!(change.kind, ChangeKind::Renamed | ChangeKind::Copied))
            .filter_map(|change| change.original_path.map(|source| (change.path, source)))
            .collect();
        for (destination, source) in renames {
            if paths.iter().any(|path| path == &destination) && !resolved.contains(&source) {
                resolved.push(source);
            }
        }
        Ok(resolved)
    }

    /// Stages the complete current contents of each path, including deletions.
    pub fn stage_paths(&self, paths: &[PathBuf]) -> Result<(), GitReviewError> {
        let paths = validate_paths(paths)?;
        let mut args = literal_path_command(["add"]);
        args.push(OsString::from("--"));
        args.extend(paths);
        self.run_mutation(args, "staging paths")
    }

    /// Restores paths in the index from HEAD. On an unborn branch, removes
    /// them from the index while preserving their worktree contents.
    ///
    /// A staged rename occupies two index entries, and callers only know the
    /// destination — the diff row and the file list both name the new path.
    /// Resetting that path alone leaves the source staged as a deletion, so
    /// the next commit removes the file's content entirely. Rename sources are
    /// resolved here, where the porcelain format is already understood, rather
    /// than asking every caller to carry them.
    pub fn unstage_paths(&self, paths: &[PathBuf]) -> Result<(), GitReviewError> {
        let paths = self.with_rename_sources(paths)?;
        let paths = validate_paths(&paths)?;
        let head = run_git(
            &self.root,
            ["rev-parse", "--verify", "--quiet", "HEAD"],
            None,
            "checking HEAD",
        )?;
        let mut args = if head.status.success() {
            literal_path_command(["reset", "--quiet", "HEAD"])
        } else if head.status.code() == Some(1) {
            literal_path_command(["rm", "--quiet", "-r", "-f", "--cached", "--ignore-unmatch"])
        } else {
            return Err(head.failure("checking HEAD"));
        };
        args.push(OsString::from("--"));
        args.extend(paths);
        self.run_mutation(args, "unstaging paths")
    }

    /// Discards only unstaged tracked changes, restoring from the index.
    /// Untracked files are never deleted by this method.
    pub fn discard_unstaged(&self, paths: &[PathBuf]) -> Result<(), GitReviewError> {
        let paths = validate_paths(paths)?;
        let mut args = literal_path_command(["restore", "--worktree"]);
        args.push(OsString::from("--"));
        args.extend(paths);
        self.run_mutation(args, "discarding unstaged changes")
    }

    /// Applies one or more complete unified-diff hunks as a single mutation.
    ///
    /// The patch is preflighted against the current repository state. A hunk
    /// loaded before an overlapping external edit therefore returns
    /// [`GitReviewError::PatchDoesNotApply`] without changing the index or
    /// worktree. `Discard` never receives `--cached`, so callers must only pass
    /// patches from the tracked working-tree lane.
    pub fn apply_patch(&self, patch: &[u8], mutation: PatchMutation) -> Result<(), GitReviewError> {
        if patch.iter().all(u8::is_ascii_whitespace) {
            return Err(GitReviewError::EmptyPatch);
        }
        if patch.len() > MAX_PATCH_BYTES {
            return Err(GitReviewError::PatchTooLarge {
                size: patch.len(),
                limit: MAX_PATCH_BYTES,
            });
        }
        if patch.contains(&0) {
            return Err(GitReviewError::InvalidPatch {
                reason: "patch contains a NUL byte",
            });
        }
        if mutation == PatchMutation::Discard && patch_creates_file(patch) {
            return Err(GitReviewError::InvalidPatch {
                reason: "discard cannot delete an untracked or newly added file",
            });
        }

        let mut options = vec!["apply", "--recount", "--whitespace=nowarn"];
        match mutation {
            PatchMutation::Stage => options.push("--cached"),
            PatchMutation::Unstage => {
                options.push("--cached");
                options.push("--reverse");
            }
            PatchMutation::Discard => options.push("--reverse"),
        }

        if mutation == PatchMutation::Stage {
            // `--cached --check` only validates the patch's preimage against
            // the index. Also reverse-check its postimage against the live
            // worktree so a snapshot cannot stage content that has since been
            // edited again. Unrelated hunks remain valid and can still stage.
            let worktree_check = run_git(
                &self.root,
                [
                    "apply",
                    "--recount",
                    "--whitespace=nowarn",
                    "--reverse",
                    "--check",
                ],
                Some(patch),
                "checking review patch against worktree",
            )?;
            if !worktree_check.status.success() {
                return Err(patch_rejected(worktree_check, mutation));
            }
        }

        let mut check_options = options.clone();
        check_options.push("--check");
        let preflight = run_git(
            &self.root,
            check_options,
            Some(patch),
            "checking review patch",
        )?;
        if !preflight.status.success() {
            return Err(patch_rejected(preflight, mutation));
        }

        let applied = run_git(&self.root, options, Some(patch), "applying review patch")?;
        if !applied.status.success() {
            return Err(patch_rejected(applied, mutation));
        }
        Ok(())
    }

    /// Creates a commit from the current index with a validated message.
    /// Editor, signing, and hooks are disabled so this operation cannot prompt
    /// outside the cockpit; the repository's index is otherwise respected.
    pub fn commit(&self, message: &str) -> Result<CommitResult, GitReviewError> {
        if message.trim().is_empty() {
            return Err(GitReviewError::EmptyCommitMessage);
        }
        if message.len() > MAX_COMMIT_MESSAGE_BYTES {
            return Err(GitReviewError::CommitMessageTooLarge {
                size: message.len(),
                limit: MAX_COMMIT_MESSAGE_BYTES,
            });
        }
        if message.as_bytes().contains(&0) {
            return Err(GitReviewError::GitFailed {
                operation: "validating commit message",
                exit_code: None,
                message: "commit message contains a NUL byte".to_owned(),
            });
        }

        let output = run_git(
            &self.root,
            [
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "commit.gpgSign=false",
                "commit",
                "--quiet",
                "--file=-",
                "--cleanup=strip",
            ],
            Some(message.as_bytes()),
            "creating commit",
        )?;
        ensure_success(output, "creating commit")?;

        let identity = run_git(
            &self.root,
            ["show", "-s", "--format=%H%x00%s", "HEAD"],
            None,
            "reading new commit",
        )?;
        let identity = ensure_success(identity, "reading new commit")?;
        let identity = trim_line_ending(&identity.stdout);
        let separator = identity.iter().position(|byte| *byte == 0).ok_or_else(|| {
            GitReviewError::MalformedStatus("new commit identity has no separator".to_owned())
        })?;
        let (oid, summary) = (&identity[..separator], &identity[separator + 1..]);
        let oid = String::from_utf8_lossy(oid).into_owned();
        if oid.is_empty() {
            return Err(GitReviewError::MalformedStatus(
                "new commit object id is empty".to_owned(),
            ));
        }

        Ok(CommitResult {
            oid,
            summary: String::from_utf8_lossy(summary).into_owned(),
        })
    }

    fn run_mutation(
        &self,
        args: Vec<OsString>,
        operation: &'static str,
    ) -> Result<(), GitReviewError> {
        let output = run_git(&self.root, args, None, operation)?;
        ensure_success(output, operation).map(|_| ())
    }
}
