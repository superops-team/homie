use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use homie_client::{LauncherOptions, RuntimeLauncher};
use homie_proto::paths::RuntimePaths;

pub const DAEMON_EXECUTABLE_NAME: &str = "homie-runtime-daemon";

pub fn resolve_daemon_executable(current_executable: &Path) -> Result<PathBuf, DaemonPathError> {
    if !current_executable.is_absolute() {
        return Err(DaemonPathError::CurrentExecutableMustBeAbsolute);
    }
    let current_executable = fs::canonicalize(current_executable)
        .map_err(DaemonPathError::CurrentExecutableUnavailable)?;
    let parent = current_executable
        .parent()
        .ok_or(DaemonPathError::CurrentExecutableHasNoParent)?;
    let candidate = bundle_contents(parent)
        .map(|contents| {
            contents
                .join("Resources")
                .join("bin")
                .join(DAEMON_EXECUTABLE_NAME)
        })
        .unwrap_or_else(|| parent.join(DAEMON_EXECUTABLE_NAME));
    let daemon = fs::canonicalize(candidate).map_err(DaemonPathError::DaemonUnavailable)?;
    if !daemon.is_absolute() || !daemon.is_file() {
        return Err(DaemonPathError::DaemonIsNotAFile);
    }
    Ok(daemon)
}

fn bundle_contents(executable_parent: &Path) -> Option<&Path> {
    if executable_parent.file_name()? != "MacOS" {
        return None;
    }
    let contents = executable_parent.parent()?;
    if contents.file_name()? != "Contents"
        || contents
            .parent()?
            .extension()
            .and_then(|value| value.to_str())
            != Some("app")
    {
        return None;
    }
    Some(contents)
}

pub async fn ensure_sibling_daemon(
    data_dir: PathBuf,
    current_executable: PathBuf,
    startup_probe_timeout: Duration,
) -> Result<RuntimePaths, DaemonLaunchError> {
    let daemon_executable = resolve_daemon_executable(&current_executable)?;
    RuntimeLauncher::ensure_running(LauncherOptions {
        data_dir,
        daemon_executable,
        startup_probe_timeout,
    })
    .await
    .map_err(DaemonLaunchError::Launcher)
}

#[derive(Debug, thiserror::Error)]
pub enum DaemonPathError {
    #[error("current executable path must be absolute")]
    CurrentExecutableMustBeAbsolute,
    #[error("current executable is unavailable: {0}")]
    CurrentExecutableUnavailable(std::io::Error),
    #[error("current executable has no parent directory")]
    CurrentExecutableHasNoParent,
    #[error("configured homie-runtime-daemon is unavailable: {0}")]
    DaemonUnavailable(std::io::Error),
    #[error("configured homie-runtime-daemon is not a regular file")]
    DaemonIsNotAFile,
}

impl DaemonPathError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        "bad_request"
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DaemonLaunchError {
    #[error(transparent)]
    Path(#[from] DaemonPathError),
    #[error(transparent)]
    Launcher(homie_client::LauncherError),
}

impl DaemonLaunchError {
    #[must_use]
    pub fn code(&self) -> &str {
        match self {
            Self::Path(error) => error.code(),
            Self::Launcher(error) => error.code(),
        }
    }
}
