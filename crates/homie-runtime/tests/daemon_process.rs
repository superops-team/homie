#![cfg(unix)]

use std::fs;
use std::io::Read as _;
use std::os::unix::fs::{
    DirBuilderExt as _, FileTypeExt as _, MetadataExt as _, PermissionsExt as _,
};
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use homie_client::{ClientOptions, HomieClient};
use homie_proto::Method;
use homie_proto::model::{SessionSummary, StateSnapshot};
use homie_proto::paths::{RuntimeEndpoint, RuntimePaths};
use homie_proto::transport::{AckResult, ClientRole, ShutdownResult};
use homie_proto::{SessionKillRequest, SessionSpawnRequest};
use homie_runtime::holder;
use homie_runtime::{HolderPaths, HolderRequest};

const SAFE_STARTUP_ERROR: &str = "homie-runtime-daemon: startup failed\n";

#[test]
fn cargo_exposes_absolute_daemon_binary() {
    let binary = env!("CARGO_BIN_EXE_homie-runtime-daemon");

    assert!(
        Path::new(binary).is_absolute(),
        "binary path must be absolute"
    );
}

#[test]
fn relative_data_directory_is_rejected_without_echoing_input() {
    let output = Command::new(daemon_binary())
        .args(["--data-dir", "relative-secret-marker"])
        .output()
        .expect("run daemon");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("UTF-8 stderr"),
        SAFE_STARTUP_ERROR
    );
}

#[test]
fn data_directory_is_the_only_explicit_argument() {
    let temp = tempfile::tempdir().expect("tempdir");
    let absolute = temp.path().to_str().expect("UTF-8 temp path");
    let invalid_arguments = [
        Vec::new(),
        vec!["--data-dir", absolute, "--extra"],
        vec!["--data-dir", absolute, "--data-dir", absolute],
        vec!["--other", absolute],
    ];

    for arguments in invalid_arguments {
        let output = Command::new(daemon_binary())
            .args(arguments)
            .output()
            .expect("run daemon");
        assert!(!output.status.success(), "arguments must be rejected");
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8(output.stderr).expect("UTF-8 stderr"),
            SAFE_STARTUP_ERROR
        );
    }
}

#[tokio::test]
async fn daemon_starts_with_exact_identity_snapshot_and_owner_only_paths() {
    let temp = tempfile::tempdir().expect("tempdir");
    let data_dir = temp.path().join("data");
    fs::create_dir(&data_dir).expect("data dir");
    let mut daemon = DaemonChild::spawn(&data_dir);
    let paths = wait_for_socket(&mut daemon, &data_dir).await;
    let client = connect_client(&paths).await;
    let hello = client.hello().expect("hello");
    let snapshot: StateSnapshot = client
        .request(Method::STATE_SNAPSHOT, serde_json::json!({}))
        .await
        .expect("state snapshot");
    let runtime_metadata = fs::symlink_metadata(&paths.runtime_dir).expect("runtime metadata");
    let lock_metadata = fs::symlink_metadata(&paths.lock).expect("lock metadata");
    let socket_metadata = fs::symlink_metadata(&paths.socket).expect("socket metadata");
    let canonical_binary = fs::canonicalize(daemon_binary()).expect("canonical daemon binary");
    let expected_hash = homie_runtime::executable_sha256(&canonical_binary)
        .await
        .expect("daemon hash");
    let instance_id = uuid::Uuid::parse_str(&hello.daemon_instance_id).expect("instance UUID");

    assert_eq!(runtime_metadata.permissions().mode() & 0o7777, 0o700);
    assert!(lock_metadata.is_file());
    assert_eq!(lock_metadata.permissions().mode() & 0o7777, 0o600);
    assert!(socket_metadata.file_type().is_socket());
    assert_eq!(socket_metadata.permissions().mode() & 0o7777, 0o600);
    assert_eq!(hello.daemon_build, "homie-runtime/0.1.0");
    assert_eq!(hello.daemon_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(hello.daemon_pid, daemon.id());
    assert_eq!(hello.executable_hash, expected_hash);
    assert_eq!(instance_id.get_version_num(), 7);
    assert!(snapshot.sessions.is_empty());
    assert_eq!(snapshot.event_cursor, 0);

    client.close().await.expect("close client");
}

#[tokio::test]
async fn simultaneous_daemons_leave_exactly_one_owner_and_one_successful_loser() {
    let temp = tempfile::tempdir().expect("tempdir");
    let data_dir = temp.path().join("data");
    fs::create_dir(&data_dir).expect("data dir");
    let mut first = DaemonChild::spawn(&data_dir);
    let mut second = DaemonChild::spawn(&data_dir);
    let deadline = Instant::now() + Duration::from_secs(5);
    let (first_status, second_status) = loop {
        let first_status = first.child.try_wait().expect("first status");
        let second_status = second.child.try_wait().expect("second status");
        if first_status.is_some() || second_status.is_some() {
            break (first_status, second_status);
        }
        assert!(
            Instant::now() < deadline,
            "neither daemon lost singleton race"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    };

    assert_ne!(first_status.is_some(), second_status.is_some());
    let (loser_status, loser, owner) = if let Some(status) = first_status {
        (status, &mut first, &mut second)
    } else {
        (
            second_status.expect("second loser status"),
            &mut second,
            &mut first,
        )
    };
    assert!(loser_status.success(), "singleton loser must exit 0");
    loser.assert_silent();
    let paths = wait_for_socket(owner, &data_dir).await;
    let socket_identity = fs::symlink_metadata(&paths.socket).expect("socket metadata");
    let client = connect_client(&paths).await;
    let hello = client.hello().expect("hello");

    assert_eq!(hello.daemon_pid, owner.id());
    assert!(owner.child.try_wait().expect("owner status").is_none());
    assert_eq!(
        fs::symlink_metadata(&paths.socket)
            .expect("current socket metadata")
            .ino(),
        socket_identity.ino()
    );

    client.close().await.expect("close client");
}

#[tokio::test]
async fn owner_replaces_stale_socket_before_accepting_connections() {
    let temp = tempfile::tempdir().expect("tempdir");
    let data_dir = temp.path().join("data");
    fs::create_dir(&data_dir).expect("data dir");
    let paths = RuntimePaths::new(&data_dir).expect("runtime paths");
    fs::DirBuilder::new()
        .recursive(false)
        .mode(0o700)
        .create(&paths.runtime_dir)
        .expect("runtime dir");
    let stale = UnixListener::bind(&paths.socket).expect("stale socket");
    let stale_inode = fs::symlink_metadata(&paths.socket)
        .expect("stale metadata")
        .ino();
    drop(stale);
    let mut daemon = DaemonChild::spawn(&data_dir);
    let paths = wait_for_socket(&mut daemon, &data_dir).await;
    let client = connect_client(&paths).await;

    assert_ne!(
        fs::symlink_metadata(&paths.socket)
            .expect("replacement metadata")
            .ino(),
        stale_inode
    );

    client.close().await.expect("close client");
}

#[tokio::test]
async fn admin_prepare_and_shutdown_ack_before_clean_exit_and_socket_removal() {
    let temp = tempfile::tempdir().expect("tempdir");
    let data_dir = temp.path().join("data");
    fs::create_dir(&data_dir).expect("data dir");
    let mut daemon = DaemonChild::spawn(&data_dir);
    let paths = wait_for_socket(&mut daemon, &data_dir).await;
    let client = connect_client(&paths).await;

    let prepared: AckResult = client
        .request(Method::DAEMON_PREPARE_SHUTDOWN, serde_json::json!({}))
        .await
        .expect("prepare ACK");
    assert!(prepared.ok);
    assert_eq!(
        fs::metadata(data_dir.join("homie.sqlite-wal"))
            .expect("WAL metadata")
            .len(),
        0
    );
    let mutation_error = client
        .request::<_, SessionSummary>(
            Method::SESSION_SPAWN,
            SessionSpawnRequest {
                cwd: temp.path().display().to_string(),
                title: Some("must be rejected".to_string()),
                parent_session_id: None,
            },
        )
        .await
        .expect_err("draining daemon must reject new mutation");
    assert_eq!(mutation_error.code(), "unavailable");
    let shutdown: ShutdownResult = client
        .request(Method::DAEMON_SHUTDOWN, serde_json::json!({}))
        .await
        .expect("shutdown ACK");
    assert!(shutdown.acknowledged);
    let status = wait_for_exit(&mut daemon).await;

    assert!(status.success(), "daemon exit status: {status}");
    assert!(!paths.socket.exists());
    daemon.assert_silent();
    client.close().await.expect("close client");
}

#[tokio::test]
async fn sigterm_drains_and_exits_cleanly_with_socket_removal() {
    let temp = tempfile::tempdir().expect("tempdir");
    let data_dir = temp.path().join("data");
    fs::create_dir(&data_dir).expect("data dir");
    let mut daemon = DaemonChild::spawn(&data_dir);
    let paths = wait_for_socket(&mut daemon, &data_dir).await;

    // SAFETY: the positive PID belongs to the live child owned by this test.
    let sent = unsafe { libc::kill(daemon.id() as i32, libc::SIGTERM) };
    assert_eq!(sent, 0, "send SIGTERM");
    let status = wait_for_exit(&mut daemon).await;

    assert!(status.success(), "daemon exit status: {status}");
    assert!(!paths.socket.exists());
    daemon.assert_silent();
}

#[tokio::test]
async fn sigint_drains_and_exits_cleanly_with_socket_removal() {
    let temp = tempfile::tempdir().expect("tempdir");
    let data_dir = temp.path().join("data");
    fs::create_dir(&data_dir).expect("data dir");
    let mut daemon = DaemonChild::spawn(&data_dir);
    let paths = wait_for_socket(&mut daemon, &data_dir).await;

    // SAFETY: the positive PID belongs to the live child owned by this test.
    let sent = unsafe { libc::kill(daemon.id() as i32, libc::SIGINT) };
    assert_eq!(sent, 0, "send SIGINT");
    let status = wait_for_exit(&mut daemon).await;

    assert!(status.success(), "daemon exit status: {status}");
    assert!(!paths.socket.exists());
    daemon.assert_silent();
}

#[tokio::test]
async fn live_holder_survives_daemon_sigterm_and_is_cleaned_up_explicitly_after_restart() {
    let temp = tempfile::tempdir().expect("tempdir");
    let data_dir = temp.path().join("data");
    fs::create_dir(&data_dir).expect("data dir");
    let mut first_daemon = DaemonChild::spawn(&data_dir);
    let paths = wait_for_socket(&mut first_daemon, &data_dir).await;
    let first_client = connect_client(&paths).await;
    let first_instance = first_client
        .hello()
        .expect("first hello")
        .daemon_instance_id;
    let session: SessionSummary = first_client
        .request(
            Method::SESSION_SPAWN,
            SessionSpawnRequest {
                cwd: temp.path().display().to_string(),
                title: Some("holder survival".to_string()),
                parent_session_id: None,
            },
        )
        .await
        .expect("spawn session");
    let holder_paths = HolderPaths::new(&paths.data_dir, &session.id);
    let holder_pid = wait_for_holder_pid(&holder_paths).await;
    let mut holder_cleanup = HolderCleanup::new(holder_paths, holder_pid);
    assert_process_alive(holder_pid);

    // SAFETY: the positive PID belongs to the live daemon child owned by this test.
    let sent = unsafe { libc::kill(first_daemon.id() as i32, libc::SIGTERM) };
    assert_eq!(sent, 0, "send SIGTERM");
    let first_status = wait_for_exit(&mut first_daemon).await;
    assert!(
        first_status.success(),
        "first daemon status: {first_status}"
    );
    assert_process_alive(holder_pid);
    first_daemon.assert_silent();
    first_client.close().await.expect("close first client");

    let mut second_daemon = DaemonChild::spawn(&data_dir);
    let paths = wait_for_socket(&mut second_daemon, &data_dir).await;
    let second_client = connect_client(&paths).await;
    let second_instance = second_client
        .hello()
        .expect("second hello")
        .daemon_instance_id;
    let snapshot: StateSnapshot = second_client
        .request(Method::STATE_SNAPSHOT, serde_json::json!({}))
        .await
        .expect("snapshot after restart");

    assert_ne!(first_instance, second_instance);
    assert!(snapshot.sessions.iter().any(|item| item.id == session.id));
    assert_process_alive(holder_pid);

    let killed: AckResult = second_client
        .request(
            Method::SESSION_KILL,
            SessionKillRequest {
                session_id: session.id.into(),
            },
        )
        .await
        .expect("kill holder session");
    assert!(killed.ok);
    wait_for_process_exit(holder_pid).await;
    holder_cleanup.disarm();
    let shutdown: ShutdownResult = second_client
        .request(Method::DAEMON_SHUTDOWN, serde_json::json!({}))
        .await
        .expect("second shutdown ACK");
    assert!(shutdown.acknowledged);
    let second_status = wait_for_exit(&mut second_daemon).await;

    assert!(
        second_status.success(),
        "second daemon status: {second_status}"
    );
    assert!(!paths.socket.exists());
    second_daemon.assert_silent();
    second_client.close().await.expect("close second client");
}

#[test]
fn missing_holder_fails_safely_without_modifying_existing_data() {
    let temp = tempfile::tempdir().expect("tempdir");
    let bin_dir = temp.path().join("bin");
    let data_dir = temp.path().join("data");
    fs::create_dir(&bin_dir).expect("bin dir");
    fs::create_dir(&data_dir).expect("data dir");
    let copied_daemon = bin_dir.join("homie-runtime-daemon");
    fs::copy(daemon_binary(), &copied_daemon).expect("copy daemon");
    fs::set_permissions(&copied_daemon, fs::Permissions::from_mode(0o700))
        .expect("daemon permissions");
    let database = data_dir.join("homie.sqlite");
    let sentinel = data_dir.join("sentinel");
    fs::write(&database, b"database-sentinel").expect("database sentinel");
    fs::write(&sentinel, b"credential-payload-sentinel").expect("sentinel");

    let output = Command::new(&copied_daemon)
        .arg("--data-dir")
        .arg(&data_dir)
        .output()
        .expect("run copied daemon");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("UTF-8 stderr"),
        SAFE_STARTUP_ERROR
    );
    assert_eq!(
        fs::read(database).expect("database remains"),
        b"database-sentinel"
    );
    assert_eq!(
        fs::read(sentinel).expect("sentinel remains"),
        b"credential-payload-sentinel"
    );
    assert!(!data_dir.join("runtime/daemon.sock").exists());
}

#[test]
fn storage_startup_failure_preserves_database_and_sentinel() {
    let temp = tempfile::tempdir().expect("tempdir");
    let bin_dir = temp.path().join("bin");
    let data_dir = temp.path().join("data");
    fs::create_dir(&bin_dir).expect("bin dir");
    fs::create_dir(&data_dir).expect("data dir");
    let copied_daemon = bin_dir.join("homie-runtime-daemon");
    let copied_holder = bin_dir.join("homie-runtime-holder");
    fs::copy(daemon_binary(), &copied_daemon).expect("copy daemon");
    fs::copy(holder_binary(), &copied_holder).expect("copy holder");
    fs::set_permissions(&copied_daemon, fs::Permissions::from_mode(0o700))
        .expect("daemon permissions");
    fs::set_permissions(&copied_holder, fs::Permissions::from_mode(0o700))
        .expect("holder permissions");
    let database = data_dir.join("homie.sqlite");
    let sentinel = data_dir.join("sentinel");
    fs::write(&database, b"not-a-sqlite-database").expect("invalid database");
    fs::write(&sentinel, b"preserve-me").expect("sentinel");

    let output = Command::new(&copied_daemon)
        .arg("--data-dir")
        .arg(&data_dir)
        .output()
        .expect("run copied daemon");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("UTF-8 stderr"),
        SAFE_STARTUP_ERROR
    );
    assert_eq!(
        fs::read(database).expect("database remains"),
        b"not-a-sqlite-database"
    );
    assert_eq!(
        fs::read(sentinel).expect("sentinel remains"),
        b"preserve-me"
    );
    assert!(!data_dir.join("runtime/daemon.sock").exists());
}

fn daemon_binary() -> &'static Path {
    let binary = env!("CARGO_BIN_EXE_homie-runtime-daemon");
    let binary = Path::new(binary);
    assert!(binary.is_absolute(), "daemon binary path must be absolute");
    binary
}

fn holder_binary() -> &'static Path {
    let binary = Path::new(env!("CARGO_BIN_EXE_homie-runtime-holder"));
    assert!(binary.is_absolute(), "holder binary path must be absolute");
    binary
}

async fn connect_client(paths: &RuntimePaths) -> HomieClient {
    HomieClient::connect(ClientOptions {
        endpoint: RuntimeEndpoint::new(paths.socket.clone()).expect("absolute endpoint"),
        role: ClientRole::Cli,
        connect_timeout: Duration::from_secs(2),
        request_timeout: Duration::from_secs(2),
    })
    .await
    .expect("connect daemon")
}

async fn wait_for_socket(daemon: &mut DaemonChild, data_dir: &Path) -> RuntimePaths {
    let paths = RuntimePaths::new(data_dir).expect("runtime paths");
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if paths.socket.exists() {
            return paths;
        }
        if let Some(status) = daemon.child.try_wait().expect("daemon status") {
            panic!("daemon exited before readiness: {status}");
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("daemon socket did not become ready");
}

async fn wait_for_exit(daemon: &mut DaemonChild) -> std::process::ExitStatus {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = daemon.child.try_wait().expect("daemon status") {
            return status;
        }
        assert!(Instant::now() < deadline, "daemon did not exit");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

async fn wait_for_holder_pid(paths: &HolderPaths) -> i32 {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Ok(pid) = fs::read_to_string(&paths.pid_file)
            && let Ok(pid) = pid.trim().parse()
        {
            return pid;
        }
        assert!(Instant::now() < deadline, "holder PID did not become ready");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

async fn wait_for_process_exit(pid: i32) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while process_exists(pid) {
        assert!(Instant::now() < deadline, "process {pid} did not exit");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

fn assert_process_alive(pid: i32) {
    assert!(process_exists(pid), "expected process {pid} to be alive");
}

fn process_exists(pid: i32) -> bool {
    // SAFETY: kill(pid, 0) performs existence and permission checking only.
    unsafe { libc::kill(pid, 0) == 0 }
}

struct DaemonChild {
    child: Child,
}

impl DaemonChild {
    fn spawn(data_dir: &Path) -> Self {
        let child = Command::new(daemon_binary())
            .arg("--data-dir")
            .arg(data_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn daemon");
        Self { child }
    }

    fn id(&self) -> u32 {
        self.child.id()
    }

    fn assert_silent(&mut self) {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        self.child
            .stdout
            .take()
            .expect("daemon stdout")
            .read_to_end(&mut stdout)
            .expect("read daemon stdout");
        self.child
            .stderr
            .take()
            .expect("daemon stderr")
            .read_to_end(&mut stderr)
            .expect("read daemon stderr");
        assert!(stdout.is_empty(), "unexpected daemon stdout");
        assert!(stderr.is_empty(), "unexpected daemon stderr");
    }
}

struct HolderCleanup {
    paths: HolderPaths,
    pid: i32,
    armed: bool,
}

impl HolderCleanup {
    fn new(paths: HolderPaths, pid: i32) -> Self {
        Self {
            paths,
            pid,
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for HolderCleanup {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let _ = holder::request(&self.paths.socket, &HolderRequest::Terminate);
        let deadline = Instant::now() + Duration::from_secs(3);
        while process_exists(self.pid) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        if process_exists(self.pid) {
            // SAFETY: this PID was read from the holder pid file owned by this fixture.
            unsafe {
                libc::kill(self.pid, libc::SIGKILL);
            }
        }
    }
}

impl Drop for DaemonChild {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            // SAFETY: the positive PID belongs to the child owned by this test.
            unsafe {
                libc::kill(self.child.id() as i32, libc::SIGKILL);
            }
            let _ = self.child.wait();
        }
    }
}
