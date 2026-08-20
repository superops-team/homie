use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::GitReviewError;

const GIT_TIMEOUT: Duration = Duration::from_secs(20);
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const MAX_STDOUT_BYTES: usize = 8 * 1024 * 1024;
const MAX_STDERR_BYTES: usize = 128 * 1024;

pub(super) struct GitOutput {
    pub(super) status: ExitStatus,
    pub(super) stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl GitOutput {
    pub(super) fn stderr_message(&self) -> String {
        String::from_utf8_lossy(&self.stderr).trim().to_owned()
    }

    pub(super) fn failure(&self, operation: &'static str) -> GitReviewError {
        GitReviewError::GitFailed {
            operation,
            exit_code: self.status.code(),
            message: self.stderr_message(),
        }
    }
}

pub(super) fn ensure_success(
    output: GitOutput,
    operation: &'static str,
) -> Result<GitOutput, GitReviewError> {
    if output.status.success() {
        Ok(output)
    } else {
        Err(output.failure(operation))
    }
}

pub(super) fn run_git<I, S>(
    cwd: &Path,
    args: I,
    input: Option<&[u8]>,
    operation: &'static str,
) -> Result<GitOutput, GitReviewError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new("git");
    command
        .current_dir(cwd)
        .arg("--no-pager")
        .arg("-c")
        .arg("color.ui=false")
        .arg("-c")
        .arg("core.fsmonitor=false")
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_ASKPASS", "true")
        .env("SSH_ASKPASS", "true")
        .env("GCM_INTERACTIVE", "never")
        .env("GIT_EDITOR", "true")
        .env("GIT_SEQUENCE_EDITOR", "true")
        .env("GIT_MERGE_AUTOEDIT", "no")
        .env("LC_ALL", "C")
        .env("GIT_OPTIONAL_LOCKS", "0");

    let mut child = command
        .spawn()
        .map_err(|source| GitReviewError::CouldNotRunGit { operation, source })?;
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let stdout_reader = thread::spawn(move || read_bounded(stdout, MAX_STDOUT_BYTES));
    let stderr_reader = thread::spawn(move || read_bounded(stderr, MAX_STDERR_BYTES));

    let stdin_writer = input.map(|input| {
        let input = input.to_vec();
        let mut stdin = child.stdin.take().expect("piped stdin");
        thread::spawn(move || {
            let result = stdin.write_all(&input);
            drop(stdin);
            result
        })
    });

    let deadline = Instant::now() + GIT_TIMEOUT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(GitReviewError::TimedOut {
                    operation,
                    timeout: GIT_TIMEOUT,
                });
            }
            Err(source) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(GitReviewError::CouldNotRunGit { operation, source });
            }
        }
    };

    if let Some(writer) = stdin_writer {
        match writer.join() {
            Ok(Ok(())) => {}
            // Git may reject input before consuming the complete patch. Its
            // stderr is more useful than the resulting broken pipe.
            Ok(Err(error)) if error.kind() == io::ErrorKind::BrokenPipe => {}
            Ok(Err(source)) => {
                return Err(GitReviewError::CouldNotRunGit { operation, source });
            }
            Err(_) => {
                return Err(GitReviewError::CouldNotRunGit {
                    operation,
                    source: io::Error::other("Git stdin writer panicked"),
                });
            }
        }
    }

    let (stdout, stdout_truncated) = join_reader(stdout_reader, operation)?;
    let (stderr, stderr_truncated) = join_reader(stderr_reader, operation)?;
    if stdout_truncated {
        return Err(GitReviewError::OutputTooLarge {
            operation,
            limit: MAX_STDOUT_BYTES,
        });
    }
    let stderr = if stderr_truncated {
        let mut stderr = stderr;
        stderr.extend_from_slice(b"\n[stderr truncated]");
        stderr
    } else {
        stderr
    };

    Ok(GitOutput {
        status,
        stdout,
        stderr,
    })
}

fn read_bounded<R: Read>(mut reader: R, limit: usize) -> io::Result<(Vec<u8>, bool)> {
    let mut bytes = Vec::with_capacity(limit.min(64 * 1024));
    let mut truncated = false;
    let mut chunk = [0_u8; 16 * 1024];
    loop {
        let count = reader.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        let remaining = limit.saturating_sub(bytes.len());
        bytes.extend_from_slice(&chunk[..count.min(remaining)]);
        truncated |= count > remaining;
    }
    Ok((bytes, truncated))
}

fn join_reader(
    reader: thread::JoinHandle<io::Result<(Vec<u8>, bool)>>,
    operation: &'static str,
) -> Result<(Vec<u8>, bool), GitReviewError> {
    match reader.join() {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(source)) => Err(GitReviewError::CouldNotRunGit { operation, source }),
        Err(_) => Err(GitReviewError::CouldNotRunGit {
            operation,
            source: io::Error::other("Git output reader panicked"),
        }),
    }
}
