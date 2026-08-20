//! Worktree diff loading and unified-patch parsing.
//!
//! The inspector crosses this module through one interface: a session cwd in,
//! a flat render snapshot out. Git process details, untracked files, hunk line
//! accounting, and output limits stay local to the implementation.

use std::fmt;
use std::ops::Range;
use std::path::{Path, PathBuf};

use homie_proto::{SessionDiffBase, SessionReadDiffResult};

pub(super) const MAX_DIFF_BYTES: usize = 16 * 1024 * 1024;
pub(super) const MAX_UNTRACKED_FILES: usize = 200;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DiffLayer {
    #[default]
    Branch,
    Staged,
    Working,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffRowKind {
    File,
    Hunk,
    Context,
    Addition,
    Deletion,
    Meta,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffRow {
    pub kind: DiffRowKind,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub text: String,
}

/// One changed file and the semantic hunks contained by its visible rows.
///
/// `row_range` uses the same indices as [`DiffSnapshot::rows`]. Paths are
/// repository-relative for locally loaded diffs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffFile {
    pub path: PathBuf,
    pub row_range: Range<usize>,
    pub additions: usize,
    pub deletions: usize,
    pub hunks: Vec<DiffHunk>,
}

/// A whole, independently applicable unified-diff hunk.
///
/// `patch` repeats the complete file preamble before the selected hunk so it
/// can be sent directly to `git apply`. The fingerprint is deterministic FNV-1a
/// over those exact bytes and is intended for UI identity, not trust.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffHunk {
    pub header: String,
    pub row_range: Range<usize>,
    pub old_start: Option<u32>,
    pub new_start: Option<u32>,
    pub additions: usize,
    pub deletions: usize,
    pub patch: Vec<u8>,
    pub fingerprint: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiffSnapshot {
    pub repo_root: PathBuf,
    pub base_ref: Option<String>,
    pub layer: DiffLayer,
    pub rows: Vec<DiffRow>,
    pub file_diffs: Vec<DiffFile>,
    pub files: usize,
    pub additions: usize,
    pub deletions: usize,
    pub max_text_columns: usize,
    pub truncated: bool,
}

#[derive(Debug)]
pub enum DiffError {
    NotRepository,
    Git(String),
}

impl fmt::Display for DiffError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRepository => {
                formatter.write_str("This session is not inside a Git repository")
            }
            Self::Git(message) => write!(formatter, "Git could not load changes: {message}"),
        }
    }
}

impl std::error::Error for DiffError {}

pub fn load_worktree_diff_against(
    cwd: &Path,
    comparison: SessionDiffBase,
) -> Result<DiffSnapshot, DiffError> {
    if comparison == SessionDiffBase::DefaultBranch {
        return load_local_diff(cwd, DiffLayer::Branch);
    }

    let repo_root = load::discover_repository(cwd)?;
    load::load_diff_from_repository(&repo_root, DiffLayer::Branch, LocalDiffSource::Head)
}

/// Loads one semantic local review lane.
///
/// `Branch` is the combined feature-branch overview against the default branch,
/// including index, worktree, and untracked content. `Staged` is HEAD to index;
/// `Working` is index to worktree plus bounded untracked content. Keeping these
/// lanes separate is what makes the returned hunk patches safe to mutate.
pub fn load_local_diff(cwd: &Path, layer: DiffLayer) -> Result<DiffSnapshot, DiffError> {
    let repo_root = load::discover_repository(cwd)?;
    let source = match layer {
        DiffLayer::Branch => LocalDiffSource::DefaultBranch,
        DiffLayer::Staged => LocalDiffSource::Staged,
        DiffLayer::Working => LocalDiffSource::Working,
    };
    load::load_diff_from_repository(&repo_root, layer, source)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LocalDiffSource {
    DefaultBranch,
    Head,
    Staged,
    Working,
}

pub fn parse_unified_diff(patch: &str) -> DiffSnapshot {
    parse::parse_unified_diff_bytes(patch.as_bytes())
}

/// Converts the daemon's bounded wire payload into the same render snapshot
/// used by local Git. Keeping this conversion here guarantees identical row,
/// hunk, and summary behavior for local and remote sessions.
pub fn snapshot_from_read_diff(result: SessionReadDiffResult) -> DiffSnapshot {
    let mut snapshot = parse::parse_unified_diff_bytes(&result.patch);
    snapshot.repo_root = PathBuf::from(result.repo_root);
    snapshot.base_ref = result.base_ref;
    snapshot.layer = DiffLayer::Branch;
    snapshot.truncated = result.truncated;
    if result.truncated {
        snapshot.rows.push(DiffRow {
            kind: DiffRowKind::Meta,
            old_line: None,
            new_line: None,
            text: "Diff truncated by the daemon".to_owned(),
        });
    }
    snapshot
}

mod load;
mod parse;
#[cfg(test)]
mod tests;
