use std::process::Command;

use serde_json::Value;

mod support;

#[test]
fn events_list_reads_events_created_by_the_real_daemon() {
    let temp = tempfile::tempdir().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let session_id = create_session(temp.path());

    let output = homie_json([
        "events",
        "list",
        "--data-dir",
        temp.path().to_str().expect("data dir"),
        "--after-seq",
        "0",
        "--event",
        "session.spawned",
    ]);

    let events = output["events"].as_array().expect("events");
    assert!(
        events.iter().any(|event| {
            event["event"] == "session.spawned" && event["sessionId"] == session_id
        })
    );
    assert!(output["cursor"]["nextSeq"].as_u64().unwrap_or_default() > 0);
}

#[test]
fn events_wait_returns_a_real_daemon_event_without_timing_out() {
    let temp = tempfile::tempdir().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let session_id = create_session(temp.path());
    report_turn_complete(temp.path(), &session_id);

    let output = homie_json([
        "events",
        "wait",
        "--data-dir",
        temp.path().to_str().expect("data dir"),
        "--after-seq",
        "0",
        "--event",
        "session.status",
        "--timeout-ms",
        "100",
    ]);

    assert_eq!(output["timedOut"], false);
    assert!(
        output["events"]
            .as_array()
            .expect("events")
            .iter()
            .any(|event| event["event"] == "session.status" && event["sessionId"] == session_id)
    );
}

fn create_session(data_dir: &std::path::Path) -> String {
    let output = homie_json([
        "session",
        "create",
        "--data-dir",
        data_dir.to_str().expect("data dir"),
        "--workspace",
        data_dir.to_str().expect("workspace"),
        "--title",
        "Events CLI",
        "--json",
    ]);
    output["id"].as_str().expect("session id").to_string()
}

fn report_turn_complete(data_dir: &std::path::Path, session_id: &str) {
    let payload = serde_json::json!({
        "type": "agent-turn-complete",
        "thread-id": session_id,
        "input-messages": ["done"]
    })
    .to_string();
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args([
            "notify",
            "--data-dir",
            data_dir.to_str().expect("data dir"),
            &payload,
        ])
        .output()
        .expect("notify");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn homie_json<const N: usize>(args: [&str; N]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args(args)
        .output()
        .expect("run homie");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("JSON output")
}
