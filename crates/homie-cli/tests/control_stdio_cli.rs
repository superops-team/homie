use std::io::Write as _;
use std::process::{Command, Stdio};

use homie_proto::{ControlMessage, Method, RequestId};
use serde_json::{Value, json};

mod support;

const MAX_CONTROL_MESSAGE_BYTES: usize = 4 * 1024 * 1024;

#[test]
fn control_stdio_forwards_requests_to_one_real_daemon_with_correlation() {
    let temp = tempfile::tempdir().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let requests = [
        ControlMessage::request(RequestId::from(41), Method::SESSION_LIST, json!({})),
        ControlMessage::request(RequestId::from(42), Method::STATE_SNAPSHOT, json!({})),
    ];

    let output = control_roundtrip(temp.path(), &requests);
    let responses = response_map(&output.stdout);

    assert!(output.status.success(), "stderr={}", stderr(&output));
    assert_eq!(responses[&41]["requestId"], 41);
    assert_eq!(responses[&41]["result"], json!([]));
    assert_eq!(responses[&42]["requestId"], 42);
    assert_eq!(responses[&42]["result"]["sessions"], json!([]));
}

#[test]
fn control_stdio_preserves_daemon_method_not_found() {
    let temp = tempfile::tempdir().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let request =
        ControlMessage::request(RequestId::from(77), "future.unsupported", json!({"x": 1}));

    let output = control_roundtrip(temp.path(), &[request]);
    let responses = response_map(&output.stdout);

    assert!(output.status.success(), "stderr={}", stderr(&output));
    assert_eq!(responses[&77]["ok"], false);
    assert_eq!(responses[&77]["error"]["code"], "method_not_found");
}

#[test]
fn control_stdio_rejects_oversized_message_before_deserializing_json() {
    let temp = tempfile::tempdir().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let oversized_invalid_json = vec![b'['; MAX_CONTROL_MESSAGE_BYTES + 1];
    let mut child = control_child(temp.path());
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(&oversized_invalid_json)
        .expect("write oversized input");
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("control output");

    assert!(!output.status.success());
    let stderr = stderr(&output);
    assert!(stderr.contains("control message exceeds 4194304 bytes"));
    assert!(!stderr.contains("parse"));
}

fn control_roundtrip(
    data_dir: &std::path::Path,
    requests: &[ControlMessage],
) -> std::process::Output {
    let mut child = control_child(data_dir);
    let stdin = child.stdin.as_mut().expect("stdin");
    for request in requests {
        serde_json::to_writer(&mut *stdin, request).expect("request");
        stdin.write_all(b"\n").expect("newline");
    }
    drop(child.stdin.take());
    child.wait_with_output().expect("control output")
}

fn control_child(data_dir: &std::path::Path) -> std::process::Child {
    Command::new(env!("CARGO_BIN_EXE_homie"))
        .arg("control-stdio")
        .arg("--data-dir")
        .arg(data_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn control stdio")
}

fn response_map(stdout: &[u8]) -> std::collections::BTreeMap<u64, Value> {
    String::from_utf8_lossy(stdout)
        .lines()
        .map(|line| {
            let response = serde_json::from_str::<Value>(line).expect("response");
            let request_id = response["requestId"].as_u64().expect("request id");
            (request_id, response)
        })
        .collect()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
