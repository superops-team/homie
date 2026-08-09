use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io;
use std::os::fd::AsRawFd as _;
use std::os::unix::fs::{
    DirBuilderExt as _, FileTypeExt as _, MetadataExt as _, OpenOptionsExt as _,
    PermissionsExt as _,
};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use homie_proto::paths::{RuntimePathError, RuntimePaths};
use sha2::{Digest as _, Sha256};
use thiserror::Error;
use tokio::io::AsyncReadExt as _;
use tokio::net::UnixListener;
use uuid::Uuid;

const EXECUTABLE_HASH_BUFFER_SIZE: usize = 64 * 1024;

#[derive(Debug)]
pub struct DaemonLease {
    paths: RuntimePaths,
    _lock_file: File,
    socket_identity: Option<FileIdentity>,
}

impl DaemonLease {
    pub fn acquire(data_dir: impl AsRef<Path>) -> Result<Self, DaemonError> {
        let paths = RuntimePaths::new(data_dir)?;
        prepare_runtime_directory(&paths.runtime_dir)?;
        let lock_file = open_lock_file(&paths.lock)?;
        acquire_lock(&lock_file)?;
        remove_stale_socket(&paths.socket)?;

        Ok(Self {
            paths,
            _lock_file: lock_file,
            socket_identity: None,
        })
    }

    #[must_use]
    pub const fn paths(&self) -> &RuntimePaths {
        &self.paths
    }

    pub fn bind(&mut self) -> Result<UnixListener, DaemonError> {
        if self.socket_identity.is_some() {
            return Err(DaemonError::AlreadyBound);
        }

        let listener = UnixListener::bind(&self.paths.socket)?;
        let metadata = fs::symlink_metadata(&self.paths.socket)?;
        let identity = FileIdentity::from_metadata(&metadata);
        if !metadata.file_type().is_socket() || metadata.uid() != current_uid() {
            let _ = unlink_socket_if_identity(&self.paths.socket, identity);
            return Err(DaemonError::UnsafeSocket);
        }

        if let Err(error) =
            fs::set_permissions(&self.paths.socket, fs::Permissions::from_mode(0o600))
        {
            let _ = unlink_socket_if_identity(&self.paths.socket, identity);
            return Err(error.into());
        }

        let verified = match fs::symlink_metadata(&self.paths.socket) {
            Ok(metadata) => metadata,
            Err(error) => {
                let _ = unlink_socket_if_identity(&self.paths.socket, identity);
                return Err(error.into());
            }
        };
        if !verified.file_type().is_socket()
            || verified.uid() != current_uid()
            || verified.mode() & 0o7777 != 0o600
            || FileIdentity::from_metadata(&verified) != identity
        {
            let _ = unlink_socket_if_identity(&self.paths.socket, identity);
            return Err(DaemonError::UnsafeSocket);
        }

        self.socket_identity = Some(identity);
        Ok(listener)
    }
}

impl Drop for DaemonLease {
    fn drop(&mut self) {
        if let Some(identity) = self.socket_identity.take() {
            let _ = unlink_socket_if_identity(&self.paths.socket, identity);
        }
        // `_lock_file` is dropped after this method, so socket cleanup remains lock-owned.
    }
}

pub async fn executable_sha256(path: impl AsRef<Path>) -> Result<String, DaemonError> {
    let canonical = tokio::fs::canonicalize(path)
        .await
        .map_err(DaemonError::ExecutableHash)?;
    let mut file = tokio::fs::File::open(canonical)
        .await
        .map_err(DaemonError::ExecutableHash)?;
    let metadata = file.metadata().await.map_err(DaemonError::ExecutableHash)?;
    if !metadata.is_file() {
        return Err(DaemonError::ExecutableHash(io::Error::new(
            io::ErrorKind::InvalidInput,
            "daemon executable is not a regular file",
        )));
    }
    let mut digest = Sha256::new();
    let mut buffer = vec![0_u8; EXECUTABLE_HASH_BUFFER_SIZE];
    loop {
        let read = file
            .read(&mut buffer)
            .await
            .map_err(DaemonError::ExecutableHash)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

pub fn canonical_daemon_executables(
    current_exe: impl AsRef<Path>,
) -> Result<(PathBuf, PathBuf), DaemonError> {
    let daemon = fs::canonicalize(current_exe).map_err(DaemonError::Executable)?;
    validate_executable(&daemon)?;
    let holder = fs::canonicalize(daemon.with_file_name("homie-runtime-holder"))
        .map_err(DaemonError::Executable)?;
    validate_executable(&holder)?;
    Ok((daemon, holder))
}

#[must_use]
pub fn daemon_instance_id() -> String {
    Uuid::now_v7().to_string()
}

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error(transparent)]
    RuntimePath(#[from] RuntimePathError),
    #[error("runtime directory must be a mode 0700 directory owned by the current user")]
    UnsafeRuntimeDirectory,
    #[error("daemon lock must be a mode 0600 regular file owned by the current user")]
    UnsafeLockFile,
    #[error("a daemon already owns this data directory")]
    AlreadyRunning,
    #[error("runtime socket must be a socket owned by the current user")]
    UnsafeSocket,
    #[error("runtime socket is live without the singleton lock")]
    InconsistentLiveSocket,
    #[error("daemon socket is already bound")]
    AlreadyBound,
    #[error("daemon executable hash failed")]
    ExecutableHash(#[source] io::Error),
    #[error("daemon executable validation failed")]
    Executable(#[source] io::Error),
    #[error("daemon filesystem operation failed")]
    Io(#[from] io::Error),
}

fn validate_executable(path: &Path) -> Result<(), DaemonError> {
    let metadata = fs::metadata(path).map_err(DaemonError::Executable)?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(DaemonError::Executable(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "required executable is not a regular executable file",
        )));
    }
    Ok(())
}

fn prepare_runtime_directory(path: &Path) -> Result<(), DaemonError> {
    match fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            match DirBuilder::new().mode(0o700).create(path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error.into()),
            }
        }
        Err(error) => return Err(error.into()),
    }

    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != current_uid()
        || metadata.mode() & 0o7777 != 0o700
    {
        return Err(DaemonError::UnsafeRuntimeDirectory);
    }
    Ok(())
}

fn open_lock_file(path: &Path) -> Result<File, DaemonError> {
    let new_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path);

    let file = match new_file {
        Ok(file) => {
            file.set_permissions(fs::Permissions::from_mode(0o600))?;
            file
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)
            .map_err(map_unsafe_lock_open)?,
        Err(error) => return Err(map_unsafe_lock_open(error)),
    };

    validate_lock_file(&file)?;
    Ok(file)
}

fn map_unsafe_lock_open(error: io::Error) -> DaemonError {
    if matches!(error.raw_os_error(), Some(libc::ELOOP) | Some(libc::EISDIR)) {
        DaemonError::UnsafeLockFile
    } else {
        DaemonError::Io(error)
    }
}

fn validate_lock_file(file: &File) -> Result<(), DaemonError> {
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.uid() != current_uid() || metadata.mode() & 0o7777 != 0o600 {
        return Err(DaemonError::UnsafeLockFile);
    }
    Ok(())
}

fn acquire_lock(file: &File) -> Result<(), DaemonError> {
    // SAFETY: flock receives a valid descriptor owned by `file` and no pointer arguments.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
        return Ok(());
    }

    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::EWOULDBLOCK) || error.raw_os_error() == Some(libc::EAGAIN)
    {
        Err(DaemonError::AlreadyRunning)
    } else {
        Err(error.into())
    }
}

fn remove_stale_socket(path: &Path) -> Result<(), DaemonError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if !metadata.file_type().is_socket() || metadata.uid() != current_uid() {
        return Err(DaemonError::UnsafeSocket);
    }
    let identity = FileIdentity::from_metadata(&metadata);

    match UnixStream::connect(path) {
        Ok(_) => Err(DaemonError::InconsistentLiveSocket),
        Err(error) if !connect_error_requires_stale_cleanup(error.kind()) => Ok(()),
        Err(_) => {
            let current = match fs::symlink_metadata(path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
                Err(error) => return Err(error.into()),
            };
            if !current.file_type().is_socket()
                || current.uid() != current_uid()
                || FileIdentity::from_metadata(&current) != identity
            {
                return Err(DaemonError::UnsafeSocket);
            }
            fs::remove_file(path)?;
            Ok(())
        }
    }
}

fn connect_error_requires_stale_cleanup(kind: io::ErrorKind) -> bool {
    kind != io::ErrorKind::NotFound
}

fn unlink_socket_if_identity(path: &Path, identity: FileIdentity) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata)
            if metadata.file_type().is_socket()
                && FileIdentity::from_metadata(&metadata) == identity =>
        {
            fs::remove_file(path)
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileIdentity {
    device: u64,
    inode: u64,
}

impl FileIdentity {
    fn from_metadata(metadata: &fs::Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
}

fn current_uid() -> u32 {
    // SAFETY: geteuid has no preconditions and does not access caller-provided memory.
    unsafe { libc::geteuid() }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, DirBuilder};
    use std::os::fd::AsRawFd as _;
    use std::os::unix::fs::{
        DirBuilderExt as _, FileTypeExt as _, MetadataExt as _, PermissionsExt as _, symlink,
    };
    use std::os::unix::net::UnixListener;
    use std::path::{Path, PathBuf};

    use homie_proto::paths::RuntimePathError;
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn acquire_rejects_relative_data_directory() {
        let error = DaemonLease::acquire(Path::new("relative"))
            .expect_err("relative data directory must fail");

        assert!(matches!(
            error,
            DaemonError::RuntimePath(RuntimePathError::DataDirMustBeAbsolute)
        ));
    }

    #[test]
    fn acquire_rejects_runtime_directory_with_unsafe_mode() {
        let fixture = Fixture::new();
        let runtime_dir = fixture.create_runtime_dir();
        fs::set_permissions(&runtime_dir, fs::Permissions::from_mode(0o750))
            .expect("set unsafe runtime mode");

        let error = DaemonLease::acquire(&fixture.data_dir)
            .expect_err("unsafe runtime directory must fail");

        assert!(matches!(error, DaemonError::UnsafeRuntimeDirectory));
    }

    #[test]
    fn acquire_rejects_runtime_directory_symlink() {
        let fixture = Fixture::new();
        let target = fixture.data_dir.join("runtime-target");
        DirBuilder::new()
            .mode(0o700)
            .create(&target)
            .expect("create symlink target");
        symlink(&target, fixture.runtime_dir()).expect("create runtime symlink");

        let error = DaemonLease::acquire(&fixture.data_dir).expect_err("runtime symlink must fail");

        assert!(matches!(error, DaemonError::UnsafeRuntimeDirectory));
    }

    #[test]
    fn acquire_rejects_lock_with_unsafe_mode() {
        let fixture = Fixture::new();
        fixture.create_runtime_dir();
        let lock = fixture.lock_path();
        fs::write(&lock, b"").expect("create lock");
        fs::set_permissions(&lock, fs::Permissions::from_mode(0o640))
            .expect("set unsafe lock mode");

        let error =
            DaemonLease::acquire(&fixture.data_dir).expect_err("unsafe lock mode must fail");

        assert!(matches!(error, DaemonError::UnsafeLockFile));
    }

    #[test]
    fn acquire_rejects_lock_symlink_without_modifying_target() {
        let fixture = Fixture::new();
        fixture.create_runtime_dir();
        let target = fixture.data_dir.join("lock-target");
        fs::write(&target, b"sentinel").expect("create lock target");
        fs::set_permissions(&target, fs::Permissions::from_mode(0o600))
            .expect("set lock target mode");
        symlink(&target, fixture.lock_path()).expect("create lock symlink");

        let error = DaemonLease::acquire(&fixture.data_dir).expect_err("lock symlink must fail");

        assert!(
            matches!(error, DaemonError::UnsafeLockFile)
                && fs::read(&target).expect("read target") == b"sentinel"
        );
    }

    #[test]
    fn acquire_opens_lock_close_on_exec() {
        let fixture = Fixture::new();
        let lease = DaemonLease::acquire(&fixture.data_dir).expect("acquire lease");

        // SAFETY: F_GETFD only reads flags from the valid descriptor owned by the lease.
        let flags = unsafe { libc::fcntl(lease._lock_file.as_raw_fd(), libc::F_GETFD) };

        assert!(flags >= 0 && flags & libc::FD_CLOEXEC != 0);
    }

    #[test]
    fn acquire_allows_only_one_live_lease() {
        let fixture = Fixture::new();
        let first = DaemonLease::acquire(&fixture.data_dir).expect("first lease");

        let error =
            DaemonLease::acquire(&fixture.data_dir).expect_err("second lease must not acquire");

        assert!(matches!(error, DaemonError::AlreadyRunning));
        drop(first);
        DaemonLease::acquire(&fixture.data_dir).expect("lease after release");
    }

    #[test]
    fn acquire_removes_owner_stale_socket_after_locking() {
        let fixture = Fixture::new();
        fixture.create_runtime_dir();
        let listener = UnixListener::bind(fixture.socket_path()).expect("bind stale socket");
        drop(listener);

        let _lease = DaemonLease::acquire(&fixture.data_dir).expect("acquire over stale socket");

        assert!(!fixture.socket_path().exists());
    }

    #[test]
    fn permission_denied_connect_error_requires_stale_socket_cleanup() {
        assert!(connect_error_requires_stale_cleanup(
            io::ErrorKind::PermissionDenied
        ));
    }

    #[test]
    fn not_found_connect_error_is_a_stale_socket_race_noop() {
        assert!(!connect_error_requires_stale_cleanup(
            io::ErrorKind::NotFound
        ));
    }

    #[test]
    fn acquire_preserves_live_socket_when_lock_is_free() {
        let fixture = Fixture::new();
        fixture.create_runtime_dir();
        let _listener = UnixListener::bind(fixture.socket_path()).expect("bind live socket");

        let error = DaemonLease::acquire(&fixture.data_dir)
            .expect_err("live socket without lock must be inconsistent");

        assert!(
            matches!(error, DaemonError::InconsistentLiveSocket) && fixture.socket_path().exists()
        );
    }

    #[test]
    fn acquire_preserves_regular_file_at_socket_path() {
        let fixture = Fixture::new();
        fixture.create_runtime_dir();
        fs::write(fixture.socket_path(), b"sentinel").expect("create regular socket path");

        let error = DaemonLease::acquire(&fixture.data_dir)
            .expect_err("regular socket path must be rejected");

        assert!(
            matches!(error, DaemonError::UnsafeSocket)
                && fs::read(fixture.socket_path()).expect("read socket path") == b"sentinel"
        );
    }

    #[test]
    fn acquire_preserves_symlink_at_socket_path() {
        let fixture = Fixture::new();
        fixture.create_runtime_dir();
        let target = fixture.data_dir.join("socket-target");
        fs::write(&target, b"sentinel").expect("create socket target");
        symlink(&target, fixture.socket_path()).expect("create socket symlink");

        let error =
            DaemonLease::acquire(&fixture.data_dir).expect_err("socket symlink must be rejected");

        assert!(
            matches!(error, DaemonError::UnsafeSocket)
                && fs::symlink_metadata(fixture.socket_path())
                    .expect("socket symlink metadata")
                    .file_type()
                    .is_symlink()
                && fs::read(&target).expect("read socket target") == b"sentinel"
        );
    }

    #[tokio::test]
    async fn bind_creates_owner_only_socket() {
        let fixture = Fixture::new();
        let mut lease = DaemonLease::acquire(&fixture.data_dir).expect("acquire lease");

        let _listener = lease.bind().expect("bind socket");
        let metadata = fs::symlink_metadata(fixture.socket_path()).expect("socket metadata");

        assert!(
            metadata.file_type().is_socket()
                && metadata.uid() == current_uid()
                && metadata.mode() & 0o7777 == 0o600
        );
    }

    #[tokio::test]
    async fn drop_removes_socket_owned_by_lease() {
        let fixture = Fixture::new();
        let mut lease = DaemonLease::acquire(&fixture.data_dir).expect("acquire lease");
        let listener = lease.bind().expect("bind socket");
        drop(listener);

        drop(lease);

        assert!(!fixture.socket_path().exists());
    }

    #[tokio::test]
    async fn drop_preserves_replacement_socket_with_different_inode() {
        let fixture = Fixture::new();
        let mut lease = DaemonLease::acquire(&fixture.data_dir).expect("acquire lease");
        let listener = lease.bind().expect("bind owned socket");
        let owned = fs::symlink_metadata(fixture.socket_path()).expect("owned socket metadata");
        fs::remove_file(fixture.socket_path()).expect("remove owned socket path");
        let replacement =
            UnixListener::bind(fixture.socket_path()).expect("bind replacement socket");
        let replacement_metadata =
            fs::symlink_metadata(fixture.socket_path()).expect("replacement socket metadata");
        assert_ne!(
            FileIdentity::from_metadata(&owned),
            FileIdentity::from_metadata(&replacement_metadata)
        );
        drop(listener);

        drop(lease);

        let preserved =
            fs::symlink_metadata(fixture.socket_path()).expect("replacement must remain");
        assert_eq!(
            FileIdentity::from_metadata(&preserved),
            FileIdentity::from_metadata(&replacement_metadata)
        );
        drop(replacement);
    }

    #[tokio::test]
    async fn executable_sha256_matches_known_digest() {
        let fixture = Fixture::new();
        let executable = fixture.data_dir.join("executable");
        fs::write(&executable, b"abc").expect("write executable");

        let digest = executable_sha256(&executable)
            .await
            .expect("hash executable");

        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[tokio::test]
    async fn executable_sha256_changes_with_contents() {
        let fixture = Fixture::new();
        let executable = fixture.data_dir.join("executable");
        fs::write(&executable, vec![0x5a; 64 * 1024 + 1]).expect("write first executable");
        let first = executable_sha256(&executable)
            .await
            .expect("hash first executable");
        fs::write(&executable, vec![0x5b; 64 * 1024 + 1]).expect("write changed executable");

        let changed = executable_sha256(&executable)
            .await
            .expect("hash changed executable");

        assert_ne!(first, changed);
    }

    #[tokio::test]
    async fn executable_sha256_hashes_canonical_target() {
        let fixture = Fixture::new();
        let executable = fixture.data_dir.join("executable");
        let alias = fixture.data_dir.join("executable-alias");
        fs::write(&executable, b"canonical target").expect("write executable");
        symlink(&executable, &alias).expect("create executable alias");

        let direct = executable_sha256(&executable)
            .await
            .expect("hash executable");
        let through_alias = executable_sha256(&alias).await.expect("hash alias");

        assert_eq!(direct, through_alias);
    }

    #[tokio::test]
    async fn executable_sha256_rejects_invalid_path() {
        let fixture = Fixture::new();

        let error = executable_sha256(fixture.data_dir.join("missing"))
            .await
            .expect_err("missing executable must fail");

        assert!(matches!(error, DaemonError::ExecutableHash(_)));
    }

    #[tokio::test]
    async fn executable_sha256_rejects_non_regular_file() {
        let error = executable_sha256(Path::new("/dev/null"))
            .await
            .expect_err("device must not be hashed as an executable");

        assert!(matches!(error, DaemonError::ExecutableHash(_)));
    }

    #[test]
    fn daemon_instance_id_is_uuid_v7() {
        let id = daemon_instance_id();
        let parsed = uuid::Uuid::parse_str(&id).expect("parse instance ID");

        assert_eq!(parsed.get_version_num(), 7);
    }

    #[test]
    fn daemon_instance_ids_are_unique() {
        assert_ne!(daemon_instance_id(), daemon_instance_id());
    }

    struct Fixture {
        _temp: TempDir,
        data_dir: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = tempfile::tempdir().expect("tempdir");
            let data_dir = fs::canonicalize(temp.path()).expect("canonical data dir");
            Self {
                _temp: temp,
                data_dir,
            }
        }

        fn runtime_dir(&self) -> PathBuf {
            self.data_dir.join("runtime")
        }

        fn lock_path(&self) -> PathBuf {
            self.runtime_dir().join("daemon.lock")
        }

        fn socket_path(&self) -> PathBuf {
            self.runtime_dir().join("daemon.sock")
        }

        fn create_runtime_dir(&self) -> PathBuf {
            let runtime_dir = self.runtime_dir();
            if !runtime_dir.exists() {
                DirBuilder::new()
                    .mode(0o700)
                    .create(&runtime_dir)
                    .expect("create runtime dir");
            }
            runtime_dir
        }
    }
}
