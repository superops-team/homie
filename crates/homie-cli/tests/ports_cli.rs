use serde_json::Value;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

mod support;

#[test]
fn lists_ports_from_runtime_session_output() {
    let temp = tempfile::tempdir().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let session_id = create_session(temp.path(), "Ports CLI");
    send_text(temp.path(), &session_id, "echo http://localhost:5173");
    let ports = wait_for_ports(temp.path(), "localhost:5173");
    let rows = ports["ports"].as_array().expect("ports array");
    assert!(rows.iter().any(|row| {
        row["port"] == 5173
            && row["sessionId"] == session_id
            && row["sessionTitle"] == "Ports CLI"
            && row["url"] == "http://localhost:5173"
    }));
}

#[test]
fn ports_cli_reports_empty_state() {
    let temp = tempfile::tempdir().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let ports = homie_json([
        "ports",
        "--data-dir",
        temp.path().to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(ports["ports"].as_array().expect("ports array").len(), 0);
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
    let value: Value = serde_json::from_slice(&output.stdout).expect("session json");
    value["id"].as_str().expect("session id").to_string()
}

fn send_text(data_dir: &std::path::Path, session_id: &str, text: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args(["control-stdio", "--data-dir", data_dir.to_str().unwrap()])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("control stdio");
    let mut child = output;
    {
        let stdin = child.stdin.as_mut().expect("stdin");
        use std::io::Write;
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

fn wait_for_ports(data_dir: &std::path::Path, needle: &str) -> Value {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let value = homie_json(["ports", "--data-dir", data_dir.to_str().unwrap(), "--json"]);
        if value["ports"]
            .as_array()
            .expect("ports")
            .iter()
            .any(|row| row["url"].as_str().unwrap_or_default().contains(needle))
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

fn homie_json<const N: usize>(args: [&str; N]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args(args)
        .output()
        .expect("homie");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("json")
}
