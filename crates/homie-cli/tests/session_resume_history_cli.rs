use std::process::Command;

use tempfile::TempDir;

mod support;

#[test]
fn session_resume_history_spawns_runtime_session_from_history_identity() {
    let temp = TempDir::new().expect("tempdir");
    let data_dir = temp.path().join("data");
    let _runtime = support::RuntimeGuard::new(&data_dir);
    let cwd = temp.path().join("project");
    let claude_root = temp.path().join("claude");
    let codex_root = temp.path().join("codex");
    std::fs::create_dir_all(&cwd).expect("cwd");
    std::fs::create_dir_all(&claude_root).expect("claude root");
    write_codex_history(&codex_root, &cwd);
    import_history(&data_dir, &claude_root, &codex_root);

    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args([
            "session",
            "resume-history",
            "--data-dir",
            data_dir.to_str().unwrap(),
            "--agent-kind",
            "codex",
            "--external-id",
            "thread-id",
            "--cwd",
            cwd.to_str().unwrap(),
            "--title",
            "Resume Thread",
        ])
        .output()
        .expect("run resume-history");

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(value["title"], "Resume Thread");
    assert_eq!(value["status"], "running");

    let list = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args([
            "session",
            "list",
            "--data-dir",
            data_dir.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("list sessions");
    assert!(list.status.success());
    let sessions: serde_json::Value = serde_json::from_slice(&list.stdout).expect("sessions");
    assert_eq!(sessions.as_array().unwrap().len(), 1);
}

fn write_codex_history(root: &std::path::Path, cwd: &std::path::Path) {
    let day = root.join("2026/08/08");
    std::fs::create_dir_all(&day).expect("codex day");
    std::fs::write(
        day.join("rollout-2026-08-08-thread-id.jsonl"),
        format!(
            "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"thread-id\",\"cwd\":{}}}}}\n{{\"type\":\"event_msg\",\"payload\":{{\"type\":\"user_message\",\"message\":\"Resume me\"}}}}\n",
            serde_json::to_string(&cwd.to_string_lossy()).expect("cwd JSON")
        ),
    )
    .expect("codex history");
}

fn import_history(
    data_dir: &std::path::Path,
    claude_root: &std::path::Path,
    codex_root: &std::path::Path,
) {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args([
            "session",
            "history",
            "--data-dir",
            data_dir.to_str().expect("data dir"),
            "--claude-root",
            claude_root.to_str().expect("claude root"),
            "--codex-root",
            codex_root.to_str().expect("codex root"),
        ])
        .output()
        .expect("import history");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}
