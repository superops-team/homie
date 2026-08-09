use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use homie_client::{LauncherOptions, RuntimeLauncher};
use homie_proto::paths::{RuntimePathError, RuntimePaths};
use homie_proto::transport::{
    EndpointRole, Frame, FrameHeader, FrameKind, HelloResponse, PREFACE_LEN, WIRE_MAJOR, WIRE_MINOR,
};
use sha2::{Digest as _, Sha256};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::UnixListener;

#[test]
fn runtime_paths_derive_fixed_paths_from_absolute_data_dir() {
    let temp = tempfile::tempdir().expect("tempdir");
    let data_dir = fs::canonicalize(temp.path()).expect("canonical tempdir");

    let paths = RuntimePaths::new(temp.path()).expect("absolute runtime paths");

    assert_eq!(paths.data_dir, data_dir);
    assert_eq!(paths.runtime_dir, data_dir.join("runtime"));
    assert_eq!(paths.socket, data_dir.join("runtime/daemon.sock"));
    assert_eq!(paths.lock, data_dir.join("runtime/daemon.lock"));
    assert_eq!(paths.boot_log, data_dir.join("runtime/daemon.boot.log"));
    assert_eq!(paths.daemon_log, data_dir.join("runtime/daemon.log"));
}

#[test]
fn runtime_paths_canonicalize_existing_data_dir() {
    let temp = tempfile::tempdir().expect("tempdir");
    let data_dir = temp.path().join("data");
    let alias = temp.path().join("data-alias");
    fs::create_dir(&data_dir).expect("data dir");
    std::os::unix::fs::symlink(&data_dir, &alias).expect("data dir symlink");

    let paths = RuntimePaths::new(&alias).expect("canonical runtime paths");
    let data_dir = fs::canonicalize(&data_dir).expect("canonical data dir");

    assert_eq!(paths.data_dir, data_dir);
    assert_eq!(paths.runtime_dir, data_dir.join("runtime"));
}

#[test]
fn runtime_paths_reject_relative_data_dir() {
    let error = RuntimePaths::new(Path::new("relative/data")).expect_err("relative path");

    assert_eq!(error, RuntimePathError::DataDirMustBeAbsolute);
}

#[test]
fn runtime_paths_reject_missing_data_dir() {
    let temp = tempfile::tempdir().expect("tempdir");
    let missing = temp.path().join("missing");

    let error = RuntimePaths::new(&missing).expect_err("missing data dir");

    assert_eq!(error, RuntimePathError::DataDirUnavailable);
}

#[tokio::test]
async fn launcher_creates_missing_data_dir_before_deriving_paths() {
    let fixture = LauncherFixture::new();
    let data_dir = fixture._temp.path().join("missing");
    let options = LauncherOptions {
        data_dir: data_dir.clone(),
        ..fixture.options()
    };

    let paths = RuntimeLauncher::ensure_running(options)
        .await
        .expect("spawn daemon");

    assert_eq!(
        paths.data_dir,
        fs::canonicalize(&data_dir).expect("canonical data dir")
    );
}

#[tokio::test]
async fn launcher_rejects_runtime_directory_symlink_without_chmod_target() {
    let fixture = LauncherFixture::new();
    let runtime_target = fixture._temp.path().join("runtime-target");
    fs::create_dir(&runtime_target).expect("runtime target");
    fs::set_permissions(&runtime_target, fs::Permissions::from_mode(0o755))
        .expect("runtime target mode");
    std::os::unix::fs::symlink(&runtime_target, fixture.data_dir.join("runtime"))
        .expect("runtime symlink");

    RuntimeLauncher::ensure_running(fixture.options())
        .await
        .expect_err("runtime directory symlink");

    assert_eq!(mode(&runtime_target), 0o755);
}

#[tokio::test]
async fn launcher_rejects_existing_runtime_path_that_is_not_directory() {
    let fixture = LauncherFixture::new();
    fs::write(fixture.data_dir.join("runtime"), "not a directory").expect("runtime file");

    RuntimeLauncher::ensure_running(fixture.options())
        .await
        .expect_err("runtime path is not a directory");

    assert!(
        !fixture.argv_path.exists(),
        "invalid runtime path spawned daemon"
    );
}

#[tokio::test]
async fn launcher_rejects_runtime_directory_with_broad_permissions_without_chmod() {
    let fixture = LauncherFixture::new();
    let runtime_dir = fixture.data_dir.join("runtime");
    fs::create_dir(&runtime_dir).expect("runtime dir");
    fs::set_permissions(&runtime_dir, fs::Permissions::from_mode(0o750)).expect("runtime dir mode");

    RuntimeLauncher::ensure_running(fixture.options())
        .await
        .expect_err("broad runtime directory permissions");

    assert_eq!(mode(&runtime_dir), 0o750);
}

#[tokio::test]
async fn launcher_rejects_socket_symlink_without_chmod_target() {
    let fixture = LauncherFixture::new();
    let paths = RuntimePaths::new(&fixture.data_dir).expect("runtime paths");
    fs::create_dir(&paths.runtime_dir).expect("runtime dir");
    fs::set_permissions(&paths.runtime_dir, fs::Permissions::from_mode(0o700))
        .expect("runtime dir mode");
    let target = fixture._temp.path().join("socket-target");
    fs::write(&target, "not a socket").expect("socket target");
    fs::set_permissions(&target, fs::Permissions::from_mode(0o644)).expect("socket target mode");
    std::os::unix::fs::symlink(&target, &paths.socket).expect("socket symlink");

    RuntimeLauncher::ensure_running(fixture.options())
        .await
        .expect_err("socket symlink");

    assert_eq!(mode(&target), 0o644);
}

#[tokio::test]
async fn launcher_does_not_chmod_live_socket() {
    let fixture = LauncherFixture::new();
    let paths = RuntimePaths::new(&fixture.data_dir).expect("runtime paths");
    fs::create_dir(&paths.runtime_dir).expect("runtime dir");
    fs::set_permissions(&paths.runtime_dir, fs::Permissions::from_mode(0o700))
        .expect("runtime dir mode");
    let listener = UnixListener::bind(&paths.socket).expect("bind socket");
    fs::set_permissions(&paths.socket, fs::Permissions::from_mode(0o660)).expect("socket mode");
    let expected_hash = executable_hash(&fixture.daemon_path);
    let server = tokio::spawn(serve_hello(listener, expected_hash));

    RuntimeLauncher::ensure_running(fixture.options())
        .await
        .expect("live daemon");

    server.await.expect("server task");
    assert_eq!(mode(&paths.socket), 0o660);
}

#[tokio::test]
async fn launcher_rejects_boot_log_symlink_without_chmod_target() {
    let fixture = LauncherFixture::new();
    let paths = prepare_runtime_dir(&fixture);
    let target = fixture._temp.path().join("boot-log-target");
    fs::write(&target, "existing log").expect("boot log target");
    fs::set_permissions(&target, fs::Permissions::from_mode(0o644)).expect("boot log target mode");
    std::os::unix::fs::symlink(&target, &paths.boot_log).expect("boot log symlink");

    let result = RuntimeLauncher::ensure_running(fixture.options()).await;

    assert_eq!(mode(&target), 0o644);
    result.expect_err("boot log symlink");
}

#[tokio::test]
async fn launcher_rejects_existing_boot_log_with_wrong_mode_without_chmod() {
    let fixture = LauncherFixture::new();
    let paths = prepare_runtime_dir(&fixture);
    fs::write(&paths.boot_log, "existing log").expect("boot log");
    fs::set_permissions(&paths.boot_log, fs::Permissions::from_mode(0o640)).expect("boot log mode");

    let result = RuntimeLauncher::ensure_running(fixture.options()).await;

    assert_eq!(mode(&paths.boot_log), 0o640);
    result.expect_err("boot log mode");
}

#[tokio::test]
async fn launcher_rejects_existing_boot_log_that_is_not_regular_file() {
    let fixture = LauncherFixture::new();
    let paths = prepare_runtime_dir(&fixture);
    fs::create_dir(&paths.boot_log).expect("boot log directory");
    fs::set_permissions(&paths.boot_log, fs::Permissions::from_mode(0o600))
        .expect("boot log directory mode");

    let error = RuntimeLauncher::ensure_running(fixture.options())
        .await
        .expect_err("boot log is not regular");

    assert_eq!(error.code(), "bad_request");
}

#[tokio::test]
async fn launcher_creates_boot_log_with_mode_0600() {
    let fixture = LauncherFixture::new();

    let paths = RuntimeLauncher::ensure_running(fixture.options())
        .await
        .expect("spawn daemon");

    assert_eq!(mode(&paths.boot_log), 0o600);
}

#[tokio::test]
async fn launcher_spawns_missing_daemon_with_explicit_data_dir_and_returns_early() {
    let fixture = LauncherFixture::new();
    let started = Instant::now();

    RuntimeLauncher::ensure_running(fixture.options())
        .await
        .expect("spawn daemon");

    assert!(
        started.elapsed() < Duration::from_secs(1),
        "launcher waited for daemon readiness"
    );
    let argv = wait_for_file(&fixture.argv_path).await;
    assert_eq!(
        argv.lines().collect::<Vec<_>>(),
        [
            "--data-dir",
            fs::canonicalize(&fixture.data_dir)
                .expect("canonical data dir")
                .to_str()
                .expect("utf8 path")
        ]
    );
}

#[tokio::test]
async fn launcher_spawns_canonical_daemon_executable() {
    let fixture = LauncherFixture::new();
    let real_dir = fixture._temp.path().join("real");
    let alias_dir = fixture._temp.path().join("alias");
    fs::create_dir(&real_dir).expect("real dir");
    fs::create_dir(&alias_dir).expect("alias dir");
    let daemon = real_dir.join("daemon");
    let alias = alias_dir.join("daemon");
    let spawned_path = fixture._temp.path().join("spawned-path");
    write_executable(
        &daemon,
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$0\" > \"{}\"\n",
            spawned_path.display()
        ),
    );
    std::os::unix::fs::symlink(&daemon, &alias).expect("daemon symlink");
    let options = LauncherOptions {
        daemon_executable: alias.clone(),
        ..fixture.options()
    };

    RuntimeLauncher::ensure_running(options)
        .await
        .expect("spawn daemon");

    assert_eq!(
        wait_for_file(&spawned_path).await.trim(),
        fs::canonicalize(alias)
            .expect("canonical daemon")
            .to_str()
            .expect("utf8 daemon path")
    );
}

#[tokio::test]
async fn launcher_hashes_canonical_daemon_when_symlink_changes_during_probe() {
    let fixture = LauncherFixture::new();
    let paths = prepare_runtime_dir(&fixture);
    let original = fixture._temp.path().join("daemon-original");
    let replacement = fixture._temp.path().join("daemon-replacement");
    let alias = fixture._temp.path().join("daemon-alias");
    write_executable(&original, "#!/bin/sh\nexit 0\n");
    write_executable(&replacement, "#!/bin/sh\nexit 1\n");
    std::os::unix::fs::symlink(&original, &alias).expect("daemon symlink");
    let listener = UnixListener::bind(&paths.socket).expect("bind socket");
    let expected_hash = executable_hash(&original);
    let alias_for_server = alias.clone();
    let server = tokio::spawn(serve_hello_after(listener, expected_hash, move || {
        fs::remove_file(&alias_for_server).expect("remove daemon symlink");
        std::os::unix::fs::symlink(&replacement, &alias_for_server)
            .expect("replace daemon symlink");
    }));
    let options = LauncherOptions {
        daemon_executable: alias,
        ..fixture.options()
    };

    RuntimeLauncher::ensure_running(options)
        .await
        .expect("live daemon with canonical executable");

    server.await.expect("server task");
}

#[tokio::test]
async fn launcher_does_not_spawn_when_live_daemon_hello_succeeds() {
    let fixture = LauncherFixture::new();
    let paths = prepare_runtime_dir(&fixture);
    let listener = UnixListener::bind(&paths.socket).expect("bind socket");
    let expected_hash = executable_hash(&fixture.daemon_path);
    let server = tokio::spawn(serve_hello(listener, expected_hash));

    RuntimeLauncher::ensure_running(fixture.options())
        .await
        .expect("live daemon");

    server.await.expect("server task");
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(!fixture.argv_path.exists(), "live daemon was replaced");
}

#[tokio::test]
async fn launcher_does_not_replace_live_daemon_with_different_hash() {
    let fixture = LauncherFixture::new();
    let paths = prepare_runtime_dir(&fixture);
    let listener = UnixListener::bind(&paths.socket).expect("bind socket");
    let server = tokio::spawn(serve_hello(listener, "different-hash".to_string()));

    let error = RuntimeLauncher::ensure_running(fixture.options())
        .await
        .expect_err("hash mismatch");

    server.await.expect("server task");
    assert_eq!(error.code(), "version_mismatch");
    assert!(!fixture.argv_path.exists(), "live daemon was replaced");
}

#[tokio::test]
async fn launcher_rejects_relative_missing_and_non_executable_daemon_paths() {
    let fixture = LauncherFixture::new();
    let relative = LauncherOptions {
        daemon_executable: PathBuf::from("relative-daemon"),
        ..fixture.options()
    };
    let missing = LauncherOptions {
        daemon_executable: fixture.data_dir.join("missing-daemon"),
        ..fixture.options()
    };
    let non_executable_path = fixture.data_dir.join("not-executable");
    fs::write(&non_executable_path, "#!/bin/sh\n").expect("write fixture");
    let non_executable = LauncherOptions {
        daemon_executable: non_executable_path,
        ..fixture.options()
    };

    let relative_error = RuntimeLauncher::ensure_running(relative)
        .await
        .expect_err("relative executable");
    let missing_error = RuntimeLauncher::ensure_running(missing)
        .await
        .expect_err("missing executable");
    let non_executable_error = RuntimeLauncher::ensure_running(non_executable)
        .await
        .expect_err("non executable");

    assert_eq!(relative_error.code(), "bad_request");
    assert_eq!(missing_error.code(), "bad_request");
    assert_eq!(non_executable_error.code(), "bad_request");
    assert!(
        !fixture.argv_path.exists(),
        "invalid daemon path was spawned"
    );
}

struct LauncherFixture {
    _temp: tempfile::TempDir,
    data_dir: PathBuf,
    daemon_path: PathBuf,
    argv_path: PathBuf,
}

impl LauncherFixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("tempdir");
        let data_dir = temp.path().join("data");
        fs::create_dir_all(&data_dir).expect("data dir");
        let daemon_path = temp.path().join("fake-daemon");
        let argv_path = temp.path().join("argv");
        fs::write(
            &daemon_path,
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$(dirname \"$0\")/argv\"\nsleep 2\n",
        )
        .expect("write daemon fixture");
        fs::set_permissions(&daemon_path, fs::Permissions::from_mode(0o700))
            .expect("chmod daemon fixture");
        Self {
            _temp: temp,
            data_dir,
            daemon_path,
            argv_path,
        }
    }

    fn options(&self) -> LauncherOptions {
        LauncherOptions {
            data_dir: self.data_dir.clone(),
            daemon_executable: self.daemon_path.clone(),
            startup_probe_timeout: Duration::from_secs(1),
        }
    }
}

async fn serve_hello(listener: UnixListener, executable_hash: String) {
    serve_hello_after(listener, executable_hash, || {}).await;
}

async fn serve_hello_after(
    listener: UnixListener,
    executable_hash: String,
    before_response: impl FnOnce() + Send,
) {
    let (mut stream, _) = listener.accept().await.expect("accept");
    let mut preface = [0_u8; PREFACE_LEN];
    stream.read_exact(&mut preface).await.expect("preface");
    let mut length = [0_u8; 4];
    stream.read_exact(&mut length).await.expect("frame length");
    let frame_len = u32::from_be_bytes(length) as usize;
    let mut encoded = Vec::with_capacity(4 + frame_len);
    encoded.extend_from_slice(&length);
    encoded.resize(4 + frame_len, 0);
    stream
        .read_exact(&mut encoded[4..])
        .await
        .expect("hello frame");
    let hello = Frame::decode(&encoded, EndpointRole::Client)
        .expect("decode hello")
        .expect("complete hello")
        .0;
    assert_eq!(hello.header.kind, FrameKind::Hello);
    before_response();

    let response = HelloResponse {
        wire_major: WIRE_MAJOR,
        wire_minor: WIRE_MINOR,
        daemon_build: "test".to_string(),
        daemon_version: "0.1.0".to_string(),
        daemon_pid: std::process::id(),
        daemon_instance_id: "test-daemon".to_string(),
        executable_hash,
        method_capabilities: Vec::new(),
        stream_capabilities: Vec::new(),
        event_oldest_seq: 0,
        event_latest_seq: 0,
    };
    let frame = Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::HelloAck,
            flags: 0,
            stream_id: 0,
            message_id: 0,
            sequence: 0,
        },
        payload: serde_json::to_vec(&response).expect("hello response"),
    };
    stream
        .write_all(
            &frame
                .encode(EndpointRole::Server)
                .expect("encode hello ack"),
        )
        .await
        .expect("write hello ack");
}

fn executable_hash(path: &Path) -> String {
    let mut digest = Sha256::new();
    digest.update(fs::read(path).expect("read executable"));
    format!("{:x}", digest.finalize())
}

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).expect("write executable");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("executable mode");
}

fn mode(path: &Path) -> u32 {
    fs::symlink_metadata(path)
        .expect("path metadata")
        .permissions()
        .mode()
        & 0o777
}

fn prepare_runtime_dir(fixture: &LauncherFixture) -> RuntimePaths {
    let paths = RuntimePaths::new(&fixture.data_dir).expect("runtime paths");
    fs::create_dir(&paths.runtime_dir).expect("runtime dir");
    fs::set_permissions(&paths.runtime_dir, fs::Permissions::from_mode(0o700))
        .expect("runtime dir mode");
    paths
}

async fn wait_for_file(path: &Path) -> String {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Ok(contents) = fs::read_to_string(path) {
            return contents;
        }
        assert!(Instant::now() < deadline, "fixture did not write argv");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
