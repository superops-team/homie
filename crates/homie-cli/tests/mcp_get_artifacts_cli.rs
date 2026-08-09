use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

mod support;

#[test]
fn mcp_get_artifacts_reads_real_session_output() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let session_id = create_session(temp.path(), "Artifacts MCP");
    send_text(
        temp.path(),
        &session_id,
        "echo https://github.example/repo/pull/42 http://localhost:5173 https://example.invalid/docs",
    );

    let artifacts = wait_for_artifacts(temp.path(), &session_id, "localhost:5173");
    assert_eq!(artifacts["sessionId"], session_id);
    assert!(
        artifacts["artifacts"]
            .as_array()
            .expect("artifacts")
            .iter()
            .any(|artifact| artifact["url"] == "https://github.example/repo/pull/42"),
        "missing PR artifact: {artifacts}"
    );
    assert!(
        artifacts["artifacts"]
            .as_array()
            .expect("artifacts")
            .iter()
            .any(|artifact| artifact["url"] == "http://localhost:5173"),
        "missing preview artifact: {artifacts}"
    );
    assert!(
        artifacts["artifacts"]
            .as_array()
            .expect("artifacts")
            .iter()
            .any(|artifact| artifact["url"] == "https://example.invalid/docs"),
        "missing link artifact: {artifacts}"
    );
    assert!(
        artifacts["listeningPorts"]
            .as_array()
            .expect("listeningPorts")
            .iter()
            .any(|port| port["port"] == 5173 && port["url"] == "http://localhost:5173"),
        "missing listening port: {artifacts}"
    );
    kill_session(temp.path(), &session_id);
}

#[test]
fn missing_session_id_returns_invalid_params() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let response = mcp_response(temp.path(), "get_artifacts", serde_json::json!({}));
    assert_eq!(response["error"]["code"], -32602);
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("session_id or sessionId is required")
    );
}

fn create_session(data_dir: &std::path::Path, title: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args([
            "session",
            "create",
            "--data-dir",
            data_dir.to_str().unwrap(),
            "--workspace",
            data_dir.to_str().unwrap(),
            "--title",
            title,
            "--json",
        ])
        .output()
        .expect("session create");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).expect("json")["id"]
        .as_str()
        .expect("id")
        .to_string()
}

fn send_text(data_dir: &std::path::Path, session_id: &str, text: &str) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args(["control-stdio", "--data-dir", data_dir.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("control stdio");
    {
        let stdin = child.stdin.as_mut().expect("stdin");
        let line = serde_json::json!({
            "type": "request",
            "requestId": 1,
            "method": "session.send_text",
            "params": {
                "sessionId": session_id,
                "text": text,
                "submit": true
            }
        })
        .to_string();
        stdin.write_all(line.as_bytes()).expect("write");
        stdin.write_all(b"\n").expect("newline");
    }
    let output = child.wait_with_output().expect("wait");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn kill_session(data_dir: &std::path::Path, session_id: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args([
            "session",
            "kill",
            "--data-dir",
            data_dir.to_str().unwrap(),
            "--id",
            session_id,
        ])
        .output()
        .expect("session kill");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn wait_for_artifacts(data_dir: &std::path::Path, session_id: &str, needle: &str) -> Value {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let value = mcp_tool(
            data_dir,
            "get_artifacts",
            serde_json::json!({ "session_id": session_id }),
        );
        if value["artifacts"]
            .as_array()
            .expect("artifacts")
            .iter()
            .any(|artifact| {
                artifact["url"]
                    .as_str()
                    .unwrap_or_default()
                    .contains(needle)
            })
        {
            return value;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {needle}: {value}"
        );
        thread::sleep(Duration::from_millis(100));
    }
}

fn mcp_tool(data_dir: &std::path::Path, name: &str, arguments: Value) -> Value {
    let response = mcp_response(data_dir, name, arguments);
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("tool text");
    serde_json::from_str(text).expect("tool payload")
}

fn mcp_response(data_dir: &std::path::Path, name: &str, arguments: Value) -> Value {
    mcp_roundtrip(
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
    )
}

fn mcp_roundtrip(data_dir: &std::path::Path, line: &str) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_homie"))
        .arg("mcp-stdio")
        .arg("--data-dir")
        .arg(data_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn mcp");
    {
        let stdin = child.stdin.as_mut().expect("stdin");
        stdin.write_all(line.as_bytes()).expect("line");
        stdin.write_all(b"\n").expect("newline");
    }
    let output = child.wait_with_output().expect("output");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("json")
}
