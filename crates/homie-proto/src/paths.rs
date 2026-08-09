use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RuntimeEndpoint(PathBuf);

impl RuntimeEndpoint {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, RuntimePathError> {
        let path = path.into();
        if !path.is_absolute() {
            return Err(RuntimePathError::EndpointMustBeAbsolute);
        }
        Ok(Self(path))
    }

    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimePaths {
    pub data_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub socket: PathBuf,
    pub lock: PathBuf,
    pub boot_log: PathBuf,
    pub daemon_log: PathBuf,
}

impl RuntimePaths {
    pub fn new(data_dir: impl AsRef<Path>) -> Result<Self, RuntimePathError> {
        let data_dir = data_dir.as_ref();
        if !data_dir.is_absolute() {
            return Err(RuntimePathError::DataDirMustBeAbsolute);
        }
        let data_dir =
            fs::canonicalize(data_dir).map_err(|_| RuntimePathError::DataDirUnavailable)?;
        if !data_dir.is_dir() {
            return Err(RuntimePathError::DataDirUnavailable);
        }

        let runtime_dir = data_dir.join("runtime");
        Ok(Self {
            socket: runtime_dir.join("daemon.sock"),
            lock: runtime_dir.join("daemon.lock"),
            boot_log: runtime_dir.join("daemon.boot.log"),
            daemon_log: runtime_dir.join("daemon.log"),
            runtime_dir,
            data_dir,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RuntimePathError {
    #[error("runtime data directory must be absolute")]
    DataDirMustBeAbsolute,
    #[error("runtime data directory is unavailable")]
    DataDirUnavailable,
    #[error("runtime endpoint must be absolute")]
    EndpointMustBeAbsolute,
}
