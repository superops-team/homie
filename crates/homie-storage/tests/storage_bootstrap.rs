use homie_storage::{StorageConfig, open_or_create};

#[test]
fn doctor_storage_creates_sqlite_and_reports_wal_foreign_keys() {
    let temp = tempfile::tempdir().expect("tempdir");
    let storage = open_or_create(StorageConfig {
        data_dir: temp.path().to_path_buf(),
    })
    .expect("open storage");

    let report = storage.migrate().expect("migrate");
    assert_eq!(report.schema_version, 1);

    let health = storage.health_check().expect("health");
    assert_eq!(health.schema_version, 1);
    assert!(health.foreign_keys);
    assert_eq!(health.journal_mode, "wal");
    assert_eq!(health.database_path, temp.path().join("homie.sqlite"));
    assert!(health.database_path.exists());
}

#[test]
fn migration_is_idempotent() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config = StorageConfig {
        data_dir: temp.path().to_path_buf(),
    };

    let storage = open_or_create(config.clone()).expect("open storage");
    let first = storage.migrate().expect("first migrate");
    let second = storage.migrate().expect("second migrate");

    assert_eq!(first.schema_version, 1);
    assert_eq!(second.schema_version, 1);
    assert!(second.applied.is_empty());
}

#[test]
fn sqlite_constraints() {
    let temp = tempfile::tempdir().expect("tempdir");
    let storage = open_or_create(StorageConfig {
        data_dir: temp.path().to_path_buf(),
    })
    .expect("open storage");
    storage.migrate().expect("migrate");

    let conn = storage.connection();
    conn.execute_batch(
        r#"
        INSERT INTO providers(id, kind, name, base_url, api_key_ref, created_at, updated_at)
        VALUES ('provider_1', 'openai_compatible', 'Local', 'http://127.0.0.1:1', 'secret_ref', 1, 1);
        INSERT INTO llm_profiles(id, provider_id, name, default_model, allowed_models_json, params_json, created_at, updated_at)
        VALUES ('llm_1', 'provider_1', 'Default', 'model-a', '["model-a"]', '{}', 1, 1);
        INSERT INTO runtime_descriptors(id, kind, display_name, binary, argv_template_json, env_json, env_scrub_json, status_authority, created_at, updated_at)
        VALUES ('runtime_codex', 'codex', 'Codex', 'codex', '[]', '{}', '[]', 'screen', 1, 1);
        INSERT INTO permission_profiles(id, name, filesystem_json, network_json, shell_json, approval_json, created_at, updated_at)
        VALUES ('perm_1', 'Default', '{}', '{}', '{}', '{}', 1, 1);
        INSERT INTO agent_profiles(id, name, runtime_id, llm_profile_id, permission_profile_id, workspace_scope_json, enabled, is_default, created_at, updated_at)
        VALUES ('agent_1', 'Default Codex', 'runtime_codex', 'llm_1', 'perm_1', '{}', 1, 1, 1, 1);
        INSERT INTO skills(id, name, source_json, enabled_by_default, created_at, updated_at)
        VALUES ('skill_1', 'Skill', '{}', 1, 1, 1);
        INSERT INTO mcp_servers(id, name, transport, command, url, env_refs_json, enabled, created_at, updated_at)
        VALUES ('mcp_1', 'MCP', 'stdio', 'mcp', NULL, '{}', 1, 1, 1);
        INSERT INTO model_pricing(id, provider_id, model, input_price_per_million, output_price_per_million, cached_input_price_per_million, currency, effective_at, created_at)
        VALUES ('pricing_1', 'provider_1', 'model-a', '1.0', '2.0', '0.1', 'USD', 1, 1);
        "#,
    )
    .expect("seed");

    conn.execute(
        "INSERT INTO agent_profile_skills(agent_profile_id, skill_id, enabled) VALUES (?1, ?2, ?3)",
        ("agent_1", "skill_1", 1),
    )
    .expect("first skill binding");
    assert!(
        conn.execute(
            "INSERT INTO agent_profile_skills(agent_profile_id, skill_id, enabled) VALUES (?1, ?2, ?3)",
            ("agent_1", "skill_1", 1),
        )
        .is_err()
    );

    conn.execute(
        "INSERT INTO agent_profile_mcp_servers(agent_profile_id, mcp_server_id, enabled) VALUES (?1, ?2, ?3)",
        ("agent_1", "mcp_1", 1),
    )
    .expect("first mcp binding");
    assert!(
        conn.execute(
            "INSERT INTO agent_profile_mcp_servers(agent_profile_id, mcp_server_id, enabled) VALUES (?1, ?2, ?3)",
            ("agent_1", "mcp_1", 1),
        )
        .is_err()
    );

    assert!(
        conn.execute(
            "INSERT INTO model_pricing(id, provider_id, model, input_price_per_million, output_price_per_million, cached_input_price_per_million, currency, effective_at, created_at)
             VALUES ('pricing_2', 'provider_1', 'model-a', '1.0', '2.0', '0.1', 'USD', 1, 1)",
            [],
        )
        .is_err()
    );

    assert!(
        conn.execute(
            "INSERT INTO agent_profiles(id, name, runtime_id, llm_profile_id, permission_profile_id, workspace_scope_json, enabled, is_default, created_at, updated_at)
             VALUES ('agent_2', 'Other', 'runtime_codex', 'llm_1', 'perm_1', '{}', 1, 1, 1, 1)",
            [],
        )
        .is_err()
    );

    assert!(
        conn.execute(
            "INSERT INTO agent_profiles(id, name, runtime_id, llm_profile_id, permission_profile_id, workspace_scope_json, enabled, is_default, created_at, updated_at)
             VALUES ('agent_bad', 'Bad', 'missing_runtime', 'llm_1', 'perm_1', '{}', 1, 0, 1, 1)",
            [],
        )
        .is_err()
    );
}

#[test]
fn usage_metrics_schema() {
    let temp = tempfile::tempdir().expect("tempdir");
    let storage = open_or_create(StorageConfig {
        data_dir: temp.path().to_path_buf(),
    })
    .expect("open storage");
    storage.migrate().expect("migrate");
    let conn = storage.connection();
    conn.execute_batch(
        r#"
        INSERT INTO providers(id, kind, name, base_url, api_key_ref, created_at, updated_at)
        VALUES ('provider_1', 'openai_compatible', 'Local', 'http://127.0.0.1:1', 'secret_ref', 1, 1);
        INSERT INTO llm_profiles(id, provider_id, name, default_model, allowed_models_json, params_json, created_at, updated_at)
        VALUES ('llm_1', 'provider_1', 'Default', 'model-a', '["model-a"]', '{}', 1, 1);
        INSERT INTO runtime_descriptors(id, kind, display_name, binary, argv_template_json, env_json, env_scrub_json, status_authority, created_at, updated_at)
        VALUES ('runtime_codex', 'codex', 'Codex', 'codex', '[]', '{}', '[]', 'screen', 1, 1);
        INSERT INTO permission_profiles(id, name, filesystem_json, network_json, shell_json, approval_json, created_at, updated_at)
        VALUES ('perm_1', 'Default', '{}', '{}', '{}', '{}', 1, 1);
        INSERT INTO agent_profiles(id, name, runtime_id, llm_profile_id, permission_profile_id, workspace_scope_json, enabled, is_default, created_at, updated_at)
        VALUES ('agent_1', 'Default Codex', 'runtime_codex', 'llm_1', 'perm_1', '{}', 1, 1, 1, 1);
        INSERT INTO pricing_snapshots(id, provider_id, model, input_price_per_million, output_price_per_million, cached_input_price_per_million, currency, source_pricing_id, captured_at)
        VALUES ('snap_1', 'provider_1', 'model-a', '1.0', '2.0', '0.1', 'USD', NULL, 1);
        INSERT INTO virtual_keys(id, session_id, agent_profile_id, provider_id, key_hash, allowed_models_json, expires_at, revoked_at, created_at)
        VALUES ('vk_1', NULL, 'agent_1', 'provider_1', 'hash', '["model-a"]', 999, NULL, 1);
        INSERT INTO sessions(id, agent_profile_id, runtime_id, llm_profile_id, permission_profile_id, effective_config_id, workspace, title, status, output_log_path, output_tail_offset, virtual_key_id, created_at, updated_at, last_seen_at)
        VALUES ('session_1', 'agent_1', 'runtime_codex', 'llm_1', 'perm_1', NULL, '/tmp', 'Session', 'idle', 'runtime/output/session_1.log', 0, 'vk_1', 1, 1, NULL);
        "#,
    )
    .expect("seed");

    conn.execute(
        r#"
        INSERT INTO usage_records(
            id, request_id, session_id, agent_profile_id, runtime_id, provider_id, llm_profile_id, model, request_kind, status,
            input_tokens, output_tokens, cached_input_tokens, cache_read_tokens, cache_write_tokens, cache_hit_rate,
            reasoning_tokens, total_tokens, unit_price_input, unit_price_output, currency, pricing_snapshot_id,
            estimated_cost, first_token_latency_ms, total_latency_ms, started_at, completed_at, safe_error_code
        )
        VALUES (
            'usage_1', 'req_1', 'session_1', 'agent_1', 'runtime_codex', 'provider_1', 'llm_1', 'model-a', 'chat', 'ok',
            100, 20, 40, 40, 10, '0.4',
            5, 125, '1.0', '2.0', 'USD', 'snap_1',
            '0.00014', 50, 300, 1, 2, NULL
        )
        "#,
        [],
    )
    .expect("usage insert");

    conn.execute(
        r#"
        INSERT INTO tool_call_metrics(
            id, session_id, agent_profile_id, runtime_id, tool_name, mcp_server_id, status,
            latency_ms, queue_latency_ms, input_bytes, output_bytes, started_at, completed_at, safe_error_code
        )
        VALUES ('tool_1', 'session_1', 'agent_1', 'runtime_codex', 'apply_patch', NULL, 'ok', 42, 3, 100, 200, 1, 2, NULL)
        "#,
        [],
    )
    .expect("tool metric insert");
}
