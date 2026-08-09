use std::fs::{self, DirBuilder, OpenOptions};
use std::os::unix::fs::{
    DirBuilderExt as _, MetadataExt as _, OpenOptionsExt as _, PermissionsExt as _,
};
use std::os::unix::process::CommandExt as _;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use homie_proto::ControlMessage;
use homie_proto::paths::{RuntimePathError, RuntimePaths};
use homie_proto::transport::{
    ClientRole, EndpointRole, FRAME_HEADER_LEN, Frame, FrameHeader, FrameKind, HelloRequest,
    HelloResponse, MAX_FRAME_LEN, Preface, WIRE_MAJOR, WIRE_MINOR,
};
use sha2::{Digest as _, Sha256};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::UnixStream;
use tokio::process::Command;

#[derive(Clone, Debug)]
pub struct LauncherOptions {
    pub data_dir: PathBuf,
    pub daemon_executable: PathBuf,
    pub startup_probe_timeout: Duration,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RuntimeLauncher;

impl RuntimeLauncher {
    pub async fn ensure_running(options: LauncherOptions) -> Result<RuntimePaths, LauncherError> {
        prepare_data_directory(&options.data_dir)?;
        let paths = RuntimePaths::new(&options.data_dir)?;
        let daemon_executable = validate_executable(&options.daemon_executable)?;
        prepare_runtime_directory(&paths)?;

        match probe(&paths.socket, options.startup_probe_timeout).await {
            Ok(hello) => {
                let expected_hash = executable_hash(&daemon_executable).await?;
                if hello.executable_hash != expected_hash {
                    return Err(LauncherError::ExecutableHashMismatch);
                }
                Ok(paths)
            }
            Err(ProbeError::EndpointUnavailable) => {
                spawn_daemon(&daemon_executable, &paths)?;
                Ok(paths)
            }
            Err(ProbeError::Fatal(error)) => Err(error),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LauncherError {
    #[error(transparent)]
    RuntimePath(#[from] RuntimePathError),
    #[error("daemon executable path must be absolute")]
    ExecutableMustBeAbsolute,
    #[error("daemon executable is missing or is not a regular file")]
    ExecutableMissing,
    #[error("daemon executable is not executable")]
    ExecutableNotExecutable,
    #[error("runtime directory must be a private directory owned by the current user")]
    UnsafeRuntimeDirectory,
    #[error("runtime socket path must not be a symbolic link")]
    UnsafeRuntimeSocket,
    #[error("boot log must be a regular file owned by the current user with mode 0600")]
    UnsafeBootLog,
    #[error("live daemon executable hash differs from the configured executable")]
    ExecutableHashMismatch,
    #[error("runtime endpoint is unavailable")]
    Unavailable,
    #[error("runtime protocol version mismatch")]
    VersionMismatch,
    #[error("runtime endpoint rejected the launcher probe")]
    Unauthorized,
    #[error("runtime protocol error: {0}")]
    Protocol(String),
    #[error("runtime launcher I/O failed: {0}")]
    Io(#[from] std::io::Error),
}

impl LauncherError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::RuntimePath(_)
            | Self::ExecutableMustBeAbsolute
            | Self::ExecutableMissing
            | Self::ExecutableNotExecutable
            | Self::UnsafeRuntimeDirectory
            | Self::UnsafeRuntimeSocket
            | Self::UnsafeBootLog => "bad_request",
            Self::ExecutableHashMismatch | Self::VersionMismatch => "version_mismatch",
            Self::Unavailable | Self::Io(_) => "unavailable",
            Self::Unauthorized => "unauthorized",
            Self::Protocol(_) => "internal",
        }
    }
}

enum ProbeError {
    EndpointUnavailable,
    Fatal(LauncherError),
}

async fn probe(socket: &Path, timeout: Duration) -> Result<HelloResponse, ProbeError> {
    let stream = tokio::time::timeout(timeout, UnixStream::connect(socket))
        .await
        .map_err(|_| ProbeError::Fatal(LauncherError::Unavailable))?;
    let mut stream = match stream {
        Ok(stream) => stream,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
            ) =>
        {
            return Err(ProbeError::EndpointUnavailable);
        }
        Err(error) => return Err(ProbeError::Fatal(LauncherError::Io(error))),
    };

    let hello = HelloRequest {
        wire_major: WIRE_MAJOR,
        wire_minor: WIRE_MINOR,
        client_name: "homie-runtime-launcher".to_string(),
        client_version: env!("CARGO_PKG_VERSION").to_string(),
        client_role: ClientRole::Cli,
        process_id: std::process::id(),
    };
    let frame = Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Hello,
            flags: 0,
            stream_id: 0,
            message_id: 0,
            sequence: 0,
        },
        payload: serde_json::to_vec(&hello)
            .map_err(|error| ProbeError::Fatal(LauncherError::Protocol(error.to_string())))?,
    };

    let exchange = async {
        stream
            .write_all(
                &Preface {
                    major: WIRE_MAJOR,
                    minor: WIRE_MINOR,
                }
                .encode(),
            )
            .await?;
        stream
            .write_all(
                &frame
                    .encode(EndpointRole::Client)
                    .map_err(transport_io_error)?,
            )
            .await?;
        read_frame(&mut stream).await
    };
    let response = tokio::time::timeout(timeout, exchange)
        .await
        .map_err(|_| ProbeError::Fatal(LauncherError::Unavailable))?
        .map_err(|error| ProbeError::Fatal(LauncherError::Io(error)))?;

    match response.header.kind {
        FrameKind::HelloAck => {
            let hello: HelloResponse = serde_json::from_slice(&response.payload)
                .map_err(|error| ProbeError::Fatal(LauncherError::Protocol(error.to_string())))?;
            if hello.wire_major != WIRE_MAJOR {
                return Err(ProbeError::Fatal(LauncherError::VersionMismatch));
            }
            Ok(hello)
        }
        FrameKind::Response => {
            let response: ControlMessage = serde_json::from_slice(&response.payload)
                .map_err(|error| ProbeError::Fatal(LauncherError::Protocol(error.to_string())))?;
            let ControlMessage::Response { error, .. } = response else {
                return Err(ProbeError::Fatal(LauncherError::Protocol(
                    "launcher probe received a non-response control payload".to_string(),
                )));
            };
            let Some(error) = error else {
                return Err(ProbeError::Fatal(LauncherError::Protocol(
                    "launcher probe received a response without an error".to_string(),
                )));
            };
            Err(ProbeError::Fatal(match error.code.as_str() {
                "version_mismatch" => LauncherError::VersionMismatch,
                "unauthorized" => LauncherError::Unauthorized,
                _ => LauncherError::Protocol("launcher probe was rejected".to_string()),
            }))
        }
        _ => Err(ProbeError::Fatal(LauncherError::Protocol(
            "launcher probe expected HelloAck".to_string(),
        ))),
    }
}

async fn read_frame(stream: &mut UnixStream) -> std::io::Result<Frame> {
    let mut length = [0_u8; 4];
    stream.read_exact(&mut length).await?;
    let frame_len = u32::from_be_bytes(length) as usize;
    if !(FRAME_HEADER_LEN..=MAX_FRAME_LEN).contains(&frame_len) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid runtime frame length",
        ));
    }
    let mut encoded = vec![0_u8; 4 + frame_len];
    encoded[..4].copy_from_slice(&length);
    stream.read_exact(&mut encoded[4..]).await?;
    Frame::decode(&encoded, EndpointRole::Server)
        .map_err(transport_io_error)?
        .map(|(frame, _)| frame)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "incomplete runtime frame",
            )
        })
}

fn prepare_data_directory(data_dir: &Path) -> Result<(), LauncherError> {
    if !data_dir.is_absolute() {
        return Err(RuntimePathError::DataDirMustBeAbsolute.into());
    }
    fs::create_dir_all(data_dir)?;
    Ok(())
}

fn validate_executable(path: &Path) -> Result<PathBuf, LauncherError> {
    if !path.is_absolute() {
        return Err(LauncherError::ExecutableMustBeAbsolute);
    }
    let path = fs::canonicalize(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            LauncherError::ExecutableMissing
        } else {
            LauncherError::Io(error)
        }
    })?;
    let metadata = fs::metadata(&path)?;
    if !metadata.is_file() {
        return Err(LauncherError::ExecutableMissing);
    }
    if metadata.permissions().mode() & 0o111 == 0 {
        return Err(LauncherError::ExecutableNotExecutable);
    }
    Ok(path)
}

fn validate_runtime_directory_attributes(
    is_directory: bool,
    owner_uid: u32,
    mode: u32,
    current_uid: u32,
) -> Result<(), LauncherError> {
    if !is_directory || owner_uid != current_uid || mode & 0o077 != 0 {
        return Err(LauncherError::UnsafeRuntimeDirectory);
    }
    Ok(())
}

fn validate_boot_log_attributes(
    is_file: bool,
    owner_uid: u32,
    mode: u32,
    current_uid: u32,
) -> Result<(), LauncherError> {
    if !is_file || owner_uid != current_uid || mode & 0o7777 != 0o600 {
        return Err(LauncherError::UnsafeBootLog);
    }
    Ok(())
}

fn prepare_runtime_directory(paths: &RuntimePaths) -> Result<(), LauncherError> {
    let metadata = match fs::symlink_metadata(&paths.runtime_dir) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            DirBuilder::new().mode(0o700).create(&paths.runtime_dir)?;
            fs::symlink_metadata(&paths.runtime_dir)?
        }
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() {
        return Err(LauncherError::UnsafeRuntimeDirectory);
    }
    validate_runtime_directory_attributes(
        metadata.is_dir(),
        metadata.uid(),
        metadata.mode(),
        current_uid(),
    )?;
    match fs::symlink_metadata(&paths.socket) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(LauncherError::UnsafeRuntimeSocket);
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

fn current_uid() -> u32 {
    // SAFETY: geteuid has no preconditions and does not access caller-provided memory.
    unsafe { libc::geteuid() }
}

fn open_boot_log(path: &Path) -> Result<fs::File, LauncherError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return Err(LauncherError::UnsafeBootLog);
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|error| {
            if matches!(error.raw_os_error(), Some(libc::ELOOP | libc::EISDIR)) {
                LauncherError::UnsafeBootLog
            } else {
                LauncherError::Io(error)
            }
        })?;
    let metadata = file.metadata()?;
    validate_boot_log_attributes(
        metadata.is_file(),
        metadata.uid(),
        metadata.mode(),
        current_uid(),
    )?;
    Ok(file)
}

fn spawn_daemon(daemon_executable: &Path, paths: &RuntimePaths) -> Result<(), LauncherError> {
    let boot_log = open_boot_log(&paths.boot_log)?;
    let boot_log_stderr = boot_log.try_clone()?;

    let mut command = Command::new(daemon_executable);
    command
        .arg("--data-dir")
        .arg(&paths.data_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::from(boot_log))
        .stderr(Stdio::from(boot_log_stderr));
    command.as_std_mut().process_group(0);
    command.spawn()?;
    Ok(())
}

async fn executable_hash(path: &Path) -> Result<String, LauncherError> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut digest = Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn transport_io_error(error: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(test)]
mod tests {
    use std::os::fd::AsRawFd as _;

    use super::*;

    #[test]
    fn runtime_directory_attributes_reject_non_current_uid() {
        validate_runtime_directory_attributes(true, 501, 0o700, 502)
            .expect_err("runtime directory owned by another user");
    }

    #[test]
    fn boot_log_attributes_reject_non_current_uid() {
        validate_boot_log_attributes(true, 501, 0o600, 502)
            .expect_err("boot log owned by another user");
    }

    #[test]
    fn boot_log_descriptor_is_close_on_exec() {
        let temp = tempfile::tempdir().expect("tempdir");
        let file = open_boot_log(&temp.path().join("daemon.boot.log")).expect("boot log");

        // SAFETY: fcntl with F_GETFD only reads flags from a valid owned descriptor.
        let flags = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFD) };

        assert_ne!(flags, -1, "F_GETFD failed");
        assert_ne!(flags & libc::FD_CLOEXEC, 0);
    }
}
