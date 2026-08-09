#![cfg(unix)]

use std::fs::{self, File};
use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::net::UnixStream;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use homie_app::runtime_bridge::{
    BridgeConnectionState, BridgeEvent, BridgeProjection, RuntimeBridge, RuntimeBridgeConfig,
    RuntimeCommand,
};
use homie_proto::transport::{
    ClientRole, EndpointRole, FRAME_HEADER_LEN, Frame, FrameHeader, FrameKind, HelloRequest,
    MAX_FRAME_LEN, Preface, WIRE_MAJOR, WIRE_MINOR,
};
use homie_proto::{ControlMessage, Method, RequestId};
use serde_json::{Value, json};
use tempfile::TempDir;

const PROCESS_TIMEOUT: Duration = Duration::from_secs(10);
const STATE_TIMEOUT: Duration = Duration::from_secs(12);
const IO_TIMEOUT: Duration = Duration::from_secs(2);
const SECURITY_MARKER: &[u8] = b"task19-sensitive-frame-payload";
const BEFORE_RESTART_MARKER: &str = "task19-before-restart";
const AFTER_RESTART_MARKER: &str = "task19-after-restart";

#[test]
fn app_cli_and_mcp_share_and_recover_one_real_daemon() {
    let mut fixture = E2eFixture::new();
    let result = catch_unwind(AssertUnwindSafe(|| exercise_cross_entry(&mut fixture)));
    let cleanup = fixture.cleanup_and_assert_zero();

    match (result, cleanup) {
        (Ok(()), Ok(())) => {}
        (Ok(()), Err(error)) => panic!("Task19 cleanup failed: {error}"),
        (Err(original), Ok(())) => resume_unwind(original),
        (Err(original), Err(cleanup_error)) => {
            panic!(
                "Task19 failed ({}) and cleanup failed: {cleanup_error}",
                panic_message(&original)
            );
        }
    }
}

fn exercise_cross_entry(fixture: &mut E2eFixture) {
    fixture.start_daemon();
    exercise_real_daemon_security(&fixture.socket_path());

    let first_cli_status = fixture.runtime_status();
    let first_instance = required_str(&first_cli_status, "daemonInstanceId").to_string();
    let first_daemon_pid = required_u64(&first_cli_status, "daemonPid") as u32;
    assert_eq!(Some(first_daemon_pid), fixture.current_daemon_pid());

    let mut bridge = RuntimeBridge::start(RuntimeBridgeConfig {
        data_dir: fixture.data_dir.clone(),
        current_executable: fixture.cli.clone(),
        workspace: fixture.data_dir.clone(),
        startup_probe_timeout: Duration::from_millis(250),
        connect_timeout: Duration::from_secs(5),
        request_timeout: Duration::from_secs(10),
    })
    .expect("start production app runtime bridge");
    let initial_events =
        wait_for_bridge(&mut bridge, "initial app snapshot and identity", |p, e| {
            p.connection == BridgeConnectionState::Connected
                && p.daemon_instance_id.as_deref() == Some(first_instance.as_str())
                && e.iter()
                    .any(|event| matches!(event, BridgeEvent::Snapshot(_)))
        });
    assert!(
        initial_events
            .iter()
            .any(|event| matches!(event, BridgeEvent::Snapshot(_))),
        "production app bridge did not publish its initial snapshot"
    );
    assert_eq!(
        bridge.projection().daemon_instance_id.as_deref(),
        Some(first_instance.as_str()),
        "app bridge and CLI Hello must identify the same daemon"
    );
    let initial_cursor = bridge.projection().event_cursor;

    let created = fixture.session_create("Task19 shared session");
    let session_id = required_str(&created, "id").to_string();
    fixture.sessions.push(session_id.clone());
    let session_snapshot = fixture.session_snapshot(&session_id);
    let holder_pid = required_u64(
        session_snapshot
            .get("holder")
            .expect("session snapshot holder"),
        "pid",
    ) as i32;
    fixture.holder_pids.push(holder_pid);

    let mcp_list = fixture.mcp_tool("list_agents", json!({}));
    assert!(
        mcp_list["agents"]
            .as_array()
            .expect("MCP agents")
            .iter()
            .any(|session| session["id"] == session_id),
        "MCP did not observe the CLI-created session: {mcp_list}"
    );

    bridge
        .dispatch(RuntimeCommand::RefreshSessions)
        .expect("refresh app sessions");
    wait_for_bridge(
        &mut bridge,
        "app observes CLI-created session",
        |projection, _| {
            projection
                .sessions
                .iter()
                .any(|session| session.id == session_id)
                && projection.event_cursor > initial_cursor
        },
    );
    assert_eq!(
        bridge.projection().selected_session_id.as_deref(),
        Some(session_id.as_str()),
        "app selection must resolve to the shared session"
    );
    let spawn_cursor = bridge.projection().event_cursor;
    let cli_events = fixture.events_after(0);
    assert_eq!(
        required_u64(
            cli_events.get("cursor").expect("CLI event cursor"),
            "nextSeq"
        ),
        spawn_cursor,
        "CLI and app must confirm the same event cursor"
    );

    bridge
        .dispatch(RuntimeCommand::SelectSession {
            session_id: session_id.clone(),
            output_offset: 0,
        })
        .expect("select app terminal");
    wait_for_bridge(&mut bridge, "selected terminal opens", |_, events| {
        events
            .iter()
            .any(|event| matches!(event, BridgeEvent::TerminalGrid(_)))
    });

    let sent = fixture.mcp_tool(
        "send_prompt",
        json!({
            "sessionId": session_id,
            "text": format!("printf '{}\\n'", BEFORE_RESTART_MARKER),
            "submit": true
        }),
    );
    assert_eq!(sent["ok"], true);
    let output_log = fixture
        .data_dir
        .join("runtime/output")
        .join(format!("{session_id}.log"));
    let confirmed_before_restart = wait_for_terminal_marker(&mut bridge, BEFORE_RESTART_MARKER, 0);
    let confirmed_before_restart =
        wait_for_confirmed_terminal_idle(&mut bridge, &output_log, confirmed_before_restart);
    let cursor_before_restart = bridge.projection().event_cursor;
    assert!(cursor_before_restart > spawn_cursor);

    let mut control = InteractiveControl::start(&fixture.cli, &fixture.data_dir);
    control.send(ControlMessage::request(
        RequestId::from(90),
        Method::SESSION_LIST,
        json!({}),
    ));
    assert_control_success(control.recv(PROCESS_TIMEOUT), 90);
    control.send(ControlMessage::request(
        RequestId::from(91),
        Method::EVENTS_WAIT,
        json!({
            "afterSeq": cursor_before_restart,
            "timeoutMs": 30_000,
            "eventFilter": ["task19.never"]
        }),
    ));
    control.assert_no_response(Duration::from_millis(200));

    let first_status = fixture.stop_daemon();
    assert!(first_status.success(), "first daemon exit: {first_status}");
    assert_control_error(control.recv(PROCESS_TIMEOUT), 91, "unavailable");
    control.assert_no_response(Duration::from_millis(200));
    wait_for_bridge(
        &mut bridge,
        "app observes daemon disconnect",
        |projection, _| {
            matches!(
                projection.connection,
                BridgeConnectionState::Reconnecting | BridgeConnectionState::Unavailable
            )
        },
    );
    assert_eq!(
        bridge.projection().selected_session_id.as_deref(),
        Some(session_id.as_str()),
        "terminal selection must survive daemon shutdown"
    );
    assert!(
        process_exists(holder_pid),
        "holder must survive daemon shutdown"
    );

    fixture.start_daemon();
    let reconnect_events = wait_for_bridge(&mut bridge, "app reconnects to new daemon", |p, _| {
        p.connection == BridgeConnectionState::Connected
            && p.daemon_instance_id
                .as_deref()
                .is_some_and(|instance| instance != first_instance)
    });
    let second_instance = bridge
        .projection()
        .daemon_instance_id
        .clone()
        .expect("reconnected app daemon identity");
    assert_ne!(first_instance, second_instance);
    assert!(
        reconnect_events.iter().any(|event| matches!(
            event,
            BridgeEvent::DaemonIdentity { instance_id } if instance_id == &second_instance
        )),
        "app did not publish the restarted daemon identity"
    );

    let second_cli_status = fixture.runtime_status();
    assert_eq!(
        required_str(&second_cli_status, "daemonInstanceId"),
        second_instance,
        "app reconnect and CLI Hello must identify the restarted daemon"
    );
    wait_for_control_success(&mut control, cursor_before_restart);

    let sent = fixture.mcp_tool(
        "send_prompt",
        json!({
            "sessionId": session_id,
            "text": format!("printf '{}\\n'", AFTER_RESTART_MARKER),
            "submit": true
        }),
    );
    assert_eq!(sent["ok"], true);
    let recovery =
        wait_for_restart_recovery(&mut bridge, AFTER_RESTART_MARKER, cursor_before_restart);
    assert_eq!(
        recovery.first_output_offset, confirmed_before_restart,
        "existing terminal must reopen from its last confirmed offset"
    );
    assert!(
        recovery.saw_resumed_event,
        "app event stream did not resume after daemon restart"
    );
    assert_eq!(
        bridge.projection().selected_session_id.as_deref(),
        Some(session_id.as_str()),
        "existing terminal selection changed during restart"
    );

    let resumed_cursor = bridge.projection().event_cursor;
    assert!(resumed_cursor > cursor_before_restart);
    let resumed_cli_events = fixture.events_after(cursor_before_restart);
    assert_eq!(
        required_u64(
            resumed_cli_events
                .get("cursor")
                .expect("resumed CLI event cursor"),
            "nextSeq"
        ),
        resumed_cursor,
        "event resume cursor must stay consistent across app and CLI"
    );
    let mcp_after_restart = fixture.mcp_tool("list_agents", json!({}));
    assert!(
        mcp_after_restart["agents"]
            .as_array()
            .expect("MCP agents after restart")
            .iter()
            .any(|session| session["id"] == session_id),
        "MCP lost the shared session after restart: {mcp_after_restart}"
    );

    fixture.assert_safe_introspection();
    drop(control);
    drop(bridge);
}

fn exercise_real_daemon_security(socket: &Path) {
    assert_rejected_raw_frame(socket, (FRAME_HEADER_LEN - 1) as u32, 1, 0, &[]);
    assert_rejected_raw_frame(socket, (MAX_FRAME_LEN + 1) as u32, 1, 0, &[]);
    assert_rejected_raw_frame(
        socket,
        (FRAME_HEADER_LEN + SECURITY_MARKER.len()) as u32,
        FrameKind::Hello as u8,
        1,
        SECURITY_MARKER,
    );
    assert_rejected_raw_frame(
        socket,
        (FRAME_HEADER_LEN + SECURITY_MARKER.len()) as u32,
        255,
        0,
        SECURITY_MARKER,
    );

    let mut active = Vec::with_capacity(64);
    for index in 0..64 {
        active.push(raw_handshake(socket, index));
    }
    let rejected = UnixStream::connect(socket).expect("connect 65th client");
    assert_connection_closed(rejected);
    assert_eq!(active.len(), 64);
}

fn raw_handshake(socket: &Path, index: usize) -> UnixStream {
    let mut stream = UnixStream::connect(socket).expect("connect raw client");
    configure_socket(&stream);
    stream
        .write_all(
            &Preface {
                major: WIRE_MAJOR,
                minor: WIRE_MINOR,
            }
            .encode(),
        )
        .expect("write client preface");
    let hello = Frame {
        header: FrameHeader {
            version: WIRE_MAJOR,
            kind: FrameKind::Hello,
            flags: 0,
            stream_id: 0,
            message_id: 0,
            sequence: 0,
        },
        payload: serde_json::to_vec(&HelloRequest {
            wire_major: WIRE_MAJOR,
            wire_minor: WIRE_MINOR,
            client_name: format!("task19-security-{index}"),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            client_role: ClientRole::Cli,
            process_id: std::process::id(),
        })
        .expect("encode raw Hello"),
    };
    stream
        .write_all(
            &hello
                .encode(EndpointRole::Client)
                .expect("encode raw Hello frame"),
        )
        .expect("write raw Hello frame");
    let ack = read_raw_frame(&mut stream);
    assert_eq!(ack.header.kind, FrameKind::HelloAck);
    stream
}

fn assert_rejected_raw_frame(socket: &Path, frame_len: u32, kind: u8, flags: u8, payload: &[u8]) {
    let mut stream = UnixStream::connect(socket).expect("connect hostile client");
    configure_socket(&stream);
    stream
        .write_all(
            &Preface {
                major: WIRE_MAJOR,
                minor: WIRE_MINOR,
            }
            .encode(),
        )
        .expect("write hostile preface");
    let mut encoded = Vec::with_capacity(4 + FRAME_HEADER_LEN + payload.len());
    encoded.extend_from_slice(&frame_len.to_be_bytes());
    encoded.extend_from_slice(&WIRE_MAJOR.to_be_bytes());
    encoded.push(kind);
    encoded.push(flags);
    encoded.extend_from_slice(&0_u32.to_be_bytes());
    encoded.extend_from_slice(&0_u64.to_be_bytes());
    encoded.extend_from_slice(&0_u64.to_be_bytes());
    encoded.extend_from_slice(payload);
    stream.write_all(&encoded).expect("write hostile frame");
    assert_connection_closed(stream);
}

fn read_raw_frame(stream: &mut UnixStream) -> Frame {
    let mut length = [0_u8; 4];
    stream.read_exact(&mut length).expect("read frame length");
    let frame_len = u32::from_be_bytes(length) as usize;
    let mut encoded = Vec::with_capacity(4 + frame_len);
    encoded.extend_from_slice(&length);
    encoded.resize(4 + frame_len, 0);
    stream
        .read_exact(&mut encoded[4..])
        .expect("read frame body");
    Frame::decode(&encoded, EndpointRole::Server)
        .expect("decode server frame")
        .expect("complete server frame")
        .0
}

fn assert_connection_closed(mut stream: UnixStream) {
    configure_socket(&stream);
    let mut byte = [0_u8; 1];
    match stream.read(&mut byte) {
        Ok(0) => {}
        Ok(read) => panic!("expected connection close, read {read} bytes"),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::BrokenPipe
                    | std::io::ErrorKind::NotConnected
            ) => {}
        Err(error) => panic!("connection was not closed within deadline: {error}"),
    }
}

fn configure_socket(stream: &UnixStream) {
    stream
        .set_read_timeout(Some(IO_TIMEOUT))
        .expect("set socket read timeout");
    stream
        .set_write_timeout(Some(IO_TIMEOUT))
        .expect("set socket write timeout");
}

fn wait_for_bridge(
    bridge: &mut RuntimeBridge,
    description: &str,
    mut ready: impl FnMut(&BridgeProjection, &[BridgeEvent]) -> bool,
) -> Vec<BridgeEvent> {
    let deadline = Instant::now() + STATE_TIMEOUT;
    let mut observed = Vec::new();
    loop {
        observed.extend(bridge.drain_events());
        if ready(bridge.projection(), &observed) {
            return observed;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {description}; projection={:?}, events={observed:?}",
            bridge.projection()
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_for_terminal_marker(
    bridge: &mut RuntimeBridge,
    marker: &str,
    mut confirmed_offset: u64,
) -> u64 {
    let deadline = Instant::now() + STATE_TIMEOUT;
    let mut output = Vec::new();
    loop {
        for event in bridge.drain_events() {
            if let BridgeEvent::TerminalOutput(item) = event {
                confirmed_offset =
                    confirmed_offset.max(item.offset.saturating_add(item.bytes.len() as u64));
                output.extend_from_slice(&item.bytes);
            }
        }
        if String::from_utf8_lossy(&output).contains(marker) {
            return confirmed_offset;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for terminal marker {marker}; output={}",
            String::from_utf8_lossy(&output)
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_for_confirmed_terminal_idle(
    bridge: &mut RuntimeBridge,
    output_log: &Path,
    mut confirmed_offset: u64,
) -> u64 {
    let deadline = Instant::now() + STATE_TIMEOUT;
    let mut stable_checks = 0;
    loop {
        for event in bridge.drain_events() {
            if let BridgeEvent::TerminalOutput(item) = event {
                confirmed_offset =
                    confirmed_offset.max(item.offset.saturating_add(item.bytes.len() as u64));
            }
        }
        let file_len = fs::metadata(output_log)
            .map(|value| value.len())
            .unwrap_or(0);
        if file_len == confirmed_offset {
            stable_checks += 1;
            if stable_checks == 5 {
                return confirmed_offset;
            }
        } else {
            stable_checks = 0;
        }
        assert!(
            Instant::now() < deadline,
            "terminal did not settle at a confirmed offset: confirmed={confirmed_offset}, file={file_len}"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

struct RestartRecovery {
    first_output_offset: u64,
    saw_resumed_event: bool,
}

fn wait_for_restart_recovery(
    bridge: &mut RuntimeBridge,
    marker: &str,
    previous_cursor: u64,
) -> RestartRecovery {
    let deadline = Instant::now() + STATE_TIMEOUT;
    let mut output = Vec::new();
    let mut first_output_offset = None;
    let mut saw_resumed_event = false;
    loop {
        for event in bridge.drain_events() {
            match event {
                BridgeEvent::TerminalOutput(item) => {
                    first_output_offset.get_or_insert(item.offset);
                    output.extend_from_slice(&item.bytes);
                }
                BridgeEvent::RuntimeEvent(event) if event.seq > previous_cursor => {
                    saw_resumed_event = true;
                }
                _ => {}
            }
        }
        if saw_resumed_event && String::from_utf8_lossy(&output).contains(marker) {
            return RestartRecovery {
                first_output_offset: first_output_offset.expect("restart terminal output"),
                saw_resumed_event,
            };
        }
        assert!(
            Instant::now() < deadline,
            "restart recovery timed out; cursor={}, output={}",
            bridge.projection().event_cursor,
            String::from_utf8_lossy(&output)
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_for_control_success(control: &mut InteractiveControl, after_seq: u64) {
    let deadline = Instant::now() + STATE_TIMEOUT;
    let mut request_id = 92_u64;
    loop {
        control.send(ControlMessage::request(
            RequestId::from(request_id),
            Method::SESSION_LIST,
            json!({ "afterSeq": after_seq }),
        ));
        let response = control.recv(PROCESS_TIMEOUT);
        if response_is_success(&response, request_id) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "control-stdio did not reconnect: {response:?}"
        );
        request_id += 1;
        thread::sleep(Duration::from_millis(50));
    }
}

fn response_is_success(message: &ControlMessage, expected_id: u64) -> bool {
    matches!(
        message,
        ControlMessage::Response {
            request_id,
            ok: true,
            ..
        } if request_id.as_u64() == expected_id
    )
}

fn assert_control_success(message: ControlMessage, expected_id: u64) {
    assert!(
        response_is_success(&message, expected_id),
        "expected successful response {expected_id}, got {message:?}"
    );
}

fn assert_control_error(message: ControlMessage, expected_id: u64, expected_code: &str) {
    match message {
        ControlMessage::Response {
            request_id,
            ok: false,
            error: Some(error),
            ..
        } => {
            assert_eq!(request_id.as_u64(), expected_id);
            assert_eq!(error.code, expected_code);
        }
        other => panic!("expected failed response {expected_id}, got {other:?}"),
    }
}

struct InteractiveControl {
    child: Child,
    stdin: Option<std::process::ChildStdin>,
    lines: Receiver<String>,
    reader: Option<JoinHandle<()>>,
}

impl InteractiveControl {
    fn start(cli: &Path, data_dir: &Path) -> Self {
        let mut child = Command::new(cli)
            .args(["control-stdio", "--data-dir"])
            .arg(data_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn persistent control-stdio");
        let stdin = child.stdin.take().expect("control stdin");
        let stdout = child.stdout.take().expect("control stdout");
        let (line_tx, lines) = mpsc::channel();
        let reader = thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else {
                    return;
                };
                if line_tx.send(line).is_err() {
                    return;
                }
            }
        });
        Self {
            child,
            stdin: Some(stdin),
            lines,
            reader: Some(reader),
        }
    }

    fn send(&mut self, message: ControlMessage) {
        let stdin = self.stdin.as_mut().expect("control stdin open");
        serde_json::to_writer(&mut *stdin, &message).expect("write control request");
        stdin.write_all(b"\n").expect("write control newline");
        stdin.flush().expect("flush control request");
    }

    fn recv(&self, timeout: Duration) -> ControlMessage {
        let line = self
            .lines
            .recv_timeout(timeout)
            .expect("control response before deadline");
        serde_json::from_str(&line).expect("decode control response")
    }

    fn assert_no_response(&self, timeout: Duration) {
        assert!(
            matches!(
                self.lines.recv_timeout(timeout),
                Err(RecvTimeoutError::Timeout)
            ),
            "control request produced more than one response"
        );
    }

    fn stop(&mut self) {
        self.stdin.take();
        if !wait_for_child(&mut self.child, Duration::from_secs(2)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

impl Drop for InteractiveControl {
    fn drop(&mut self) {
        self.stop();
    }
}

struct E2eFixture {
    _temp: TempDir,
    data_dir: PathBuf,
    cli: PathBuf,
    daemon: PathBuf,
    daemon_child: Option<Child>,
    daemon_pids: Vec<i32>,
    holder_pids: Vec<i32>,
    sessions: Vec<String>,
    daemon_logs: Vec<PathBuf>,
    cleaned: bool,
}

impl E2eFixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("Task19 tempdir");
        let data_dir = temp.path().join("data");
        let bin_dir = temp.path().join("bin");
        fs::create_dir(&data_dir).expect("create Task19 data dir");
        fs::create_dir(&bin_dir).expect("create Task19 bin dir");
        let canonical_cli =
            fs::canonicalize(env!("CARGO_BIN_EXE_homie")).expect("canonical homie binary");
        let canonical_bin_dir = canonical_cli.parent().expect("Cargo target bin directory");
        let canonical_daemon = fs::canonicalize(canonical_bin_dir.join("homie-runtime-daemon"))
            .expect("explicitly prebuilt daemon binary");
        let canonical_holder = fs::canonicalize(canonical_bin_dir.join("homie-runtime-holder"))
            .expect("explicitly prebuilt holder binary");
        let cli = copy_executable(&canonical_cli, &bin_dir.join("homie"));
        let daemon = copy_executable(&canonical_daemon, &bin_dir.join("homie-runtime-daemon"));
        copy_executable(&canonical_holder, &bin_dir.join("homie-runtime-holder"));
        assert_eq!(
            fs::read(&canonical_daemon).expect("read canonical daemon"),
            fs::read(&daemon).expect("read copied daemon"),
            "app and CLI sibling daemon hashes must be identical"
        );

        Self {
            _temp: temp,
            data_dir: fs::canonicalize(data_dir).expect("canonical Task19 data dir"),
            cli,
            daemon,
            daemon_child: None,
            daemon_pids: Vec::new(),
            holder_pids: Vec::new(),
            sessions: Vec::new(),
            daemon_logs: Vec::new(),
            cleaned: false,
        }
    }

    fn socket_path(&self) -> PathBuf {
        self.data_dir.join("runtime/daemon.sock")
    }

    fn start_daemon(&mut self) {
        assert!(
            self.daemon_child.is_none(),
            "fixture already owns a daemon child"
        );
        let log_path = self
            ._temp
            .path()
            .join(format!("daemon-{}.log", self.daemon_pids.len() + 1));
        let stdout = File::create(&log_path).expect("create daemon log");
        let stderr = stdout.try_clone().expect("clone daemon log");
        let child = Command::new(&self.daemon)
            .arg("--data-dir")
            .arg(&self.data_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .expect("spawn real daemon");
        self.daemon_pids.push(child.id() as i32);
        self.daemon_logs.push(log_path);
        self.daemon_child = Some(child);
        self.wait_for_socket();
    }

    fn wait_for_socket(&mut self) {
        let deadline = Instant::now() + STATE_TIMEOUT;
        loop {
            if self.socket_path().exists() {
                return;
            }
            if let Some(status) = self
                .daemon_child
                .as_mut()
                .expect("daemon child")
                .try_wait()
                .expect("daemon status")
            {
                panic!("daemon exited before socket readiness: {status}");
            }
            assert!(
                Instant::now() < deadline,
                "daemon socket was not ready before deadline"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn current_daemon_pid(&self) -> Option<u32> {
        self.daemon_child.as_ref().map(Child::id)
    }

    fn stop_daemon(&mut self) -> std::process::ExitStatus {
        let mut child = self.daemon_child.take().expect("owned daemon child");
        // SAFETY: this positive PID belongs to the child owned by this fixture.
        assert_eq!(unsafe { libc::kill(child.id() as i32, libc::SIGTERM) }, 0);
        let status =
            wait_for_child_status(&mut child, PROCESS_TIMEOUT).expect("daemon exits after SIGTERM");
        wait_for_path_absent(&self.socket_path(), STATE_TIMEOUT);
        status
    }

    fn runtime_status(&self) -> Value {
        self.cli_json([
            "runtime",
            "status",
            "--data-dir",
            self.data_dir.to_str().expect("UTF-8 data dir"),
            "--json",
        ])
    }

    fn session_create(&self, title: &str) -> Value {
        self.cli_json([
            "session",
            "create",
            "--data-dir",
            self.data_dir.to_str().expect("UTF-8 data dir"),
            "--workspace",
            self.data_dir.to_str().expect("UTF-8 workspace"),
            "--title",
            title,
            "--json",
        ])
    }

    fn session_snapshot(&self, session_id: &str) -> Value {
        self.cli_json([
            "session",
            "snapshot",
            "--data-dir",
            self.data_dir.to_str().expect("UTF-8 data dir"),
            "--id",
            session_id,
        ])
    }

    fn events_after(&self, cursor: u64) -> Value {
        let cursor = cursor.to_string();
        self.cli_json([
            "events",
            "list",
            "--data-dir",
            self.data_dir.to_str().expect("UTF-8 data dir"),
            "--after-seq",
            &cursor,
        ])
    }

    fn cli_json<const N: usize>(&self, arguments: [&str; N]) -> Value {
        let mut command = Command::new(&self.cli);
        command.args(arguments);
        let output = command_output(&mut command, None, PROCESS_TIMEOUT);
        assert_success(&output, "homie CLI");
        serde_json::from_slice(&output.stdout).expect("decode homie CLI JSON")
    }

    fn mcp_tool(&self, name: &str, arguments: Value) -> Value {
        let request = json!({
            "jsonrpc": "2.0",
            "id": "task19",
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });
        let mut input = request.to_string();
        input.push('\n');
        let mut command = Command::new(&self.cli);
        command
            .arg("mcp-stdio")
            .arg("--data-dir")
            .arg(&self.data_dir);
        let output = command_output(&mut command, Some(input.as_bytes()), PROCESS_TIMEOUT);
        assert_success(&output, "MCP subprocess");
        let response: Value = serde_json::from_slice(&output.stdout).expect("decode MCP response");
        assert!(
            response.get("error").is_none(),
            "MCP tool {name} failed: {response}"
        );
        let text = required_str(
            response
                .pointer("/result/content/0")
                .expect("MCP text content"),
            "text",
        );
        serde_json::from_str(text).expect("decode MCP tool payload")
    }

    fn assert_safe_introspection(&self) {
        let mut paths = self.daemon_logs.clone();
        paths.push(self.data_dir.join("runtime/daemon.boot.log"));
        paths.push(self.data_dir.join("runtime/daemon.log"));
        for path in paths {
            let Ok(bytes) = fs::read(&path) else {
                continue;
            };
            assert!(
                !bytes
                    .windows(SECURITY_MARKER.len())
                    .any(|window| window == SECURITY_MARKER),
                "daemon introspection leaked hostile payload into {}",
                path.display()
            );
        }
        let process_listing = process_listing();
        assert!(
            !process_listing
                .as_bytes()
                .windows(SECURITY_MARKER.len())
                .any(|window| window == SECURITY_MARKER),
            "daemon introspection leaked hostile payload into process arguments"
        );
    }

    fn cleanup_and_assert_zero(&mut self) -> Result<(), String> {
        let mut errors = Vec::new();
        if !self.sessions.is_empty()
            && !self.socket_path().exists()
            && let Err(error) = self.start_daemon_for_cleanup()
        {
            errors.push(error);
        }
        for session_id in self.sessions.clone() {
            let mut command = Command::new(&self.cli);
            command
                .args(["session", "kill", "--data-dir"])
                .arg(&self.data_dir)
                .args(["--id", &session_id]);
            match command_output_result(&mut command, None, Duration::from_secs(5)) {
                Ok(output) if output.status.success() => {}
                Ok(output) => errors.push(format!(
                    "session {session_id} cleanup failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )),
                Err(error) => errors.push(format!("session {session_id} cleanup failed: {error}")),
            }
        }
        for pid in self.holder_pids.iter().copied() {
            terminate_pid(pid);
        }
        if let Some(mut daemon) = self.daemon_child.take() {
            terminate_pid(daemon.id() as i32);
            if !wait_for_child(&mut daemon, Duration::from_secs(3)) {
                let _ = daemon.kill();
                let _ = daemon.wait();
            }
        }
        for pid in associated_processes(&self.data_dir) {
            terminate_pid(pid);
        }

        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let associated = associated_processes(&self.data_dir);
            let known_alive = self
                .daemon_pids
                .iter()
                .chain(self.holder_pids.iter())
                .copied()
                .filter(|pid| process_exists(*pid))
                .collect::<Vec<_>>();
            if associated.is_empty() && known_alive.is_empty() {
                self.cleaned = true;
                return if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors.join("; "))
                };
            }
            if Instant::now() >= deadline {
                return Err(format!(
                    "temp-associated daemon/holder processes remain: associated={associated:?}, known={known_alive:?}; prior={}",
                    errors.join("; ")
                ));
            }
            for pid in associated.iter().chain(known_alive.iter()).copied() {
                // SAFETY: PIDs came from this fixture or process rows containing its unique data dir.
                unsafe {
                    libc::kill(pid, libc::SIGKILL);
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn start_daemon_for_cleanup(&mut self) -> Result<(), String> {
        if let Some(child) = self.daemon_child.as_mut()
            && child
                .try_wait()
                .map_err(|error| error.to_string())?
                .is_some()
        {
            self.daemon_child = None;
        }
        if self.daemon_child.is_none() {
            let child = Command::new(&self.daemon)
                .arg("--data-dir")
                .arg(&self.data_dir)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|error| error.to_string())?;
            self.daemon_pids.push(child.id() as i32);
            self.daemon_child = Some(child);
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if self.socket_path().exists() {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(10));
        }
        Err("cleanup daemon did not become ready".to_string())
    }
}

impl Drop for E2eFixture {
    fn drop(&mut self) {
        if !self.cleaned {
            let _ = self.cleanup_and_assert_zero();
        }
    }
}

fn copy_executable(source: &Path, destination: &Path) -> PathBuf {
    fs::copy(source, destination).expect("copy Task19 executable");
    fs::set_permissions(destination, fs::Permissions::from_mode(0o700))
        .expect("set Task19 executable permissions");
    fs::canonicalize(destination).expect("canonical copied executable")
}

fn command_output(command: &mut Command, input: Option<&[u8]>, timeout: Duration) -> Output {
    command_output_result(command, input, timeout)
        .unwrap_or_else(|error| panic!("subprocess failed: {error}"))
}

fn command_output_result(
    command: &mut Command,
    input: Option<&[u8]>,
    timeout: Duration,
) -> Result<Output, String> {
    command
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|error| error.to_string())?;
    if let Some(input) = input {
        let mut stdin = child.stdin.take().ok_or("subprocess stdin unavailable")?;
        stdin.write_all(input).map_err(|error| error.to_string())?;
    }
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().map_err(|error| error.to_string()),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("subprocess exceeded {timeout:?}"));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
}

fn assert_success(output: &Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn wait_for_child(child: &mut Child, timeout: Duration) -> bool {
    wait_for_child_status(child, timeout).is_some()
}

fn wait_for_child_status(child: &mut Child, timeout: Duration) -> Option<std::process::ExitStatus> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            Ok(None) | Err(_) => return None,
        }
    }
}

fn wait_for_path_absent(path: &Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while path.exists() {
        assert!(
            Instant::now() < deadline,
            "{} remained after deadline",
            path.display()
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn terminate_pid(pid: i32) {
    if !process_exists(pid) {
        return;
    }
    // SAFETY: callers pass PIDs owned by this fixture.
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    while process_exists(pid) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    if process_exists(pid) {
        // SAFETY: the same fixture-owned PID is still alive.
        unsafe {
            libc::kill(pid, libc::SIGKILL);
        }
    }
}

fn process_exists(pid: i32) -> bool {
    // SAFETY: kill(pid, 0) checks existence without sending a signal.
    unsafe { libc::kill(pid, 0) == 0 }
}

fn process_listing() -> String {
    let output = Command::new("/bin/ps")
        .args(["-axo", "pid=,command="])
        .output()
        .expect("list processes");
    String::from_utf8(output.stdout).expect("UTF-8 process listing")
}

fn associated_processes(data_dir: &Path) -> Vec<i32> {
    let data_dir = data_dir.to_string_lossy();
    process_listing()
        .lines()
        .filter(|line| {
            line.contains(data_dir.as_ref())
                && (line.contains("homie-runtime-daemon") || line.contains("homie-runtime-holder"))
        })
        .filter_map(|line| line.split_whitespace().next()?.parse().ok())
        .collect()
}

fn required_str<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing string {key}: {value}"))
}

fn required_u64(value: &Value, key: &str) -> u64 {
    value
        .get(key)
        .and_then(Value::as_u64)
        .unwrap_or_else(|| panic!("missing integer {key}: {value}"))
}

fn panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "non-string panic".to_string())
}
