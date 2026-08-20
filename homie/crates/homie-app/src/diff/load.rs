use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;

use homie_proto::SessionDiffBase;

#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

use super::parse::parse_unified_diff_bytes;
use super::{
    DiffError, DiffLayer, DiffRow, DiffRowKind, DiffSnapshot, LocalDiffSource, MAX_DIFF_BYTES,
    MAX_UNTRACKED_FILES,
};

pub(super) fn discover_repository(cwd: &Path) -> Result<PathBuf, DiffError> {
    let root_output = git(cwd, ["rev-parse", "--show-toplevel"])?;
    if !root_output.status.success() {
        return Err(DiffError::NotRepository);
    }
    let repo_root = PathBuf::from(String::from_utf8_lossy(&root_output.stdout).trim());
    if repo_root.as_os_str().is_empty() {
        return Err(DiffError::NotRepository);
    }
    Ok(repo_root)
}

pub(super) fn load_diff_from_repository(
    repo_root: &Path,
    layer: DiffLayer,
    source: LocalDiffSource,
) -> Result<DiffSnapshot, DiffError> {
    let has_head = git(repo_root, ["rev-parse", "--verify", "HEAD"])
        .is_ok_and(|output| output.status.success());
    let mut patch = Vec::new();
    let mut base_ref = None;
    match source {
        LocalDiffSource::DefaultBranch if has_head => {
            let resolution = resolve_comparison(repo_root, SessionDiffBase::DefaultBranch)?;
            base_ref = Some(resolution.label);
            append_output(
                &mut patch,
                git(
                    repo_root,
                    [
                        "diff",
                        "--no-ext-diff",
                        "--no-color",
                        "--unified=3",
                        resolution.commit.as_str(),
                        "--",
                    ],
                )?,
            )?;
        }
        LocalDiffSource::Head if has_head => {
            base_ref = Some("HEAD".to_owned());
            append_output(
                &mut patch,
                git(
                    repo_root,
                    [
                        "diff",
                        "--no-ext-diff",
                        "--no-color",
                        "--unified=3",
                        "HEAD",
                        "--",
                    ],
                )?,
            )?;
        }
        LocalDiffSource::Staged => {
            if has_head {
                base_ref = Some("HEAD".to_owned());
                append_output(
                    &mut patch,
                    git(
                        repo_root,
                        [
                            "diff",
                            "--no-ext-diff",
                            "--no-color",
                            "--unified=3",
                            "--cached",
                            "HEAD",
                            "--",
                        ],
                    )?,
                )?;
            } else {
                append_cached_diff(repo_root, &mut patch)?;
            }
        }
        LocalDiffSource::Working => append_working_diff(repo_root, &mut patch)?,
        LocalDiffSource::DefaultBranch | LocalDiffSource::Head => {
            // An unborn branch has no comparison commit. Preserve the existing
            // combined overview by concatenating index and worktree lanes.
            append_cached_diff(repo_root, &mut patch)?;
            append_working_diff(repo_root, &mut patch)?;
        }
    }

    if matches!(
        source,
        LocalDiffSource::DefaultBranch | LocalDiffSource::Head | LocalDiffSource::Working
    ) {
        append_untracked_diffs(repo_root, &mut patch)?;
    }

    let truncated = patch.len() > MAX_DIFF_BYTES;
    patch.truncate(MAX_DIFF_BYTES);
    let mut snapshot = parse_unified_diff_bytes(&patch);
    snapshot.repo_root = repo_root.to_path_buf();
    snapshot.base_ref = base_ref;
    snapshot.layer = layer;
    snapshot.truncated = truncated;
    if truncated {
        snapshot.rows.push(DiffRow {
            kind: DiffRowKind::Meta,
            old_line: None,
            new_line: None,
            text: "Diff truncated at 16 MB".to_owned(),
        });
    }
    Ok(snapshot)
}

fn append_cached_diff(repo_root: &Path, patch: &mut Vec<u8>) -> Result<(), DiffError> {
    append_output(
        patch,
        git(
            repo_root,
            [
                "diff",
                "--no-ext-diff",
                "--no-color",
                "--unified=3",
                "--cached",
                "--",
            ],
        )?,
    )
}

fn append_working_diff(repo_root: &Path, patch: &mut Vec<u8>) -> Result<(), DiffError> {
    append_output(
        patch,
        git(
            repo_root,
            ["diff", "--no-ext-diff", "--no-color", "--unified=3", "--"],
        )?,
    )
}

fn append_untracked_diffs(repo_root: &Path, patch: &mut Vec<u8>) -> Result<(), DiffError> {
    let untracked = git(
        repo_root,
        ["ls-files", "--others", "--exclude-standard", "-z"],
    )?;
    if !untracked.status.success() {
        return Err(git_failure(&untracked));
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
        let output = git_command(repo_root)
            .args([
                "diff",
                "--no-index",
                "--no-color",
                "--unified=3",
                "--",
                "/dev/null",
            ])
            .arg(path)
            .output()
            .map_err(|error| DiffError::Git(error.to_string()))?;
        // `git diff --no-index` returns 1 when it found a difference.
        if !output.status.success() && output.status.code() != Some(1) {
            return Err(git_failure(&output));
        }
        append_bytes(patch, &output.stdout);
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ComparisonResolution {
    label: String,
    commit: String,
}

fn resolve_comparison(
    repo_root: &Path,
    comparison: SessionDiffBase,
) -> Result<ComparisonResolution, DiffError> {
    if comparison == SessionDiffBase::Head {
        return Ok(ComparisonResolution {
            label: "HEAD".to_owned(),
            commit: "HEAD".to_owned(),
        });
    }

    if let Some(base_ref) = resolve_default_branch_ref(repo_root) {
        let merge_base = git(repo_root, ["merge-base", base_ref.as_str(), "HEAD"])?;
        if merge_base.status.success() {
            let commit = String::from_utf8_lossy(&merge_base.stdout)
                .trim()
                .to_owned();
            if !commit.is_empty() {
                return Ok(ComparisonResolution {
                    label: base_ref,
                    commit,
                });
            }
        }
    }

    Ok(ComparisonResolution {
        label: "HEAD".to_owned(),
        commit: "HEAD".to_owned(),
    })
}

fn resolve_default_branch_ref(repo_root: &Path) -> Option<String> {
    let origin_head = git(
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
    .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
    .filter(|candidate| !candidate.is_empty());

    origin_head
        .into_iter()
        .chain(
            ["origin/main", "main", "origin/master", "master"]
                .into_iter()
                .map(str::to_owned),
        )
        .find(|candidate| {
            let peeled = format!("{candidate}^{{commit}}");
            git(
                repo_root,
                ["rev-parse", "--verify", "--quiet", peeled.as_str()],
            )
            .is_ok_and(|output| output.status.success())
        })
}

fn git<I, S>(cwd: &Path, args: I) -> Result<std::process::Output, DiffError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    git_command(cwd)
        .args(args)
        .output()
        .map_err(|error| DiffError::Git(error.to_string()))
}

pub(super) fn git_command(cwd: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .current_dir(cwd)
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .env("LANGUAGE", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0");
    command
}

fn append_output(patch: &mut Vec<u8>, output: std::process::Output) -> Result<(), DiffError> {
    if !output.status.success() {
        return Err(git_failure(&output));
    }
    append_bytes(patch, &output.stdout);
    Ok(())
}

fn append_bytes(patch: &mut Vec<u8>, bytes: &[u8]) {
    let remaining = MAX_DIFF_BYTES.saturating_add(1).saturating_sub(patch.len());
    patch.extend_from_slice(&bytes[..bytes.len().min(remaining)]);
}

fn git_failure(output: &std::process::Output) -> DiffError {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    DiffError::Git(if stderr.is_empty() {
        format!("git exited with {}", output.status)
    } else {
        stderr
    })
}
