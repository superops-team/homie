use homie_storage::{StorageConfig, UsageScanFileQuery, UsageScanFileState, open_or_create};

#[test]
fn upserts_and_reads_usage_scan_file_state() {
    let temp = tempfile::tempdir().expect("tempdir");
    let storage = storage(temp.path());
    let path = temp.path().join("claude.jsonl").display().to_string();

    storage
        .upsert_usage_scan_file(UsageScanFileState {
            path: path.clone(),
            provider: "claude".to_string(),
            profile_id: Some("work".to_string()),
            size: 100,
            offset: 80,
            modified_ns: 1_000,
            device: Some(10),
            inode: Some(20),
            tail_hash: 111,
            model: Some("claude-sonnet".to_string()),
            scanned_at: 200,
        })
        .expect("initial upsert");

    let initial = storage
        .usage_scan_file(&path)
        .expect("read")
        .expect("state");
    assert_eq!(initial.offset, 80);
    assert_eq!(initial.model.as_deref(), Some("claude-sonnet"));

    storage
        .upsert_usage_scan_file(UsageScanFileState {
            path: path.clone(),
            provider: "claude".to_string(),
            profile_id: Some("work".to_string()),
            size: 140,
            offset: 140,
            modified_ns: 2_000,
            device: Some(10),
            inode: Some(20),
            tail_hash: 222,
            model: Some("claude-opus".to_string()),
            scanned_at: 300,
        })
        .expect("overwrite");

    let updated = storage
        .usage_scan_file(&path)
        .expect("read updated")
        .expect("state");
    assert_eq!(updated.size, 140);
    assert_eq!(updated.offset, 140);
    assert_eq!(updated.modified_ns, 2_000);
    assert_eq!(updated.tail_hash, 222);
    assert_eq!(updated.model.as_deref(), Some("claude-opus"));
    assert_eq!(updated.scanned_at, 300);
}

#[test]
fn lists_usage_scan_files_by_provider_and_profile() {
    let temp = tempfile::tempdir().expect("tempdir");
    let storage = storage(temp.path());
    for state in [
        state("a.jsonl", "claude", Some("work")),
        state("b.jsonl", "claude", Some("personal")),
        state("c.jsonl", "codex", None),
    ] {
        storage.upsert_usage_scan_file(state).expect("upsert");
    }

    let claude = storage
        .list_usage_scan_files(UsageScanFileQuery {
            provider: Some("claude".to_string()),
            profile_id: None,
        })
        .expect("list claude");
    assert_eq!(claude.len(), 2);

    let work = storage
        .list_usage_scan_files(UsageScanFileQuery {
            provider: Some("claude".to_string()),
            profile_id: Some("work".to_string()),
        })
        .expect("list work");
    assert_eq!(work.len(), 1);
    assert_eq!(work[0].profile_id.as_deref(), Some("work"));

    let codex = storage
        .list_usage_scan_files(UsageScanFileQuery {
            provider: Some("codex".to_string()),
            profile_id: None,
        })
        .expect("list codex");
    assert_eq!(codex.len(), 1);
    assert_eq!(codex[0].profile_id, None);
}

fn storage(data_dir: &std::path::Path) -> homie_storage::Storage {
    let storage = open_or_create(StorageConfig {
        data_dir: data_dir.to_path_buf(),
    })
    .expect("storage");
    storage.migrate().expect("migrate");
    storage
}

fn state(path: &str, provider: &str, profile_id: Option<&str>) -> UsageScanFileState {
    UsageScanFileState {
        path: path.to_string(),
        provider: provider.to_string(),
        profile_id: profile_id.map(str::to_string),
        size: 10,
        offset: 10,
        modified_ns: 20,
        device: None,
        inode: None,
        tail_hash: 30,
        model: None,
        scanned_at: 40,
    }
}
