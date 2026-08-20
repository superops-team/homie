//! Path validation and patch-shape helpers for Git review mutations.
//!
//! Every mutation funnels through `validate_paths`, which enforces
//! repository-relative, normalized, non-`.git`, non-NUL paths so a caller can
//! never widen a mutation to escape the worktree or touch Git metadata.

use super::process::GitOutput;
use super::*;

use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

pub(super) fn patch_creates_file(patch: &[u8]) -> bool {
    patch.split(|byte| *byte == b'\n').any(|line| {
        line.strip_suffix(b"\r").unwrap_or(line) == b"--- /dev/null"
            || line.starts_with(b"new file mode ")
    })
}

pub(super) fn patch_rejected(output: GitOutput, mutation: PatchMutation) -> GitReviewError {
    let message = output.stderr_message();
    GitReviewError::PatchDoesNotApply {
        mutation,
        message: if message.is_empty() {
            "Git rejected the selected hunk; refresh the review and try again".to_owned()
        } else {
            message
        },
    }
}

pub(super) fn literal_path_command<const N: usize>(command: [&str; N]) -> Vec<OsString> {
    let mut args = vec![OsString::from("--literal-pathspecs")];
    args.extend(command.into_iter().map(OsString::from));
    args
}

pub(super) fn validate_paths(paths: &[PathBuf]) -> Result<Vec<OsString>, GitReviewError> {
    if paths.is_empty() {
        return Err(GitReviewError::EmptySelection);
    }

    paths
        .iter()
        .map(|path| {
            if path.as_os_str().is_empty() {
                return Err(invalid_path(path, "path is empty"));
            }
            if path.is_absolute() {
                return Err(invalid_path(path, "absolute paths are not accepted"));
            }

            let mut components = path.components();
            let Some(first) = components.next() else {
                return Err(invalid_path(path, "path is empty"));
            };
            let first = match first {
                Component::Normal(first) => first,
                Component::ParentDir => {
                    return Err(invalid_path(path, "parent traversal is not accepted"));
                }
                Component::CurDir => {
                    return Err(invalid_path(path, "current-directory paths are too broad"));
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(invalid_path(path, "absolute paths are not accepted"));
                }
            };
            if os_str_eq_ignore_ascii_case(first, OsStr::new(".git")) {
                return Err(invalid_path(path, "Git metadata cannot be mutated"));
            }
            for component in components {
                if !matches!(component, Component::Normal(_)) {
                    return Err(invalid_path(
                        path,
                        "only normalized repository-relative paths are accepted",
                    ));
                }
            }
            if os_str_contains_nul(path.as_os_str()) {
                return Err(invalid_path(path, "path contains a NUL byte"));
            }
            Ok(path.as_os_str().to_owned())
        })
        .collect()
}

fn invalid_path(path: &Path, reason: &'static str) -> GitReviewError {
    GitReviewError::InvalidPath {
        path: path.to_path_buf(),
        reason,
    }
}

fn os_str_eq_ignore_ascii_case(left: &OsStr, right: &OsStr) -> bool {
    left.to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy())
}

#[cfg(unix)]
fn os_str_contains_nul(value: &OsStr) -> bool {
    value.as_bytes().contains(&0)
}

#[cfg(not(unix))]
fn os_str_contains_nul(value: &OsStr) -> bool {
    value.to_string_lossy().contains('\0')
}
