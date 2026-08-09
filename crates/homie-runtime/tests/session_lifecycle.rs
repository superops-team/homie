use homie_proto::{NeedsInputKind, NeedsInputSource, SessionStatus};
use homie_runtime::{HolderPaths, HolderRequest, RuntimeConfig, RuntimeSupervisor, holder};
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::time::{Duration, Instant};

#[test]
fn runtime_spawn_shell_uses_live_pty() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = open_runtime(temp.path());

    let session = runtime
        .spawn_shell(temp.path(), Some("Lifecycle"))
        .expect("spawn shell");
    assert_eq!(session.status, "running");
    assert_eq!(
        runtime
            .session_status_projection(&session.id)
            .expect("status projection"),
        "running"
    );

    runtime
        .send_text(&session.id, "printf 'homie-live-pty\\n'", true)
        .expect("send text");
    assert_output_contains(&runtime, &session.id, "homie-live-pty");
    let all_output = runtime.read_output(&session.id).expect("read full output");
    let marker_offset = all_output.find("homie-live-pty").expect("marker offset") as u64;
    let (next_offset, replay) = runtime
        .read_output_range(&session.id, marker_offset, "homie-live-pty".len())
        .expect("read output range");
    assert_eq!(replay, b"homie-live-pty");
    assert_eq!(next_offset, marker_offset + replay.len() as u64);
    assert_screen_contains(&runtime, &session.id, "homie-live-pty");

    let archived = runtime.archive(&session.id).expect("archive");
    assert_eq!(archived.status, "archived");
    let hibernated = runtime.hibernate(&session.id).expect("hibernate");
    assert_eq!(hibernated.status, "hibernated");
    let running = runtime.wake(&session.id).expect("wake");
    assert_eq!(running.status, "running");

    let reopened = open_runtime(temp.path());
    let sessions = reopened.list_sessions().expect("list sessions");
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, session.id);
    assert_eq!(sessions[0].status, "running");
    assert_eq!(
        reopened
            .session_status_projection(&session.id)
            .expect("adopted status projection"),
        "running"
    );
    assert!(
        reopened
            .read_output(&session.id)
            .expect("read output after reopen")
            .contains("homie-live-pty")
    );
    reopened
        .terminate_session(&session.id)
        .expect("cleanup holder");
    let sessions = reopened.list_sessions().expect("list after terminate");
    assert_eq!(sessions[0].status, "exited");
}

#[test]
fn runtime_spawn_failure_does_not_persist_created_session() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = open_runtime(temp.path());

    let missing = temp.path().join("missing-cwd");
    let error = runtime
        .spawn_shell(&missing, Some("Missing"))
        .expect_err("invalid cwd must fail");
    assert!(
        error.to_string().contains("I/O error"),
        "unexpected error: {error}"
    );
    assert!(runtime.list_sessions().expect("list sessions").is_empty());
}

#[test]
fn runtime_holder_launch_failure_removes_created_session() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = RuntimeSupervisor::open_with_holder(
        RuntimeConfig {
            data_dir: temp.path().to_path_buf(),
        },
        temp.path().join("missing-holder"),
    )
    .expect("open runtime");

    runtime
        .spawn_shell(temp.path(), Some("Missing holder"))
        .expect_err("missing holder must fail");

    assert!(runtime.list_sessions().expect("list sessions").is_empty());
}

#[test]
fn runtime_reopen_can_adopt_holder_and_continue_session() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_id = {
        let runtime = open_runtime(temp.path());
        let session = runtime
            .spawn_shell(temp.path(), Some("Live"))
            .expect("spawn shell");
        session.id
    };

    let reopened = open_runtime(temp.path());
    let sessions = reopened.list_sessions().expect("list sessions");
    assert_eq!(sessions[0].status, "running");
    reopened
        .send_text(&session_id, "printf 'holder-survived\\n'", true)
        .expect("holder should survive supervisor drop");
    assert_output_contains(&reopened, &session_id, "holder-survived");
    reopened
        .terminate_session(&session_id)
        .expect("cleanup holder");
}

#[test]
fn runtime_reopen_marks_missing_holder_detached() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_id = {
        let runtime = open_runtime(temp.path());
        let session = runtime
            .spawn_shell(temp.path(), Some("Transient"))
            .expect("spawn shell");
        let holder = HolderPaths::new(temp.path(), &session.id);
        holder::request(&holder.socket, &HolderRequest::Terminate).expect("terminate holder only");
        assert_holder_files_removed(&holder);
        std::fs::remove_file(&holder.status_file).expect("remove status");
        session.id
    };

    let reopened = open_runtime(temp.path());
    let sessions = reopened.list_sessions().expect("list sessions");
    assert_eq!(sessions[0].id, session_id);
    assert_eq!(sessions[0].status, "detached");
    assert_eq!(
        reopened
            .session_status_projection(&session_id)
            .expect("detached status projection"),
        "detached"
    );
}

#[test]
fn runtime_reopen_marks_exited_holder_status_exited() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_id = {
        let runtime = open_runtime(temp.path());
        let session = runtime
            .spawn_shell(temp.path(), Some("Exited holder"))
            .expect("spawn shell");
        let holder = HolderPaths::new(temp.path(), &session.id);
        holder::request(&holder.socket, &HolderRequest::Terminate).expect("terminate holder only");
        assert_holder_files_removed(&holder);
        assert_holder_status(&holder, "exited");
        session.id
    };

    let reopened = open_runtime(temp.path());
    let sessions = reopened.list_sessions().expect("list sessions");
    assert_eq!(sessions[0].id, session_id);
    assert_eq!(sessions[0].status, "exited");
    assert_eq!(
        reopened
            .session_status_projection(&session_id)
            .expect("status projection"),
        "exited"
    );
}

#[test]
fn runtime_status_report_uses_headless_screen_and_reducer() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = open_runtime(temp.path());
    let session = runtime
        .spawn_shell(temp.path(), Some("Status"))
        .expect("spawn shell");

    runtime
        .send_text(&session.id, "printf 'homie-status:working\\n'", true)
        .expect("send working");
    let report = assert_status(&runtime, &session.id, SessionStatus::Running);
    assert_eq!(
        report
            .screen_observation
            .as_ref()
            .map(|observation| observation.matched_rule_id.as_str()),
        Some("runtime-working-text")
    );

    runtime
        .send_text(
            &session.id,
            "printf 'Allow command?\\npress enter to confirm or esc to cancel\\n'",
            true,
        )
        .expect("send blocker");
    let report = assert_status(&runtime, &session.id, SessionStatus::NeedsInput);
    let detail = report.needs_input.expect("needs input detail");
    assert_eq!(detail.kind, NeedsInputKind::Approval);
    assert_eq!(detail.source, NeedsInputSource::ScreenScrape);
    assert!(detail.summary.contains("Allow command?"));

    runtime
        .send_text(&session.id, "printf '\\nhomie-status:idle\\n'", true)
        .expect("send idle");
    let report = assert_status(&runtime, &session.id, SessionStatus::Idle);
    assert_eq!(
        report
            .screen_observation
            .as_ref()
            .map(|observation| observation.matched_rule_id.as_str()),
        Some("runtime-idle-text")
    );

    runtime.terminate_session(&session.id).expect("terminate");
    let report = assert_status(&runtime, &session.id, SessionStatus::Exited);
    assert!(report.needs_input.is_none());
}

#[test]
fn runtime_terminate_marks_exited_and_removes_holder_files() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = open_runtime(temp.path());
    let session = runtime
        .spawn_shell(temp.path(), Some("Terminate"))
        .expect("spawn shell");
    let holder = HolderPaths::new(temp.path(), &session.id);
    assert!(holder.socket.exists(), "holder socket should exist");
    assert!(holder.pid_file.exists(), "holder pid file should exist");

    runtime
        .terminate_session(&session.id)
        .expect("terminate session");
    assert_holder_files_removed(&holder);
    assert_holder_status(&holder, "exited");

    let sessions = runtime.list_sessions().expect("list sessions");
    assert_eq!(sessions[0].status, "exited");
    assert_eq!(
        runtime
            .session_status_projection(&session.id)
            .expect("status projection"),
        "exited"
    );
}

#[cfg(unix)]
#[test]
fn runtime_terminate_kills_detached_child_tree() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = open_runtime(temp.path());
    let session = runtime
        .spawn_shell(temp.path(), Some("Tree"))
        .expect("spawn shell");
    let holder = HolderPaths::new(temp.path(), &session.id);

    let script = "import os,signal,subprocess,sys,time\np=subprocess.Popen([sys.executable,'-c','import signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); print(\\\"child-ready\\\", flush=True); time.sleep(60)'], preexec_fn=os.setsid)\nprint(f'child-pid:{p.pid}', flush=True)\ntime.sleep(60)\n";
    runtime
        .send_text(
            &session.id,
            &format!("python3 -c {}", shell_quote(script)),
            true,
        )
        .expect("start detached child");
    let child_pid = wait_child_pid(&runtime, &session.id);
    assert_process_alive(child_pid);
    let stat = holder::request(&holder.socket, &HolderRequest::Stat).expect("stat");
    assert!(
        stat.tree_size.unwrap_or_default() >= 2,
        "holder stat should see root plus child tree: {stat:?}"
    );

    runtime
        .terminate_session(&session.id)
        .expect("terminate session");
    assert_process_gone(child_pid);
    assert_holder_files_removed(&holder);
}

#[test]
fn runtime_holder_stat_tracks_resize_and_log_offsets() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = open_runtime(temp.path());
    let session = runtime
        .spawn_shell(temp.path(), Some("Stat"))
        .expect("spawn shell");
    let holder = HolderPaths::new(temp.path(), &session.id);

    let initial = holder::request(&holder.socket, &HolderRequest::Stat).expect("stat");
    assert_eq!(initial.cols, Some(120));
    assert_eq!(initial.rows, Some(40));
    assert_eq!(initial.epoch_offset, Some(0));
    assert_eq!(initial.log_offset, Some(0));

    runtime
        .send_text(&session.id, "printf 'offset-check\\n'", true)
        .expect("send text");
    assert_output_contains(&runtime, &session.id, "offset-check");
    let after_output = holder::request(&holder.socket, &HolderRequest::Stat).expect("stat");
    assert!(
        after_output.log_offset.unwrap_or_default() > after_output.epoch_offset.unwrap_or_default(),
        "expected log offset to advance: {after_output:?}"
    );

    runtime
        .resize_session(&session.id, 100, 30)
        .expect("resize session");
    let resized = holder::request(&holder.socket, &HolderRequest::Stat).expect("stat");
    assert_eq!(resized.cols, Some(100));
    assert_eq!(resized.rows, Some(30));

    runtime.terminate_session(&session.id).expect("terminate");
}

#[test]
fn runtime_holder_accepts_arbitrary_raw_bytes() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = open_runtime(temp.path());
    let session = runtime
        .spawn_shell(temp.path(), Some("Raw bytes"))
        .expect("spawn shell");

    runtime
        .send_bytes(&session.id, &[0xff, 0x80, 0x00])
        .expect("send raw bytes");

    runtime.terminate_session(&session.id).expect("terminate");
}

#[test]
fn runtime_reopen_snapshot_combines_registry_holder_status_and_replay() {
    let temp = tempfile::tempdir().expect("tempdir");
    let (session_id, marker_offset) = {
        let runtime = open_runtime(temp.path());
        let session = runtime
            .spawn_shell(temp.path(), Some("Snapshot"))
            .expect("spawn shell");
        runtime
            .send_text(&session.id, "printf 'snapshot-ready\\n'", true)
            .expect("send text");
        assert_output_contains(&runtime, &session.id, "snapshot-ready");
        let output = runtime.read_output(&session.id).expect("output");
        let marker_offset = output.find("snapshot-ready").expect("marker") as u64;
        (session.id, marker_offset)
    };

    let reopened = open_runtime(temp.path());
    let snapshot = reopened
        .session_snapshot(&session_id, marker_offset, "snapshot-ready".len())
        .expect("snapshot");
    assert_eq!(snapshot.session.id, session_id);
    assert_eq!(snapshot.status.status, SessionStatus::Running);
    assert_eq!(snapshot.output, b"snapshot-ready");
    assert_eq!(
        snapshot.output_offset,
        marker_offset + "snapshot-ready".len() as u64
    );
    let holder = snapshot.holder.expect("holder snapshot");
    assert_eq!(holder.status.as_deref(), Some("running"));
    assert_eq!(holder.cols, Some(120));
    assert_eq!(holder.rows, Some(40));
    assert!(
        holder.log_offset.unwrap_or_default() >= snapshot.output_offset,
        "holder log offset should cover replayed bytes: {holder:?}"
    );

    reopened
        .terminate_session(&session_id)
        .expect("cleanup holder");
}

#[test]
fn runtime_screen_checkpoint_survives_supervisor_reopen() {
    let temp = tempfile::tempdir().expect("tempdir");
    let (session_id, checkpoint) = {
        let runtime = open_runtime(temp.path());
        let session = runtime
            .spawn_shell(temp.path(), Some("Checkpoint"))
            .expect("spawn shell");
        runtime
            .send_text(&session.id, "printf 'checkpoint-line\\n'", true)
            .expect("send text");
        assert_output_contains(&runtime, &session.id, "checkpoint-line");
        let checkpoint = runtime
            .write_screen_checkpoint(&session.id)
            .expect("write checkpoint");
        assert!(checkpoint.output_offset > 0);
        assert!(
            checkpoint
                .lines
                .iter()
                .any(|line| line.contains("checkpoint-line")),
            "checkpoint lines missing output: {checkpoint:?}"
        );
        (session.id, checkpoint)
    };

    let reopened = open_runtime(temp.path());
    let restored = reopened
        .read_screen_checkpoint(&session_id)
        .expect("read checkpoint")
        .expect("checkpoint exists");
    assert_eq!(restored, checkpoint);
    reopened
        .terminate_session(&session_id)
        .expect("cleanup holder");
}

#[test]
fn runtime_hibernate_stops_holder_and_wake_restarts_it() {
    let temp = tempfile::tempdir().expect("tempdir");
    let runtime = open_runtime(temp.path());
    let session = runtime
        .spawn_shell(temp.path(), Some("Hibernate"))
        .expect("spawn shell");
    let holder = HolderPaths::new(temp.path(), &session.id);
    assert!(holder.socket.exists(), "holder socket should exist");

    let hibernated = runtime.hibernate(&session.id).expect("hibernate");
    assert_eq!(hibernated.status, "hibernated");
    assert_holder_files_removed(&holder);
    assert_eq!(
        runtime
            .session_status_projection(&session.id)
            .expect("status projection"),
        "hibernated"
    );

    let running = runtime.wake(&session.id).expect("wake");
    assert_eq!(running.status, "running");
    assert_holder_running(&holder);
    runtime
        .send_text(&session.id, "printf 'woke-again\\n'", true)
        .expect("send after wake");
    assert_output_contains(&runtime, &session.id, "woke-again");

    runtime.terminate_session(&session.id).expect("terminate");
}

fn assert_screen_contains(runtime: &RuntimeSupervisor, session_id: &str, needle: &str) {
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut last = Vec::new();
    while Instant::now() < deadline {
        last = runtime.read_screen_lines(session_id).expect("read screen");
        if last.iter().any(|line| line.contains(needle)) {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("screen did not contain {needle:?}; last screen: {last:?}");
}

fn assert_output_contains(runtime: &RuntimeSupervisor, session_id: &str, needle: &str) {
    wait_output_contains(runtime, session_id, needle);
}

fn wait_output_contains(runtime: &RuntimeSupervisor, session_id: &str, needle: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut last = String::new();
    while Instant::now() < deadline {
        last = runtime.read_output(session_id).expect("read output");
        if last.contains(needle) {
            return last;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("output did not contain {needle:?}; last output: {last:?}");
}

#[cfg(unix)]
fn wait_child_pid(runtime: &RuntimeSupervisor, session_id: &str) -> i32 {
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut last = String::new();
    while Instant::now() < deadline {
        last = runtime.read_output(session_id).expect("read output");
        if let Some(pid) = parse_child_pid(&last) {
            return pid;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("output did not contain a real child pid; last output: {last:?}");
}

fn open_runtime(data_dir: &Path) -> RuntimeSupervisor {
    RuntimeSupervisor::open_with_holder(
        RuntimeConfig {
            data_dir: data_dir.to_path_buf(),
        },
        holder_binary(),
    )
    .expect("open runtime")
}

fn holder_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_homie-runtime-holder"))
}

fn assert_status(
    runtime: &RuntimeSupervisor,
    session_id: &str,
    expected: SessionStatus,
) -> homie_runtime::SessionStatusReport {
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut last = None;
    while Instant::now() < deadline {
        let report = runtime
            .session_status_report(session_id)
            .expect("status report");
        if report.status == expected {
            return report;
        }
        last = Some(report);
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("status did not become {expected:?}; last report: {last:?}");
}

fn assert_holder_files_removed(holder: &HolderPaths) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if !holder.socket.exists() && !holder.pid_file.exists() {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!(
        "holder files were not removed: socket={}, pid_file={}",
        holder.socket.display(),
        holder.pid_file.display()
    );
}

fn assert_holder_running(holder: &HolderPaths) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if holder.socket.exists()
            && holder::request(&holder.socket, &HolderRequest::Stat)
                .map(|response| response.status.as_deref() == Some("running"))
                .unwrap_or(false)
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("holder did not become running: {}", holder.socket.display());
}

fn assert_holder_status(holder: &HolderPaths, expected_prefix: &str) {
    let status = std::fs::read_to_string(&holder.status_file).expect("holder status file");
    assert!(
        status.trim().starts_with(expected_prefix),
        "unexpected holder status: {status:?}"
    );
}

#[cfg(unix)]
fn shell_quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', "'\\''"))
}

#[cfg(unix)]
fn parse_child_pid(output: &str) -> Option<i32> {
    output.lines().filter_map(parse_child_pid_line).next_back()
}

#[cfg(unix)]
fn parse_child_pid_line(line: &str) -> Option<i32> {
    let (_, rest) = line.split_once("child-pid:")?;
    let digits = rest
        .chars()
        .skip_while(|ch| ch.is_whitespace())
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    digits.parse().ok()
}

#[cfg(unix)]
fn assert_process_alive(pid: i32) {
    assert!(process_exists(pid), "expected process {pid} to be alive");
}

#[cfg(unix)]
fn assert_process_gone(pid: i32) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if !process_exists(pid) {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("process {pid} remained alive after holder tree termination");
}

#[cfg(unix)]
fn process_exists(pid: i32) -> bool {
    // SAFETY: kill(pid, 0) performs existence/permission checking only.
    unsafe { libc::kill(pid, 0) == 0 }
}
