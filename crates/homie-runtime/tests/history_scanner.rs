use std::collections::HashSet;
use std::fs;

use homie_runtime::history::{
    HistoryRoots, resume_command, scan_history, write_history_to_storage,
};
use homie_storage::{StorageConfig, open_or_create};
use tempfile::TempDir;

#[test]
fn scans_claude_and_codex_transcripts_into_history_entries() {
    let fixture = TempDir::new().expect("tempdir");
    let roots = fixture_roots(&fixture);
    let cwd = fixture.path().join("project");
    fs::create_dir_all(&cwd).expect("cwd");
    write_claude_fixture(&roots, &cwd);
    write_codex_fixture(&roots, &cwd);

    let entries = scan_history(&roots, &HashSet::new()).expect("scan history");

    assert_eq!(entries.len(), 2);
    let claude = entries
        .iter()
        .find(|entry| entry.agent_kind == "claude_code")
        .expect("claude entry");
    assert_eq!(claude.external_id, "12345678-1234-1234-1234-123456789abc");
    assert_eq!(claude.title.as_deref(), Some("Newest title"));
    assert!(claude.cwd_exists);
    assert!(
        claude
            .transcript_path
            .ends_with("12345678-1234-1234-1234-123456789abc.jsonl")
    );

    let codex = entries
        .iter()
        .find(|entry| entry.agent_kind == "codex")
        .expect("codex entry");
    assert_eq!(codex.external_id, "thread-id");
    assert_eq!(codex.title.as_deref(), Some("Build the thing carefully"));

    let mut tracked = HashSet::new();
    tracked.insert("thread-id".to_string());
    let untracked = scan_history(&roots, &tracked).expect("scan untracked");
    assert_eq!(untracked.len(), 1);
    assert_eq!(untracked[0].agent_kind, "claude_code");
}

#[test]
fn scanned_history_writes_to_storage_and_builds_resume_commands() {
    let fixture = TempDir::new().expect("tempdir");
    let roots = fixture_roots(&fixture);
    let cwd = fixture.path().join("project");
    fs::create_dir_all(&cwd).expect("cwd");
    write_claude_fixture(&roots, &cwd);
    write_codex_fixture(&roots, &cwd);

    let entries = scan_history(&roots, &HashSet::new()).expect("scan history");
    let storage = open_or_create(StorageConfig {
        data_dir: fixture.path().join("homie-data"),
    })
    .expect("storage");
    storage.migrate().expect("migrate");
    storage.seed_defaults().expect("seed");

    let written = write_history_to_storage(&storage, &entries).expect("write history");
    assert_eq!(written.len(), 2);

    let stored = storage
        .list_history_entries(10)
        .expect("list stored history");
    assert_eq!(stored.len(), 2);
    assert!(stored.iter().any(|entry| entry.external_id == "thread-id"));
    assert!(stored.iter().any(|entry| {
        entry.external_id == "12345678-1234-1234-1234-123456789abc"
            && entry.title.as_deref() == Some("Newest title")
    }));

    let claude = entries
        .iter()
        .find(|entry| entry.agent_kind == "claude_code")
        .expect("claude entry");
    let codex = entries
        .iter()
        .find(|entry| entry.agent_kind == "codex")
        .expect("codex entry");
    assert_eq!(
        resume_command(claude).as_deref(),
        Some("claude --resume 12345678-1234-1234-1234-123456789abc")
    );
    assert_eq!(
        resume_command(codex).as_deref(),
        Some("codex resume thread-id")
    );
}

fn fixture_roots(temp: &TempDir) -> HistoryRoots {
    HistoryRoots {
        claude: temp.path().join("claude"),
        codex: temp.path().join("codex"),
    }
}

fn write_claude_fixture(roots: &HistoryRoots, cwd: &std::path::Path) {
    let project = roots.claude.join("encoded-project");
    fs::create_dir_all(&project).expect("claude project");
    let transcript = project.join("12345678-1234-1234-1234-123456789abc.jsonl");
    fs::write(
        &transcript,
        format!(
            "{{\"type\":\"user\",\"cwd\":{},\"message\":{{\"content\":[{{\"type\":\"text\",\"text\":\"first prompt\"}}]}}}}\n{{\"type\":\"ai-title\",\"aiTitle\":\"Older title\"}}\n{{\"type\":\"ai-title\",\"aiTitle\":\"Newest title\"}}\n",
            serde_json::to_string(&cwd.to_string_lossy()).expect("cwd json")
        ),
    )
    .expect("write claude");
}

fn write_codex_fixture(roots: &HistoryRoots, cwd: &std::path::Path) {
    let day = roots.codex.join("2026/07/22");
    fs::create_dir_all(&day).expect("codex day");
    let transcript = day.join("rollout-2026-07-22-thread-id.jsonl");
    fs::write(
        transcript,
        format!(
            "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"thread-id\",\"cwd\":{}}}}}\n{{\"type\":\"event_msg\",\"payload\":{{\"type\":\"user_message\",\"message\":\"  Build   the thing\\ncarefully  \"}}}}\n",
            serde_json::to_string(&cwd.to_string_lossy()).expect("cwd json")
        ),
    )
    .expect("write codex");
}
