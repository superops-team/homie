use rusqlite::{Connection, OptionalExtension, params};
use std::path::{Path, PathBuf};
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

const SCHEMA_VERSION: i64 = 1;

#[derive(Clone, Debug)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
}

#[derive(Debug)]
pub struct Storage {
    database_path: PathBuf,
    connection: Connection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationReport {
    pub schema_version: i64,
    pub applied: Vec<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageHealth {
    pub database_path: PathBuf,
    pub schema_version: i64,
    pub foreign_keys: bool,
    pub journal_mode: String,
}

#[derive(Clone, Debug)]
pub struct CreateSession {
    pub workspace: PathBuf,
    pub title: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub workspace: String,
    pub agent_profile_id: String,
    pub runtime_id: String,
    pub llm_profile_id: String,
    pub permission_profile_id: String,
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("database schema version {found} is newer than supported {supported}")]
    SchemaTooNew { found: i64, supported: i64 },
    #[error("no enabled default agent profile is available")]
    DefaultAgentProfileUnavailable,
}

pub fn open_or_create(config: StorageConfig) -> Result<Storage, StorageError> {
    std::fs::create_dir_all(&config.data_dir)?;
    let database_path = config.data_dir.join("homie.sqlite");
    let connection = Connection::open(&database_path)?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    Ok(Storage {
        database_path,
        connection,
    })
}

impl Storage {
    #[must_use]
    pub fn database_path(&self) -> &Path {
        &self.database_path
    }

    #[must_use]
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn migrate(&self) -> Result<MigrationReport, StorageError> {
        self.connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at INTEGER NOT NULL
            );
            "#,
        )?;

        let current = self.schema_version()?;
        if current > SCHEMA_VERSION {
            return Err(StorageError::SchemaTooNew {
                found: current,
                supported: SCHEMA_VERSION,
            });
        }
        if current == SCHEMA_VERSION {
            return Ok(MigrationReport {
                schema_version: current,
                applied: Vec::new(),
            });
        }

        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute_batch(SCHEMA_V1)?;
        transaction.execute(
            "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, strftime('%s','now'))",
            params![SCHEMA_VERSION],
        )?;
        transaction.commit()?;

        Ok(MigrationReport {
            schema_version: SCHEMA_VERSION,
            applied: vec![SCHEMA_VERSION],
        })
    }

    pub fn health_check(&self) -> Result<StorageHealth, StorageError> {
        let schema_version = self.schema_version()?;
        let foreign_keys: i64 =
            self.connection
                .pragma_query_value(None, "foreign_keys", |row| row.get(0))?;
        let journal_mode: String =
            self.connection
                .pragma_query_value(None, "journal_mode", |row| row.get(0))?;
        Ok(StorageHealth {
            database_path: self.database_path.clone(),
            schema_version,
            foreign_keys: foreign_keys == 1,
            journal_mode: journal_mode.to_ascii_lowercase(),
        })
    }

    pub fn seed_defaults(&self) -> Result<(), StorageError> {
        let now = now_unix();
        let transaction = self.connection.unchecked_transaction()?;
        transaction.execute(
            "INSERT OR IGNORE INTO providers(id, kind, name, base_url, api_key_ref, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                "provider_local_placeholder",
                "openai_compatible",
                "Local Placeholder",
                "http://127.0.0.1:11434/v1",
                "secret:provider_local_placeholder",
                now,
                now
            ],
        )?;
        transaction.execute(
            "INSERT OR IGNORE INTO llm_profiles(id, provider_id, name, default_model, allowed_models_json, params_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                "llm_default",
                "provider_local_placeholder",
                "Default Local LLM",
                "gpt-4o-mini",
                "[\"gpt-4o-mini\"]",
                "{}",
                now,
                now
            ],
        )?;
        transaction.execute(
            "INSERT OR IGNORE INTO runtime_descriptors(id, kind, display_name, binary, argv_template_json, env_json, env_scrub_json, status_authority, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                "runtime_codex",
                "codex",
                "Codex",
                "codex",
                "[]",
                "{}",
                "[\"OPENAI_API_KEY\", \"ANTHROPIC_API_KEY\", \"AUTHORIZATION\"]",
                "screen",
                now,
                now
            ],
        )?;
        transaction.execute(
            "INSERT OR IGNORE INTO permission_profiles(id, name, filesystem_json, network_json, shell_json, approval_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                "perm_default",
                "Default Restricted",
                "{\"mode\":\"workspace\"}",
                "{\"mode\":\"ask\"}",
                "{\"mode\":\"ask\"}",
                "{\"mode\":\"on_request\"}",
                now,
                now
            ],
        )?;
        transaction.execute(
            "INSERT OR IGNORE INTO agent_profiles(id, name, runtime_id, llm_profile_id, permission_profile_id, workspace_scope_json, enabled, is_default, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, 1, ?7, ?8)",
            params![
                "agent_codex_default",
                "Default Codex",
                "runtime_codex",
                "llm_default",
                "perm_default",
                "{\"mode\":\"selected_workspace\"}",
                now,
                now
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn create_session(&self, request: CreateSession) -> Result<SessionSummary, StorageError> {
        let Some(profile) = self.default_profile()? else {
            return Err(StorageError::DefaultAgentProfileUnavailable);
        };
        let now = now_unix();
        let id = Uuid::now_v7().to_string();
        let title = request
            .title
            .unwrap_or_else(|| "Untitled Session".to_string());
        let workspace = request.workspace.display().to_string();
        let output_log_path = format!("runtime/output/{id}.log");
        self.connection.execute(
            "INSERT INTO sessions(
                id, agent_profile_id, runtime_id, llm_profile_id, permission_profile_id,
                effective_config_id, workspace, title, status, output_log_path,
                output_tail_offset, virtual_key_id, created_at, updated_at, last_seen_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, 'created', ?8, 0, NULL, ?9, ?10, NULL)",
            params![
                id,
                profile.agent_profile_id,
                profile.runtime_id,
                profile.llm_profile_id,
                profile.permission_profile_id,
                workspace,
                title,
                output_log_path,
                now,
                now
            ],
        )?;
        self.session_by_id(&id)
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionSummary>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, title, status, workspace, agent_profile_id, runtime_id, llm_profile_id, permission_profile_id
             FROM sessions
             ORDER BY created_at ASC, id ASC",
        )?;
        let sessions = statement
            .query_map([], read_session_summary)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(sessions)
    }

    fn schema_version(&self) -> Result<i64, StorageError> {
        let has_table: Option<i64> = self
            .connection
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        if has_table.is_none() {
            return Ok(0);
        }
        let version = self
            .connection
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0);
        Ok(version)
    }

    fn default_profile(&self) -> Result<Option<DefaultProfile>, StorageError> {
        self.connection
            .query_row(
                "SELECT id, runtime_id, llm_profile_id, permission_profile_id
                 FROM agent_profiles
                 WHERE enabled = 1 AND is_default = 1
                 LIMIT 1",
                [],
                |row| {
                    Ok(DefaultProfile {
                        agent_profile_id: row.get(0)?,
                        runtime_id: row.get(1)?,
                        llm_profile_id: row.get(2)?,
                        permission_profile_id: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(StorageError::from)
    }

    fn session_by_id(&self, id: &str) -> Result<SessionSummary, StorageError> {
        self.connection
            .query_row(
                "SELECT id, title, status, workspace, agent_profile_id, runtime_id, llm_profile_id, permission_profile_id
                 FROM sessions
                 WHERE id = ?1",
                params![id],
                read_session_summary,
            )
            .map_err(StorageError::from)
    }
}

#[derive(Clone, Debug)]
struct DefaultProfile {
    agent_profile_id: String,
    runtime_id: String,
    llm_profile_id: String,
    permission_profile_id: String,
}

fn read_session_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionSummary> {
    Ok(SessionSummary {
        id: row.get(0)?,
        title: row.get(1)?,
        status: row.get(2)?,
        workspace: row.get(3)?,
        agent_profile_id: row.get(4)?,
        runtime_id: row.get(5)?,
        llm_profile_id: row.get(6)?,
        permission_profile_id: row.get(7)?,
    })
}

fn now_unix() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

const SCHEMA_V1: &str = r#"
CREATE TABLE providers (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    api_key_ref TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE llm_profiles (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE RESTRICT,
    name TEXT NOT NULL,
    default_model TEXT NOT NULL,
    allowed_models_json TEXT NOT NULL,
    params_json TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE model_pricing (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    model TEXT NOT NULL,
    input_price_per_million TEXT NOT NULL,
    output_price_per_million TEXT NOT NULL,
    cached_input_price_per_million TEXT,
    currency TEXT NOT NULL,
    effective_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE(provider_id, model, effective_at)
);

CREATE TABLE pricing_snapshots (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE RESTRICT,
    model TEXT NOT NULL,
    input_price_per_million TEXT NOT NULL,
    output_price_per_million TEXT NOT NULL,
    cached_input_price_per_million TEXT,
    currency TEXT NOT NULL,
    source_pricing_id TEXT REFERENCES model_pricing(id) ON DELETE SET NULL,
    captured_at INTEGER NOT NULL
);

CREATE TABLE runtime_descriptors (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    display_name TEXT NOT NULL,
    binary TEXT,
    argv_template_json TEXT NOT NULL,
    env_json TEXT NOT NULL,
    env_scrub_json TEXT NOT NULL,
    status_authority TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE permission_profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    filesystem_json TEXT NOT NULL,
    network_json TEXT NOT NULL,
    shell_json TEXT NOT NULL,
    approval_json TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE agent_profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    runtime_id TEXT NOT NULL REFERENCES runtime_descriptors(id) ON DELETE RESTRICT,
    llm_profile_id TEXT NOT NULL REFERENCES llm_profiles(id) ON DELETE RESTRICT,
    permission_profile_id TEXT NOT NULL REFERENCES permission_profiles(id) ON DELETE RESTRICT,
    workspace_scope_json TEXT NOT NULL,
    enabled INTEGER NOT NULL CHECK(enabled IN (0, 1)),
    is_default INTEGER NOT NULL CHECK(is_default IN (0, 1)),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE UNIQUE INDEX one_enabled_default_agent_profile
ON agent_profiles(is_default)
WHERE enabled = 1 AND is_default = 1;

CREATE TABLE skills (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    source_json TEXT NOT NULL,
    enabled_by_default INTEGER NOT NULL CHECK(enabled_by_default IN (0, 1)),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE agent_profile_skills (
    agent_profile_id TEXT NOT NULL REFERENCES agent_profiles(id) ON DELETE CASCADE,
    skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    enabled INTEGER NOT NULL CHECK(enabled IN (0, 1)),
    PRIMARY KEY(agent_profile_id, skill_id)
);

CREATE TABLE mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    transport TEXT NOT NULL,
    command TEXT,
    url TEXT,
    env_refs_json TEXT NOT NULL,
    enabled INTEGER NOT NULL CHECK(enabled IN (0, 1)),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE agent_profile_mcp_servers (
    agent_profile_id TEXT NOT NULL REFERENCES agent_profiles(id) ON DELETE CASCADE,
    mcp_server_id TEXT NOT NULL REFERENCES mcp_servers(id) ON DELETE CASCADE,
    enabled INTEGER NOT NULL CHECK(enabled IN (0, 1)),
    PRIMARY KEY(agent_profile_id, mcp_server_id)
);

CREATE TABLE effective_agent_configs (
    id TEXT PRIMARY KEY,
    session_id TEXT,
    agent_profile_id TEXT NOT NULL REFERENCES agent_profiles(id) ON DELETE RESTRICT,
    runtime_id TEXT NOT NULL REFERENCES runtime_descriptors(id) ON DELETE RESTRICT,
    llm_profile_id TEXT NOT NULL REFERENCES llm_profiles(id) ON DELETE RESTRICT,
    provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE RESTRICT,
    permission_profile_id TEXT NOT NULL REFERENCES permission_profiles(id) ON DELETE RESTRICT,
    virtual_key_id TEXT,
    skill_ids_json TEXT NOT NULL,
    mcp_server_ids_json TEXT NOT NULL,
    workspace_scope_json TEXT NOT NULL,
    frozen_at INTEGER NOT NULL
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    agent_profile_id TEXT NOT NULL REFERENCES agent_profiles(id) ON DELETE RESTRICT,
    runtime_id TEXT NOT NULL REFERENCES runtime_descriptors(id) ON DELETE RESTRICT,
    llm_profile_id TEXT NOT NULL REFERENCES llm_profiles(id) ON DELETE RESTRICT,
    permission_profile_id TEXT NOT NULL REFERENCES permission_profiles(id) ON DELETE RESTRICT,
    effective_config_id TEXT REFERENCES effective_agent_configs(id) ON DELETE SET NULL,
    workspace TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    output_log_path TEXT NOT NULL,
    output_tail_offset INTEGER NOT NULL DEFAULT 0,
    virtual_key_id TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    last_seen_at INTEGER
);

CREATE TABLE context_events (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    safe_payload_json TEXT NOT NULL,
    output_offset INTEGER,
    created_at INTEGER NOT NULL
);

CREATE TABLE virtual_keys (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id) ON DELETE CASCADE,
    agent_profile_id TEXT NOT NULL REFERENCES agent_profiles(id) ON DELETE RESTRICT,
    provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE RESTRICT,
    key_hash TEXT NOT NULL,
    allowed_models_json TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    revoked_at INTEGER,
    created_at INTEGER NOT NULL
);

CREATE TABLE usage_records (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL,
    session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    agent_profile_id TEXT NOT NULL REFERENCES agent_profiles(id) ON DELETE RESTRICT,
    runtime_id TEXT NOT NULL REFERENCES runtime_descriptors(id) ON DELETE RESTRICT,
    provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE RESTRICT,
    llm_profile_id TEXT NOT NULL REFERENCES llm_profiles(id) ON DELETE RESTRICT,
    model TEXT NOT NULL,
    request_kind TEXT NOT NULL,
    status TEXT NOT NULL,
    input_tokens INTEGER,
    output_tokens INTEGER,
    cached_input_tokens INTEGER,
    cache_read_tokens INTEGER,
    cache_write_tokens INTEGER,
    cache_hit_rate TEXT,
    reasoning_tokens INTEGER,
    total_tokens INTEGER,
    unit_price_input TEXT,
    unit_price_output TEXT,
    currency TEXT,
    pricing_snapshot_id TEXT REFERENCES pricing_snapshots(id) ON DELETE SET NULL,
    estimated_cost TEXT,
    first_token_latency_ms INTEGER,
    total_latency_ms INTEGER,
    started_at INTEGER NOT NULL,
    completed_at INTEGER NOT NULL,
    safe_error_code TEXT
);

CREATE TABLE tool_call_metrics (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    agent_profile_id TEXT NOT NULL REFERENCES agent_profiles(id) ON DELETE RESTRICT,
    runtime_id TEXT NOT NULL REFERENCES runtime_descriptors(id) ON DELETE RESTRICT,
    tool_name TEXT NOT NULL,
    mcp_server_id TEXT REFERENCES mcp_servers(id) ON DELETE SET NULL,
    status TEXT NOT NULL,
    latency_ms INTEGER NOT NULL,
    queue_latency_ms INTEGER,
    input_bytes INTEGER,
    output_bytes INTEGER,
    started_at INTEGER NOT NULL,
    completed_at INTEGER NOT NULL,
    safe_error_code TEXT
);

CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    agent_profile_id TEXT REFERENCES agent_profiles(id) ON DELETE SET NULL,
    session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    metadata_json TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE config_events (
    id TEXT PRIMARY KEY,
    subject_type TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    event_kind TEXT NOT NULL,
    safe_payload_json TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE metrics_write_failures (
    id TEXT PRIMARY KEY,
    metric_kind TEXT NOT NULL,
    subject_id TEXT,
    safe_error_code TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
"#;
