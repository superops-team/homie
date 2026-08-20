use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use homie_proto::HostEntry;

use crate::remote::manager::RemoteManager;

/// The response stays below the NDJSON ceiling after base64 encoding.
const MAX_PATCH_BYTES: usize = 2 * 1024 * 1024;
const MAX_DIFF_RESPONSE_BYTES: usize = MAX_PATCH_BYTES + 16 * 1024;
const REMOTE_GIT_TIMEOUT: Duration = Duration::from_secs(30);
const DIFF_RESPONSE_MARKER: &[u8] = b"HOMIE_GIT_V1\0";

/// Dynamic values arrive on stdin, leaving the SSH command itself fixed and
/// safe to pass through any account login shell. Session paths containing a
/// newline are rejected before this script is reached.
pub(crate) const WORKING_DIFF_SCRIPT: &str = r#"set -e
export LC_ALL=C LANG=C LANGUAGE=C
IFS= read -r cwd
IFS= read -r comparison
case "$cwd" in
  "~") cwd=$HOME ;;
  "~/"*) cwd=$HOME/${cwd#\~/} ;;
esac
cd "$cwd"
command -v git >/dev/null 2>&1 || { printf '%s\n' 'git is not installed on this host' >&2; exit 127; }
export GIT_TERMINAL_PROMPT=0 GIT_OPTIONAL_LOCKS=0
root=$(git rev-parse --show-toplevel)
cd "$root"
git status --porcelain=v1 -uno >/dev/null
base_ref=HEAD
base_commit=
if git rev-parse --verify HEAD >/dev/null 2>&1; then
  if [ "$comparison" = head ]; then
    base_commit=HEAD
  else
    origin_head=$(git symbolic-ref --quiet --short refs/remotes/origin/HEAD 2>/dev/null || true)
    base_ref=
    for candidate in "$origin_head" origin/main main origin/master master; do
      [ -n "$candidate" ] || continue
      if git rev-parse --verify --quiet "${candidate}^{commit}" >/dev/null; then
        base_ref="$candidate"
        break
      fi
    done
    if [ -n "$base_ref" ]; then
      base_commit=$(git merge-base "$base_ref" HEAD 2>/dev/null || true)
    fi
    if [ -z "$base_commit" ]; then
      base_ref=HEAD
      base_commit=HEAD
    fi
  fi
fi
printf 'HOMIE_GIT_V1\0%s\0%s\0' "$root" "$base_ref"
(
  if [ -n "$base_commit" ]; then
    git diff --no-ext-diff --no-color --unified=3 "$base_commit" --
  else
    git diff --no-ext-diff --no-color --unified=3 --cached --
    git diff --no-ext-diff --no-color --unified=3 --
  fi
  git ls-files --others --exclude-standard -z | \
    xargs -0 -n 1 sh -c '
      [ "$#" -eq 0 ] && exit 0
      git diff --no-index --no-color --unified=3 -- /dev/null "$1"
      code=$?
      [ "$code" -eq 0 ] || [ "$code" -eq 1 ]
    ' sh
) | head -c 2097153
"#;

/// The working tree's diff for a session cwd, against the repository's
/// primary branch (merge-base) or plain HEAD.
///
/// One shell round trip, ported verbatim from `WorktreeDiffLoader`: validate
/// the cwd, emit `root\0base_ref\0`, then stream tracked + staged + untracked
/// changes through a hard byte cap. `xargs -0` keeps spaces and newlines in
/// untracked filenames intact.
pub fn working_diff(
    cwd: &Path,
    base: Option<&homie_proto::SessionDiffBase>,
) -> std::io::Result<homie_proto::SessionReadDiffResult> {
    let input = working_diff_input(&cwd.to_string_lossy(), base)?;
    let mut child = Command::new("/bin/sh")
        .args(["-c", WORKING_DIFF_SCRIPT])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    use std::io::Write as _;
    child
        .stdin
        .take()
        .ok_or_else(|| std::io::Error::other("git diff stdin is unavailable"))?
        .write_all(&input)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(diff_failure(&output.stderr));
    }
    parse_working_diff(output.stdout)
}

pub fn working_diff_remote(
    manager: &RemoteManager,
    host: &HostEntry,
    cwd: &str,
    base: Option<&homie_proto::SessionDiffBase>,
) -> std::io::Result<homie_proto::SessionReadDiffResult> {
    let input = working_diff_input(cwd, base)?;
    let output = manager.run_fixed_script(
        host,
        WORKING_DIFF_SCRIPT,
        input,
        REMOTE_GIT_TIMEOUT,
        MAX_DIFF_RESPONSE_BYTES,
    )?;
    if output.stdout_truncated {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "remote Git response exceeded its hard limit",
        ));
    }
    if !output.status.success() {
        return Err(diff_failure(&output.stderr));
    }
    parse_working_diff(output.stdout)
}

fn working_diff_input(
    cwd: &str,
    base: Option<&homie_proto::SessionDiffBase>,
) -> std::io::Result<Vec<u8>> {
    if cwd.is_empty() || cwd.as_bytes().contains(&0) || cwd.contains(['\r', '\n']) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "session cwd is not representable by the Git inspector",
        ));
    }
    let comparison = match base {
        Some(homie_proto::SessionDiffBase::Head) => "head",
        _ => "defaultBranch",
    };
    Ok(format!("{cwd}\n{comparison}\n").into_bytes())
}

pub(crate) fn parse_working_diff(
    stdout: Vec<u8>,
) -> std::io::Result<homie_proto::SessionReadDiffResult> {
    let marker = stdout
        .windows(DIFF_RESPONSE_MARKER.len())
        .position(|window| window == DIFF_RESPONSE_MARKER)
        .ok_or_else(|| std::io::Error::other("git diff response marker is missing"))?;
    let payload = &stdout[marker + DIFF_RESPONSE_MARKER.len()..];
    let root_end = payload
        .iter()
        .position(|&byte| byte == 0)
        .ok_or_else(|| std::io::Error::other("git diff returned an invalid response"))?;
    let base_end = payload[root_end + 1..]
        .iter()
        .position(|&byte| byte == 0)
        .map(|offset| root_end + 1 + offset)
        .ok_or_else(|| std::io::Error::other("git diff returned an invalid response"))?;
    let repo_root = String::from_utf8_lossy(&payload[..root_end]).into_owned();
    let base_ref = String::from_utf8_lossy(&payload[root_end + 1..base_end]).into_owned();
    let patch = &payload[base_end + 1..];
    let truncated = patch.len() > MAX_PATCH_BYTES;
    Ok(homie_proto::SessionReadDiffResult {
        patch: patch[..patch.len().min(MAX_PATCH_BYTES)].to_vec(),
        repo_root,
        truncated,
        base_ref: Some(base_ref),
    })
}

fn diff_failure(stderr: &[u8]) -> std::io::Error {
    let detail = String::from_utf8_lossy(stderr).trim().to_string();
    std::io::Error::other(if detail.is_empty() {
        "session cwd is not inside a Git repository".to_string()
    } else {
        detail
    })
}
