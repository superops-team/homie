use homie_storage::{StorageConfig, open_or_create};

#[test]
fn reference_parity_tables_exist_after_migration() {
    let temp = tempfile::tempdir().expect("tempdir");
    let storage = open_or_create(StorageConfig {
        data_dir: temp.path().to_path_buf(),
    })
    .expect("open storage");
    storage.migrate().expect("migrate");

    let conn = storage.connection();
    for table in [
        "preferences",
        "projects",
        "worktrees",
        "session_artifacts",
        "listening_ports",
        "pull_request_statuses",
        "history_entries",
        "hosts",
        "node_accounts",
        "handoff_records",
        "memory_candidates",
    ] {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |row| row.get(0),
            )
            .expect("table query");
        assert_eq!(exists, 1, "missing table {table}");
    }
}

#[test]
fn session_artifacts_are_unique_per_session_kind_and_url() {
    let temp = tempfile::tempdir().expect("tempdir");
    let storage = open_or_create(StorageConfig {
        data_dir: temp.path().to_path_buf(),
    })
    .expect("open storage");
    storage.migrate().expect("migrate");
    storage.seed_defaults().expect("seed defaults");
    let session = storage
        .create_session(homie_storage::CreateSession {
            workspace: temp.path().to_path_buf(),
            title: Some("artifact".to_string()),
        })
        .expect("create session");

    storage
        .connection()
        .execute(
            "INSERT INTO session_artifacts(id, session_id, kind, url, label, metadata_json, created_at, updated_at)
             VALUES ('artifact_1', ?1, 'pull_request', 'https://example.invalid/pr/1', 'PR', '{}', 1, 1)",
            [&session.id],
        )
        .expect("insert artifact");
    let duplicate = storage.connection().execute(
        "INSERT INTO session_artifacts(id, session_id, kind, url, label, metadata_json, created_at, updated_at)
         VALUES ('artifact_2', ?1, 'pull_request', 'https://example.invalid/pr/1', 'PR', '{}', 1, 1)",
        [&session.id],
    );
    assert!(duplicate.is_err(), "duplicate artifact should fail");
}
