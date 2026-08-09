use std::fs;
use std::process::Command;

use tempfile::TempDir;

mod support;

#[test]
fn session_history_scans_fixture_roots_through_cli() {
    let temp = TempDir::new().expect("tempdir");
    let data_dir = temp.path().join("data");
    let _runtime = support::RuntimeGuard::new(&data_dir);
    let claude = temp.path().join("claude");
    let codex = temp.path().join("codex");
    let cwd = temp.path().join("project");
    fs::create_dir_all(&cwd).expect("cwd");
    write_claude_fixture(&claude, &cwd);
    write_codex_fixture(&codex, &cwd);

    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args([
            "session",
            "history",
            "--data-dir",
            data_dir.to_str().unwrap(),
            "--claude-root",
            claude.to_str().unwrap(),
            "--codex-root",
            codex.to_str().unwrap(),
            "--tracked",
            "thread-id",
        ])
        .output()
        .expect("run session history");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("history json");
    let entries = value.as_array().expect("history array");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["agentKind"], "claude_code");
    assert_eq!(entries[0]["title"], "Newest title");
}

fn write_claude_fixture(root: &std::path::Path, cwd: &std::path::Path) {
    let project = root.join("encoded-project");
    fs::create_dir_all(&project).expect("claude project");
    fs::write(
        project.join("12345678-1234-1234-1234-123456789abc.jsonl"),
        format!(
            "{{\"type\":\"user\",\"cwd\":{},\"message\":{{\"content\":[{{\"type\":\"text\",\"text\":\"first prompt\"}}]}}}}\n{{\"type\":\"ai-title\",\"aiTitle\":\"Newest title\"}}\n",
            serde_json::to_string(&cwd.to_string_lossy()).expect("cwd json")
        ),
    )
    .expect("write claude");
}

fn write_codex_fixture(root: &std::path::Path, cwd: &std::path::Path) {
    let day = root.join("2026/07/22");
    fs::create_dir_all(&day).expect("codex day");
    fs::write(
        day.join("rollout-2026-07-22-thread-id.jsonl"),
        format!(
            "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"thread-id\",\"cwd\":{}}}}}\n{{\"type\":\"event_msg\",\"payload\":{{\"type\":\"user_message\",\"message\":\"Build\"}}}}\n",
            serde_json::to_string(&cwd.to_string_lossy()).expect("cwd json")
        ),
    )
    .expect("write codex");
}
