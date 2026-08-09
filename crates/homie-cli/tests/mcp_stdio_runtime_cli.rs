use std::io::Write;
use std::io::{BufRead, BufReader};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use homie_proto::{ControlMessage, Method, RequestId};
use serde_json::Value;
use tempfile::TempDir;

mod support;

#[test]
fn lists_diri_runtime_tool_descriptors() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let tools = mcp_roundtrip(
        temp.path(),
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#,
    );
    let names = tools["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .map(|tool| tool["name"].as_str().expect("tool name"))
        .collect::<Vec<_>>();
    for expected in [
        "spawn_agent",
        "list_agents",
        "get_status",
        "send_prompt",
        "read_output",
        "whoami",
    ] {
        assert!(names.contains(&expected), "missing {expected}: {names:?}");
    }
    assert!(
        !names.contains(&"browser"),
        "unsupported browser: {names:?}"
    );
    assert!(
        !names.contains(&"test_run"),
        "unsupported test_run: {names:?}"
    );
}

#[test]
fn runtime_backed_mcp_tools_list_status_and_read_output() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let session_id = create_session(temp.path(), "MCP Runtime");

    let list = mcp_tool(temp.path(), "list_agents", serde_json::json!({}));
    let agents = list["agents"].as_array().expect("agents");
    assert!(
        agents
            .iter()
            .any(|agent| { agent["id"] == session_id && agent["title"] == "MCP Runtime" }),
        "session not listed: {list}"
    );

    let status = mcp_tool(
        temp.path(),
        "get_status",
        serde_json::json!({"sessionId": session_id}),
    );
    assert_eq!(status["sessionId"], session_id);
    assert_eq!(status["status"], "running");

    let output = mcp_tool(
        temp.path(),
        "read_output",
        serde_json::json!({"sessionId": session_id}),
    );
    assert_eq!(output["sessionId"], session_id);
    assert!(
        output["outputText"]
            .as_str()
            .unwrap_or_default()
            .contains("$ "),
        "expected shell prompt: {output}"
    );
}

#[test]
fn runtime_backed_mcp_tools_send_prompt_and_spawn_agent() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let session_id = create_session(temp.path(), "MCP Send");

    let sent = mcp_tool(
        temp.path(),
        "send_prompt",
        serde_json::json!({
            "sessionId": session_id,
            "text": "printf 'mcp-live\\n'",
            "submit": true
        }),
    );
    assert_eq!(sent["ok"], true);
    wait_for_output(temp.path(), &session_id, "mcp-live");

    let spawned = mcp_tool(
        temp.path(),
        "spawn_agent",
        serde_json::json!({
            "cwd": temp.path(),
            "title": "MCP Spawned"
        }),
    );
    let new_session = spawned["session"]["id"].as_str().expect("new session");
    assert_ne!(new_session, session_id);
    assert_eq!(spawned["session"]["title"], "MCP Spawned");
}

#[test]
fn unsupported_future_tools_return_safe_errors() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let response = mcp_roundtrip(
        temp.path(),
        r#"{"jsonrpc":"2.0","id":"x","method":"tools/call","params":{"name":"browser","arguments":{}}}"#,
    );
    assert_eq!(response["error"]["code"], -32601);
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("unsupported tool")
    );
}

#[test]
fn daemon_execution_and_transport_errors_remain_distinct() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());

    let execution = mcp_roundtrip(
        temp.path(),
        r#"{"jsonrpc":"2.0","id":"execution","method":"tools/call","params":{"name":"get_status","arguments":{"sessionId":"missing"}}}"#,
    );
    assert_eq!(execution["error"]["code"], -32000);

    let mut child = mcp_child(temp.path());
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout"));
    write_request(
        &mut child,
        r#"{"jsonrpc":"2.0","id":"ready","method":"tools/list","params":{}}"#,
    );
    let ready = read_response(&mut stdout);
    assert_eq!(ready["id"], "ready");

    shutdown_daemon(temp.path());
    thread::sleep(Duration::from_millis(100));
    write_request(
        &mut child,
        r#"{"jsonrpc":"2.0","id":"transport","method":"tools/call","params":{"name":"list_agents","arguments":{}}}"#,
    );
    let transport = read_response(&mut stdout);
    assert_eq!(transport["error"]["code"], -32001);

    drop(child.stdin.take());
    assert!(child.wait().expect("wait mcp").success());
}

#[test]
fn runtime_guard_cleans_holder_after_daemon_disappears_during_panic() {
    let temp = TempDir::new().expect("tempdir");
    let daemon_pid = AtomicI32::new(0);
    let holder_pid = AtomicI32::new(0);

    let panic_result = catch_unwind(AssertUnwindSafe(|| {
        let _runtime = support::RuntimeGuard::new(temp.path());
        let session_id = create_session(temp.path(), "Panic Cleanup");
        daemon_pid.store(runtime_daemon_pid(temp.path()), Ordering::SeqCst);
        holder_pid.store(
            session_holder_pid(temp.path(), &session_id),
            Ordering::SeqCst,
        );

        shutdown_daemon(temp.path());
        wait_for_condition("daemon shutdown", || {
            !temp.path().join("runtime/daemon.sock").exists()
                && !process_exists(daemon_pid.load(Ordering::SeqCst))
        });
        assert!(
            process_exists(holder_pid.load(Ordering::SeqCst)),
            "holder must survive daemon shutdown before guard cleanup"
        );
        panic!("exercise RuntimeGuard cleanup during unwind");
    }));

    let _ = wait_until(Duration::from_secs(3), || {
        associated_runtime_processes(temp.path()).is_empty()
            && !process_exists(daemon_pid.load(Ordering::SeqCst))
            && !process_exists(holder_pid.load(Ordering::SeqCst))
    });
    let remaining = associated_runtime_processes(temp.path());
    let daemon_count = remaining
        .iter()
        .filter(|(_, command)| command.contains("homie-runtime-daemon"))
        .count();
    let holder_count = remaining
        .iter()
        .filter(|(_, command)| command.contains("homie-runtime-holder"))
        .count();
    for (pid, _) in &remaining {
        terminate_process(*pid);
    }

    assert!(
        panic_result.is_err() && daemon_count == 0 && holder_count == 0,
        "panic cleanup failed: daemon={daemon_count}, holder={holder_count}, remaining={remaining:?}"
    );
}

fn create_session(data_dir: &std::path::Path, title: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .arg("session")
        .arg("create")
        .arg("--data-dir")
        .arg(data_dir)
        .arg("--workspace")
        .arg(data_dir)
        .arg("--title")
        .arg(title)
        .arg("--json")
        .output()
        .expect("session create");
    assert!(
        output.status.success(),
        "create failed stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("session json");
    value["id"].as_str().expect("session id").to_string()
}

fn runtime_daemon_pid(data_dir: &std::path::Path) -> i32 {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args(["runtime", "status", "--data-dir"])
        .arg(data_dir)
        .arg("--json")
        .output()
        .expect("runtime status");
    assert!(
        output.status.success(),
        "runtime status failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("runtime status JSON");
    i32::try_from(value["daemonPid"].as_u64().expect("daemon pid")).expect("daemon pid fits i32")
}

fn session_holder_pid(data_dir: &std::path::Path, session_id: &str) -> i32 {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args(["session", "snapshot", "--data-dir"])
        .arg(data_dir)
        .args(["--id", session_id])
        .output()
        .expect("session snapshot");
    assert!(
        output.status.success(),
        "session snapshot failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("session snapshot JSON");
    i32::try_from(value["holder"]["pid"].as_u64().expect("holder pid"))
        .expect("holder pid fits i32")
}

fn wait_for_output(data_dir: &std::path::Path, session_id: &str, needle: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let output = mcp_tool(
            data_dir,
            "read_output",
            serde_json::json!({"sessionId": session_id}),
        );
        if output["outputText"]
            .as_str()
            .unwrap_or_default()
            .contains(needle)
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {needle}: {output}"
        );
        thread::sleep(Duration::from_millis(100));
    }
}

fn mcp_tool(data_dir: &std::path::Path, name: &str, arguments: Value) -> Value {
    let response = mcp_roundtrip(
        data_dir,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": "call",
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        })
        .to_string(),
    );
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("tool text");
    serde_json::from_str(text).expect("tool payload")
}

fn mcp_roundtrip(data_dir: &std::path::Path, line: &str) -> Value {
    let mut child = mcp_child(data_dir);
    write_request(&mut child, line);
    drop(child.stdin.take());
    let output = child.wait_with_output().expect("wait output");
    assert!(
        output.status.success(),
        "mcp failed stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    serde_json::from_str(stdout.lines().next().expect("first line")).expect("json response")
}

fn mcp_child(data_dir: &std::path::Path) -> std::process::Child {
    Command::new(env!("CARGO_BIN_EXE_homie"))
        .arg("mcp-stdio")
        .arg("--data-dir")
        .arg(data_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn homie mcp-stdio")
}

fn write_request(child: &mut std::process::Child, line: &str) {
    let stdin = child.stdin.as_mut().expect("stdin");
    stdin.write_all(line.as_bytes()).expect("write line");
    stdin.write_all(b"\n").expect("write newline");
    stdin.flush().expect("flush request");
}

fn read_response(stdout: &mut impl BufRead) -> Value {
    let mut line = String::new();
    stdout.read_line(&mut line).expect("read response");
    serde_json::from_str(&line).expect("JSON response")
}

fn shutdown_daemon(data_dir: &std::path::Path) {
    let request = serde_json::to_string(&ControlMessage::request(
        RequestId::from(99),
        Method::DAEMON_SHUTDOWN,
        serde_json::json!({}),
    ))
    .expect("shutdown request");
    let mut child = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args(["control-stdio", "--data-dir"])
        .arg(data_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn shutdown");
    write_request(&mut child, &request);
    drop(child.stdin.take());
    let output = child.wait_with_output().expect("shutdown output");
    assert!(
        output.status.success(),
        "shutdown failed stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn wait_for_condition(description: &str, condition: impl Fn() -> bool) {
    assert!(
        wait_until(Duration::from_secs(10), condition),
        "timed out waiting for {description}"
    );
}

fn wait_until(timeout: Duration, condition: impl Fn() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        thread::sleep(Duration::from_millis(25));
    }
    condition()
}

fn process_exists(pid: i32) -> bool {
    pid > 0 && {
        // SAFETY: kill(pid, 0) checks process existence without sending a signal.
        unsafe { libc::kill(pid, 0) == 0 }
    }
}

fn associated_runtime_processes(data_dir: &std::path::Path) -> Vec<(i32, String)> {
    let output = Command::new("/bin/ps")
        .args(["-axo", "pid=,command="])
        .output()
        .expect("list processes");
    let data_dir = data_dir.to_string_lossy();
    String::from_utf8(output.stdout)
        .expect("UTF-8 process listing")
        .lines()
        .filter(|line| {
            line.contains(data_dir.as_ref())
                && (line.contains("homie-runtime-daemon") || line.contains("homie-runtime-holder"))
        })
        .filter_map(|line| {
            let pid = line.split_whitespace().next()?.parse().ok()?;
            Some((pid, line.to_string()))
        })
        .collect()
}

fn terminate_process(pid: i32) {
    // SAFETY: the PID was read from a process row containing this test's unique data directory.
    unsafe {
        libc::kill(pid, libc::SIGKILL);
    }
}
