use homie_storage::{CreateSession, StorageConfig, StorageError, open_or_create};

#[test]
fn seed_defaults_is_idempotent_and_creates_codex_profile() {
    let temp = tempfile::tempdir().expect("tempdir");
    let storage = open_or_create(StorageConfig {
        data_dir: temp.path().to_path_buf(),
    })
    .expect("open storage");
    storage.migrate().expect("migrate");

    storage.seed_defaults().expect("seed defaults");
    storage.seed_defaults().expect("seed defaults again");

    let conn = storage.connection();
    let default_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM agent_profiles WHERE enabled = 1 AND is_default = 1",
            [],
            |row| row.get(0),
        )
        .expect("default count");
    assert_eq!(default_count, 1);

    let runtime_id: String = conn
        .query_row(
            "SELECT runtime_id FROM agent_profiles WHERE id = 'agent_codex_default'",
            [],
            |row| row.get(0),
        )
        .expect("runtime id");
    assert_eq!(runtime_id, "runtime_codex");
}

#[test]
fn create_and_list_session_uses_default_codex_profile() {
    let temp = tempfile::tempdir().expect("tempdir");
    let storage = open_or_create(StorageConfig {
        data_dir: temp.path().to_path_buf(),
    })
    .expect("open storage");
    storage.migrate().expect("migrate");
    storage.seed_defaults().expect("seed defaults");

    let session = storage
        .create_session(CreateSession {
            workspace: temp.path().to_path_buf(),
            title: Some("Smoke".to_string()),
        })
        .expect("create session");

    assert_eq!(session.title, "Smoke");
    assert_eq!(session.status, "created");
    assert_eq!(session.runtime_id, "runtime_codex");
    assert_eq!(session.agent_profile_id, "agent_codex_default");

    let sessions = storage.list_sessions().expect("list sessions");
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, session.id);
}

#[test]
fn create_session_fails_without_enabled_default_profile() {
    let temp = tempfile::tempdir().expect("tempdir");
    let storage = open_or_create(StorageConfig {
        data_dir: temp.path().to_path_buf(),
    })
    .expect("open storage");
    storage.migrate().expect("migrate");

    let error = storage
        .create_session(CreateSession {
            workspace: temp.path().to_path_buf(),
            title: None,
        })
        .expect_err("missing default profile");

    assert!(matches!(
        error,
        StorageError::DefaultAgentProfileUnavailable
    ));
}

#[test]
fn create_session_with_parent_is_one_atomic_insert() {
    let temp = tempfile::tempdir().expect("tempdir");
    let storage = open_or_create(StorageConfig {
        data_dir: temp.path().to_path_buf(),
    })
    .expect("open storage");
    storage.migrate().expect("migrate");
    storage.seed_defaults().expect("seed defaults");
    let parent = storage
        .create_session(CreateSession {
            workspace: temp.path().to_path_buf(),
            title: Some("Parent".to_string()),
        })
        .expect("create parent");

    let child = storage
        .create_session_with_parent(
            CreateSession {
                workspace: temp.path().to_path_buf(),
                title: Some("Child".to_string()),
            },
            Some(&parent.id),
        )
        .expect("create child with parent");

    assert_eq!(
        storage
            .session_core_metadata(&child.id)
            .expect("child metadata")
            .parent_session_id
            .as_deref(),
        Some(parent.id.as_str())
    );
    storage
        .create_session_with_parent(
            CreateSession {
                workspace: temp.path().to_path_buf(),
                title: Some("Invalid child".to_string()),
            },
            Some("missing-parent"),
        )
        .expect_err("foreign key must reject missing parent");
    assert_eq!(
        storage.list_sessions().expect("list sessions").len(),
        2,
        "failed atomic insert must not leave an orphan session"
    );
}
