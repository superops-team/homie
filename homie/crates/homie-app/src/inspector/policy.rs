//! Pure decision policy for the inspector.
//!
//! Git error classification and loading-gate decisions that sit below the GPUI
//! view. No `Window`/`Context`/`Entity`/render dependency.

use crate::inspector::state::LoadState;

pub(crate) fn git_is_not_a_repository(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("not a git repository")
        || error.contains("session cwd is not inside a git repository")
}

pub(crate) fn git_is_not_installed(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("git is not installed")
        || error.contains("git: command not found")
        || error.contains("git: not found")
}

pub(crate) fn should_show_blocking_git_loading(context_changed: bool, state: &LoadState) -> bool {
    context_changed || matches!(state, LoadState::NoSession)
}
