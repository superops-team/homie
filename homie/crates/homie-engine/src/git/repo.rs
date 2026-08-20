use std::path::{Path, PathBuf};

use super::run;

/// The current branch for a working directory: a branch name, a short SHA when
/// HEAD is detached, or `None` outside a repository.
pub fn branch(cwd: &Path) -> Option<String> {
    let git_dir = git_dir(cwd)?;
    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let trimmed = head.trim();

    if let Some(reference) = trimmed.strip_prefix("ref: ") {
        return match reference.split_once("refs/heads/") {
            Some((_, name)) => Some(name.to_string()),
            None => reference.rsplit('/').next().map(str::to_string),
        };
    }
    // Detached HEAD: a raw object id.
    Some(trimmed.chars().take(8).collect())
}

/// True when `cwd` is inside a *linked* worktree rather than the main checkout.
///
/// The signal is what `.git` is: a directory in the main checkout, a file
/// carrying `gitdir:` indirection in a linked one. This is what distinguishes
/// an agent's own worktree from the primary tree.
pub fn is_linked_worktree(cwd: &Path) -> bool {
    let mut dir = cwd.to_path_buf();
    loop {
        let dot_git = dir.join(".git");
        if let Ok(metadata) = std::fs::metadata(&dot_git) {
            return !metadata.is_dir();
        }
        if !dir.pop() {
            return false;
        }
    }
}

/// Resolves the directory holding `HEAD`, following worktree indirection.
fn git_dir(cwd: &Path) -> Option<PathBuf> {
    let mut dir = cwd.to_path_buf();
    loop {
        let dot_git = dir.join(".git");
        if let Ok(metadata) = std::fs::metadata(&dot_git) {
            if metadata.is_dir() {
                return Some(dot_git);
            }
            // `.git` is a file: "gitdir: <path>".
            let contents = std::fs::read_to_string(&dot_git).ok()?;
            let line = contents.lines().next()?;
            let target = line.strip_prefix("gitdir: ")?.trim();
            let resolved = if Path::new(target).is_absolute() {
                PathBuf::from(target)
            } else {
                dir.join(target)
            };
            return Some(resolved);
        }
        if !dir.pop() {
            return None;
        }
    }
}

pub fn is_repository(path: &Path) -> bool {
    git_dir(path).is_some()
}

/// The repository root for `path`.
pub fn repository_root(path: &Path) -> Option<String> {
    let output = run(&["rev-parse", "--show-toplevel"], path).ok()?;
    let trimmed = output.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}
