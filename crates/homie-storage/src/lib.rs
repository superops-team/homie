use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

const SCHEMA_VERSION: i64 = 3;

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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPreferences {
    pub startup_behavior: String,
    pub terminal_font_size: u8,
    pub hibernate_idle_minutes: u16,
    pub remote_companion_access: bool,
}

impl Default for SettingsPreferences {
    fn default() -> Self {
        Self {
            startup_behavior: "restore_last_session".to_string(),
            terminal_font_size: 13,
            hibernate_idle_minutes: 45,
            remote_companion_access: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaInventory {
    pub tables: Vec<TableSchema>,
}

impl SchemaInventory {
    #[must_use]
    pub fn has_table(&self, table: &str) -> bool {
        self.tables.iter().any(|schema| schema.name == table)
    }

    #[must_use]
    pub fn table(&self, table: &str) -> Option<&TableSchema> {
        self.tables.iter().find(|schema| schema.name == table)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<String>,
    pub unique_indexes: Vec<Vec<String>>,
}

impl TableSchema {
    #[must_use]
    pub fn has_column(&self, column: &str) -> bool {
        self.columns.iter().any(|candidate| candidate == column)
    }

    #[must_use]
    pub fn has_unique_index(&self, columns: &[&str]) -> bool {
        self.unique_indexes.iter().any(|candidate| {
            candidate.len() == columns.len()
                && candidate
                    .iter()
                    .map(String::as_str)
                    .eq(columns.iter().copied())
        })
    }
}

#[derive(Clone, Debug)]
pub struct HistoryEntryUpsert {
    pub agent_kind: String,
    pub external_id: String,
    pub cwd: PathBuf,
    pub title: Option<String>,
    pub title_source: String,
    pub transcript_path: PathBuf,
    pub last_active_at: i64,
    pub created_at: Option<i64>,
    pub cwd_exists: bool,
    pub metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryEntrySummary {
    pub id: String,
    pub agent_kind: String,
    pub external_id: String,
    pub cwd: String,
    pub title: Option<String>,
    pub title_source: String,
    pub transcript_path: String,
    pub last_active_at: i64,
    pub created_at: Option<i64>,
    pub cwd_exists: bool,
    pub tracked_session_id: Option<String>,
    pub metadata: Value,
}

#[derive(Clone, Debug)]
pub struct ProjectUpsert {
    pub root_path: PathBuf,
    pub display_name: Option<String>,
    pub remote_origin: Option<String>,
    pub pinned_order: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectSummary {
    pub id: String,
    pub root_path: String,
    pub display_name: Option<String>,
    pub remote_origin: Option<String>,
    pub pinned_order: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct WorktreeUpsert {
    pub project_id: String,
    pub session_id: Option<String>,
    pub path: PathBuf,
    pub branch: Option<String>,
    pub head_sha: Option<String>,
    pub is_bare: bool,
    pub is_detached: bool,
    pub is_prunable: bool,
    pub dirty: bool,
    pub merged: bool,
    pub stale_suggestion: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeSummary {
    pub id: String,
    pub project_id: String,
    pub session_id: Option<String>,
    pub path: String,
    pub branch: Option<String>,
    pub head_sha: Option<String>,
    pub is_bare: bool,
    pub is_detached: bool,
    pub is_prunable: bool,
    pub dirty: bool,
    pub merged: bool,
    pub stale_suggestion: bool,
}

#[derive(Clone, Debug)]
pub struct SessionCoreMetadataUpdate {
    pub project_id: Option<String>,
    pub worktree_path: Option<PathBuf>,
    pub git_branch: Option<String>,
    pub title_source: String,
    pub agent_session_id: Option<String>,
    pub transcript_path: Option<PathBuf>,
    pub needs_input_kind: Option<String>,
    pub needs_input_payload: Value,
    pub resumability: String,
    pub parent_session_id: Option<String>,
    pub pinned: bool,
    pub archived_at: Option<i64>,
    pub remote_active: bool,
    pub host_id: Option<String>,
    pub foreground_agent_kind: Option<String>,
    pub memory_bytes: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionCoreMetadata {
    pub id: String,
    pub project_id: Option<String>,
    pub worktree_path: Option<String>,
    pub git_branch: Option<String>,
    pub title_source: String,
    pub agent_session_id: Option<String>,
    pub transcript_path: Option<String>,
    pub needs_input_kind: Option<String>,
    pub needs_input_payload: Value,
    pub resumability: String,
    pub parent_session_id: Option<String>,
    pub pinned: bool,
    pub archived_at: Option<i64>,
    pub remote_active: bool,
    pub host_id: Option<String>,
    pub foreground_agent_kind: Option<String>,
    pub memory_bytes: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct RecordUsage {
    pub request_id: String,
    pub session_id: Option<String>,
    pub agent_profile_id: String,
    pub runtime_id: String,
    pub provider_id: String,
    pub llm_profile_id: String,
    pub model: String,
    pub request_kind: String,
    pub status: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cached_input_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub cache_write_5m_tokens: i64,
    pub cache_write_1h_tokens: i64,
    pub reasoning_tokens: i64,
    pub unit_price_input: Option<String>,
    pub unit_price_output: Option<String>,
    pub currency: Option<String>,
    pub pricing_snapshot_id: Option<String>,
    pub estimated_cost: Option<String>,
    pub billed_cost: Option<String>,
    pub first_token_latency_ms: Option<i64>,
    pub total_latency_ms: Option<i64>,
    pub started_at: i64,
    pub completed_at: i64,
    pub safe_error_code: Option<String>,
    pub value_kind: String,
    pub source: String,
    pub source_event_id: String,
}

#[derive(Clone, Debug, Default)]
pub struct UsageQuery {
    pub session_id: Option<String>,
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub from: Option<i64>,
    pub to: Option<i64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UsageTotals {
    pub events: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cached_input_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub cache_write_5m_tokens: i64,
    pub cache_write_1h_tokens: i64,
    pub reasoning_tokens: i64,
    pub total_tokens: i64,
    pub estimated_cost: f64,
    pub billed_cost: f64,
    pub authoritative_billing_available: bool,
}

#[derive(Clone, Debug)]
pub struct UsageImportDefaults {
    pub agent_profile_id: String,
    pub runtime_id: String,
    pub provider_id: String,
    pub llm_profile_id: String,
    pub request_kind: String,
    pub status: String,
}

impl UsageImportDefaults {
    #[must_use]
    pub fn from_session(session: &SessionSummary) -> Self {
        Self {
            agent_profile_id: session.agent_profile_id.clone(),
            runtime_id: session.runtime_id.clone(),
            provider_id: "provider_local_placeholder".to_string(),
            llm_profile_id: session.llm_profile_id.clone(),
            request_kind: "chat".to_string(),
            status: "ok".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UsageImportResult {
    pub inserted: usize,
    pub skipped: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageScanFileState {
    pub path: String,
    pub provider: String,
    pub profile_id: Option<String>,
    pub size: i64,
    pub offset: i64,
    pub modified_ns: i64,
    pub device: Option<i64>,
    pub inode: Option<i64>,
    pub tail_hash: i64,
    pub model: Option<String>,
    pub scanned_at: i64,
}

#[derive(Clone, Debug, Default)]
pub struct UsageScanFileQuery {
    pub provider: Option<String>,
    pub profile_id: Option<String>,
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
    #[error("preference JSON error: {0}")]
    PreferenceJson(#[from] serde_json::Error),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("{entity} not found: {id}")]
    NotFound { entity: &'static str, id: String },
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
        let mut applied = Vec::new();
        if current < 1 {
            transaction.execute_batch(SCHEMA_V1)?;
            transaction.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES (1, strftime('%s','now'))",
                [],
            )?;
            applied.push(1);
        }
        if current < 2 {
            transaction.execute_batch(SCHEMA_V2)?;
            transaction.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES (2, strftime('%s','now'))",
                [],
            )?;
            applied.push(2);
        }
        if current < 3 {
            transaction.execute_batch(SCHEMA_V3)?;
            transaction.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES (3, strftime('%s','now'))",
                [],
            )?;
            applied.push(3);
        }
        transaction.commit()?;

        Ok(MigrationReport {
            schema_version: SCHEMA_VERSION,
            applied,
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
        self.create_session_with_parent(request, None)
    }

    pub fn create_session_with_parent(
        &self,
        request: CreateSession,
        parent_session_id: Option<&str>,
    ) -> Result<SessionSummary, StorageError> {
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
                output_tail_offset, virtual_key_id, created_at, updated_at, last_seen_at,
                parent_session_id
             )
             VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, 'created', ?8, 0, NULL, ?9, ?10, NULL, ?11)",
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
                now,
                parent_session_id
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

    pub fn update_session_status(
        &self,
        id: &str,
        status: &str,
    ) -> Result<SessionSummary, StorageError> {
        self.connection.execute(
            "UPDATE sessions SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, now_unix(), id],
        )?;
        self.session_by_id(id)
    }

    pub fn delete_session(&self, id: &str) -> Result<bool, StorageError> {
        Ok(self
            .connection
            .execute("DELETE FROM sessions WHERE id = ?1", params![id])?
            != 0)
    }

    pub fn set_session_needs_input(
        &self,
        id: &str,
        kind: &str,
        payload: &Value,
    ) -> Result<(), StorageError> {
        let changed = self.connection.execute(
            "UPDATE sessions
             SET status = 'needs_input',
                 needs_input_kind = ?1,
                 needs_input_payload_json = ?2,
                 updated_at = ?3
             WHERE id = ?4",
            params![kind, serde_json::to_string(payload)?, now_unix(), id],
        )?;
        if changed == 0 {
            return Err(StorageError::NotFound {
                entity: "session",
                id: id.to_string(),
            });
        }
        Ok(())
    }

    pub fn set_session_parent(
        &self,
        id: &str,
        parent_session_id: &str,
    ) -> Result<(), StorageError> {
        let changed = self.connection.execute(
            "UPDATE sessions
             SET parent_session_id = ?1,
                 updated_at = ?2
             WHERE id = ?3",
            params![parent_session_id, now_unix(), id],
        )?;
        if changed == 0 {
            return Err(StorageError::NotFound {
                entity: "session",
                id: id.to_string(),
            });
        }
        Ok(())
    }

    pub fn list_child_sessions(
        &self,
        parent_session_id: &str,
    ) -> Result<Vec<SessionSummary>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, title, status, workspace, agent_profile_id, runtime_id, llm_profile_id, permission_profile_id
             FROM sessions
             WHERE parent_session_id = ?1
             ORDER BY created_at ASC, id ASC",
        )?;
        statement
            .query_map([parent_session_id], read_session_summary)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    pub fn schema_inventory(&self) -> Result<SchemaInventory, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT name FROM sqlite_master
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
             ORDER BY name",
        )?;
        let table_names = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        let mut tables = Vec::with_capacity(table_names.len());
        for name in table_names {
            tables.push(TableSchema {
                columns: self.table_columns(&name)?,
                unique_indexes: self.unique_indexes(&name)?,
                name,
            });
        }
        Ok(SchemaInventory { tables })
    }

    pub fn upsert_history_entry(
        &self,
        entry: HistoryEntryUpsert,
    ) -> Result<HistoryEntrySummary, StorageError> {
        let id = self
            .history_id(&entry.agent_kind, &entry.external_id)?
            .unwrap_or_else(|| Uuid::now_v7().to_string());
        let cwd = entry.cwd.display().to_string();
        let transcript_path = entry.transcript_path.display().to_string();
        let metadata_json = serde_json::to_string(&entry.metadata)?;
        self.connection.execute(
            "INSERT INTO history_entries(
                id, agent_kind, external_id, cwd, title, title_source, transcript_path,
                last_active_at, created_at, cwd_exists, tracked_session_id, metadata_json
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, NULL, ?11)
             ON CONFLICT(agent_kind, external_id) DO UPDATE SET
                cwd = excluded.cwd,
                title = excluded.title,
                title_source = excluded.title_source,
                transcript_path = excluded.transcript_path,
                last_active_at = excluded.last_active_at,
                created_at = excluded.created_at,
                cwd_exists = excluded.cwd_exists,
                metadata_json = excluded.metadata_json",
            params![
                id,
                entry.agent_kind,
                entry.external_id,
                cwd,
                entry.title,
                entry.title_source,
                transcript_path,
                entry.last_active_at,
                entry.created_at,
                bool_to_i64(entry.cwd_exists),
                metadata_json
            ],
        )?;
        self.history_by_agent_external(&entry.agent_kind, &entry.external_id)
    }

    pub fn mark_history_entry_tracked(
        &self,
        agent_kind: &str,
        external_id: &str,
        session_id: &str,
    ) -> Result<(), StorageError> {
        let changed = self.connection.execute(
            "UPDATE history_entries
             SET tracked_session_id = ?1
             WHERE agent_kind = ?2 AND external_id = ?3",
            params![session_id, agent_kind, external_id],
        )?;
        if changed == 0 {
            return Err(StorageError::NotFound {
                entity: "history_entry",
                id: format!("{agent_kind}:{external_id}"),
            });
        }
        Ok(())
    }

    pub fn list_history_entries(
        &self,
        limit: usize,
    ) -> Result<Vec<HistoryEntrySummary>, StorageError> {
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let mut statement = self.connection.prepare(
            "SELECT id, agent_kind, external_id, cwd, title, title_source, transcript_path,
                    last_active_at, created_at, cwd_exists, tracked_session_id, metadata_json
             FROM history_entries
             ORDER BY last_active_at DESC, id ASC
             LIMIT ?1",
        )?;
        statement
            .query_map([limit], read_history_entry)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    pub fn upsert_project(&self, project: ProjectUpsert) -> Result<ProjectSummary, StorageError> {
        let root_path = project.root_path.display().to_string();
        let id = self
            .project_id_by_root(&root_path)?
            .unwrap_or_else(|| stable_project_id(&root_path));
        let now = now_unix();
        self.connection.execute(
            "INSERT INTO projects(id, root_path, display_name, remote_origin, pinned_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(root_path) DO UPDATE SET
                display_name = excluded.display_name,
                remote_origin = excluded.remote_origin,
                pinned_order = excluded.pinned_order,
                updated_at = excluded.updated_at",
            params![
                id,
                root_path,
                project.display_name,
                project.remote_origin,
                project.pinned_order,
                now,
                now
            ],
        )?;
        self.project_by_root(project.root_path.display().to_string().as_str())
    }

    pub fn upsert_worktree(
        &self,
        worktree: WorktreeUpsert,
    ) -> Result<WorktreeSummary, StorageError> {
        let path = worktree.path.display().to_string();
        let id = self
            .worktree_id_by_path(&path)?
            .unwrap_or_else(|| Uuid::now_v7().to_string());
        let now = now_unix();
        self.connection.execute(
            "INSERT INTO worktrees(
                id, project_id, session_id, path, branch, head_sha, is_bare, is_detached,
                is_prunable, dirty, merged, stale_suggestion, created_at, updated_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(path) DO UPDATE SET
                project_id = excluded.project_id,
                session_id = excluded.session_id,
                branch = excluded.branch,
                head_sha = excluded.head_sha,
                is_bare = excluded.is_bare,
                is_detached = excluded.is_detached,
                is_prunable = excluded.is_prunable,
                dirty = excluded.dirty,
                merged = excluded.merged,
                stale_suggestion = excluded.stale_suggestion,
                updated_at = excluded.updated_at",
            params![
                id,
                worktree.project_id,
                worktree.session_id,
                path,
                worktree.branch,
                worktree.head_sha,
                bool_to_i64(worktree.is_bare),
                bool_to_i64(worktree.is_detached),
                bool_to_i64(worktree.is_prunable),
                bool_to_i64(worktree.dirty),
                bool_to_i64(worktree.merged),
                bool_to_i64(worktree.stale_suggestion),
                now,
                now
            ],
        )?;
        self.worktree_by_path(worktree.path.display().to_string().as_str())
    }

    pub fn list_worktrees_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<WorktreeSummary>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, session_id, path, branch, head_sha, is_bare, is_detached,
                    is_prunable, dirty, merged, stale_suggestion
             FROM worktrees
             WHERE project_id = ?1
             ORDER BY updated_at DESC, path ASC",
        )?;
        statement
            .query_map([project_id], read_worktree_summary)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectSummary>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT id, root_path, display_name, remote_origin, pinned_order
             FROM projects
             ORDER BY pinned_order ASC NULLS LAST, updated_at DESC, root_path ASC",
        )?;
        statement
            .query_map([], read_project_summary)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    pub fn update_session_core_metadata(
        &self,
        id: &str,
        metadata: SessionCoreMetadataUpdate,
    ) -> Result<(), StorageError> {
        let worktree_path = metadata
            .worktree_path
            .map(|path| path.display().to_string());
        let transcript_path = metadata
            .transcript_path
            .map(|path| path.display().to_string());
        let needs_input_payload_json = serde_json::to_string(&metadata.needs_input_payload)?;
        let changed = self.connection.execute(
            "UPDATE sessions SET
                project_id = ?1,
                worktree_path = ?2,
                git_branch = ?3,
                title_source = ?4,
                agent_session_id = ?5,
                transcript_path = ?6,
                needs_input_kind = ?7,
                needs_input_payload_json = ?8,
                resumability = ?9,
                parent_session_id = ?10,
                pinned = ?11,
                archived_at = ?12,
                remote_active = ?13,
                host_id = ?14,
                foreground_agent_kind = ?15,
                memory_bytes = ?16,
                updated_at = ?17
             WHERE id = ?18",
            params![
                metadata.project_id,
                worktree_path,
                metadata.git_branch,
                metadata.title_source,
                metadata.agent_session_id,
                transcript_path,
                metadata.needs_input_kind,
                needs_input_payload_json,
                metadata.resumability,
                metadata.parent_session_id,
                bool_to_i64(metadata.pinned),
                metadata.archived_at,
                bool_to_i64(metadata.remote_active),
                metadata.host_id,
                metadata.foreground_agent_kind,
                metadata.memory_bytes,
                now_unix(),
                id
            ],
        )?;
        if changed == 0 {
            return Err(StorageError::NotFound {
                entity: "session",
                id: id.to_string(),
            });
        }
        Ok(())
    }

    pub fn session_core_metadata(&self, id: &str) -> Result<SessionCoreMetadata, StorageError> {
        self.connection
            .query_row(
                "SELECT id, project_id, worktree_path, git_branch, title_source, agent_session_id,
                        transcript_path, needs_input_kind, needs_input_payload_json, resumability,
                        parent_session_id, pinned, archived_at, remote_active, host_id,
                        foreground_agent_kind, memory_bytes
                 FROM sessions
                 WHERE id = ?1",
                [id],
                read_session_core_metadata,
            )
            .map_err(StorageError::from)
    }

    pub fn record_usage(&self, usage: RecordUsage) -> Result<bool, StorageError> {
        validate_usage(&usage)?;
        let total_tokens = usage.input_tokens.max(0)
            + usage.output_tokens.max(0)
            + usage.cache_read_tokens.max(0)
            + usage.cache_write_tokens.max(0);
        let changed = self.connection.execute(
            "INSERT OR IGNORE INTO usage_records(
                id, request_id, session_id, agent_profile_id, runtime_id, provider_id,
                llm_profile_id, model, request_kind, status, input_tokens, output_tokens,
                cached_input_tokens, cache_read_tokens, cache_write_tokens,
                cache_write_5m_tokens, cache_write_1h_tokens, cache_hit_rate,
                reasoning_tokens, total_tokens, unit_price_input, unit_price_output,
                currency, pricing_snapshot_id, estimated_cost, billed_cost,
                first_token_latency_ms, total_latency_ms, started_at, completed_at,
                safe_error_code, value_kind, source, source_event_id
             )
             VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10, ?11, ?12,
                ?13, ?14, ?15,
                ?16, ?17, NULL,
                ?18, ?19, ?20, ?21,
                ?22, ?23, ?24, ?25,
                ?26, ?27, ?28, ?29,
                ?30, ?31, ?32, ?33
             )",
            params![
                Uuid::now_v7().to_string(),
                usage.request_id,
                usage.session_id,
                usage.agent_profile_id,
                usage.runtime_id,
                usage.provider_id,
                usage.llm_profile_id,
                usage.model,
                usage.request_kind,
                usage.status,
                usage.input_tokens,
                usage.output_tokens,
                usage.cached_input_tokens,
                usage.cache_read_tokens,
                usage.cache_write_tokens,
                usage.cache_write_5m_tokens,
                usage.cache_write_1h_tokens,
                usage.reasoning_tokens,
                total_tokens,
                usage.unit_price_input,
                usage.unit_price_output,
                usage.currency,
                usage.pricing_snapshot_id,
                usage.estimated_cost,
                usage.billed_cost,
                usage.first_token_latency_ms,
                usage.total_latency_ms,
                usage.started_at,
                usage.completed_at,
                usage.safe_error_code,
                usage.value_kind,
                usage.source,
                usage.source_event_id
            ],
        )?;
        Ok(changed != 0)
    }

    pub fn record_transcript_usage_event(
        &self,
        event: &homie_llm::TranscriptUsageEvent,
        defaults: &UsageImportDefaults,
    ) -> Result<bool, StorageError> {
        self.record_usage(transcript_event_to_record(event, defaults))
    }

    pub fn record_transcript_usage_events(
        &self,
        events: &[homie_llm::TranscriptUsageEvent],
        defaults: &UsageImportDefaults,
    ) -> Result<UsageImportResult, StorageError> {
        let mut result = UsageImportResult::default();
        for event in events {
            if self.record_transcript_usage_event(event, defaults)? {
                result.inserted += 1;
            } else {
                result.skipped += 1;
            }
        }
        Ok(result)
    }

    pub fn upsert_usage_scan_file(&self, state: UsageScanFileState) -> Result<(), StorageError> {
        validate_usage_scan_file(&state)?;
        self.connection.execute(
            "INSERT INTO usage_scan_files(
                path, provider, profile_id, size, offset, modified_ns,
                device, inode, tail_hash, model, scanned_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(path) DO UPDATE SET
                provider = excluded.provider,
                profile_id = excluded.profile_id,
                size = excluded.size,
                offset = excluded.offset,
                modified_ns = excluded.modified_ns,
                device = excluded.device,
                inode = excluded.inode,
                tail_hash = excluded.tail_hash,
                model = excluded.model,
                scanned_at = excluded.scanned_at",
            params![
                state.path,
                state.provider,
                state.profile_id,
                state.size,
                state.offset,
                state.modified_ns,
                state.device,
                state.inode,
                state.tail_hash,
                state.model,
                state.scanned_at,
            ],
        )?;
        Ok(())
    }

    pub fn usage_scan_file(&self, path: &str) -> Result<Option<UsageScanFileState>, StorageError> {
        self.connection
            .query_row(
                "SELECT path, provider, profile_id, size, offset, modified_ns,
                        device, inode, tail_hash, model, scanned_at
                 FROM usage_scan_files
                 WHERE path = ?1",
                [path],
                read_usage_scan_file_state,
            )
            .optional()
            .map_err(StorageError::from)
    }

    pub fn list_usage_scan_files(
        &self,
        query: UsageScanFileQuery,
    ) -> Result<Vec<UsageScanFileState>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT path, provider, profile_id, size, offset, modified_ns,
                    device, inode, tail_hash, model, scanned_at
             FROM usage_scan_files
             WHERE (?1 IS NULL OR provider = ?1)
               AND (?2 IS NULL OR profile_id = ?2)
             ORDER BY path",
        )?;
        let rows = statement.query_map(params![query.provider, query.profile_id], |row| {
            read_usage_scan_file_state(row)
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    pub fn query_usage_totals(&self, query: UsageQuery) -> Result<UsageTotals, StorageError> {
        self.connection
            .query_row(
                "SELECT
                    COUNT(*),
                    COALESCE(SUM(input_tokens), 0),
                    COALESCE(SUM(output_tokens), 0),
                    COALESCE(SUM(cached_input_tokens), 0),
                    COALESCE(SUM(cache_read_tokens), 0),
                    COALESCE(SUM(cache_write_tokens), 0),
                    COALESCE(SUM(cache_write_5m_tokens), 0),
                    COALESCE(SUM(cache_write_1h_tokens), 0),
                    COALESCE(SUM(reasoning_tokens), 0),
                    COALESCE(SUM(total_tokens), 0),
                    COALESCE(SUM(CAST(estimated_cost AS REAL)), 0.0),
                    COALESCE(SUM(CAST(billed_cost AS REAL)), 0.0),
                    COALESCE(MAX(CASE WHEN billed_cost IS NOT NULL THEN 1 ELSE 0 END), 0)
                 FROM usage_records
                 WHERE (?1 IS NULL OR session_id = ?1)
                   AND (?2 IS NULL OR provider_id = ?2)
                   AND (?3 IS NULL OR model = ?3)
                   AND (?4 IS NULL OR started_at >= ?4)
                   AND (?5 IS NULL OR started_at < ?5)",
                params![
                    query.session_id,
                    query.provider_id,
                    query.model,
                    query.from,
                    query.to
                ],
                |row| {
                    Ok(UsageTotals {
                        events: row.get(0)?,
                        input_tokens: row.get(1)?,
                        output_tokens: row.get(2)?,
                        cached_input_tokens: row.get(3)?,
                        cache_read_tokens: row.get(4)?,
                        cache_write_tokens: row.get(5)?,
                        cache_write_5m_tokens: row.get(6)?,
                        cache_write_1h_tokens: row.get(7)?,
                        reasoning_tokens: row.get(8)?,
                        total_tokens: row.get(9)?,
                        estimated_cost: row.get(10)?,
                        billed_cost: row.get(11)?,
                        authoritative_billing_available: row.get::<_, i64>(12)? == 1,
                    })
                },
            )
            .map_err(StorageError::from)
    }

    pub fn get_preference_json(&self, key: &str) -> Result<Option<Value>, StorageError> {
        let value = self
            .connection
            .query_row(
                "SELECT value_json FROM preferences WHERE key = ?1",
                [key],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        value
            .map(|value| serde_json::from_str(&value))
            .transpose()
            .map_err(StorageError::from)
    }

    pub fn set_preference_json(&self, key: &str, value: &Value) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT INTO preferences(key, value_json, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at",
            params![key, serde_json::to_string(value)?, now_unix()],
        )?;
        Ok(())
    }

    pub fn load_settings_preferences(&self) -> Result<SettingsPreferences, StorageError> {
        let Some(value) = self.get_preference_json("settings")? else {
            return Ok(SettingsPreferences::default());
        };
        Ok(serde_json::from_value(value)?)
    }

    pub fn save_settings_preferences(
        &self,
        preferences: &SettingsPreferences,
    ) -> Result<(), StorageError> {
        self.set_preference_json("settings", &serde_json::to_value(preferences)?)?;
        Ok(())
    }

    pub fn mark_interrupted_sessions_detached(&self) -> Result<usize, StorageError> {
        let changed = self.connection.execute(
            "UPDATE sessions
             SET status = 'detached', updated_at = ?1
             WHERE status IN ('created', 'starting', 'running')",
            params![now_unix()],
        )?;
        Ok(changed)
    }

    pub fn mark_session_running_if_exists(
        &self,
        id: &str,
    ) -> Result<Option<SessionSummary>, StorageError> {
        let changed = self.connection.execute(
            "UPDATE sessions SET status = 'running', updated_at = ?1 WHERE id = ?2",
            params![now_unix(), id],
        )?;
        if changed == 0 {
            return Ok(None);
        }
        self.session_by_id(id).map(Some)
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

    fn table_columns(&self, table: &str) -> Result<Vec<String>, StorageError> {
        let mut statement = self
            .connection
            .prepare(&format!("PRAGMA table_info({})", quote_identifier(table)))?;
        statement
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    fn unique_indexes(&self, table: &str) -> Result<Vec<Vec<String>>, StorageError> {
        let mut statement = self
            .connection
            .prepare(&format!("PRAGMA index_list({})", quote_identifier(table)))?;
        let indexes = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(1)?, row.get::<_, i64>(2)? == 1))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut unique_indexes = Vec::new();
        for (index, is_unique) in indexes {
            if !is_unique {
                continue;
            }
            let mut index_statement = self
                .connection
                .prepare(&format!("PRAGMA index_info({})", quote_identifier(&index)))?;
            let columns = index_statement
                .query_map([], |row| row.get::<_, String>(2))?
                .collect::<Result<Vec<_>, _>>()?;
            unique_indexes.push(columns);
        }
        Ok(unique_indexes)
    }

    fn history_id(
        &self,
        agent_kind: &str,
        external_id: &str,
    ) -> Result<Option<String>, StorageError> {
        self.connection
            .query_row(
                "SELECT id FROM history_entries WHERE agent_kind = ?1 AND external_id = ?2",
                params![agent_kind, external_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(StorageError::from)
    }

    fn history_by_agent_external(
        &self,
        agent_kind: &str,
        external_id: &str,
    ) -> Result<HistoryEntrySummary, StorageError> {
        self.connection
            .query_row(
                "SELECT id, agent_kind, external_id, cwd, title, title_source, transcript_path,
                        last_active_at, created_at, cwd_exists, tracked_session_id, metadata_json
                 FROM history_entries
                 WHERE agent_kind = ?1 AND external_id = ?2",
                params![agent_kind, external_id],
                read_history_entry,
            )
            .map_err(StorageError::from)
    }

    fn project_id_by_root(&self, root_path: &str) -> Result<Option<String>, StorageError> {
        self.connection
            .query_row(
                "SELECT id FROM projects WHERE root_path = ?1",
                [root_path],
                |row| row.get(0),
            )
            .optional()
            .map_err(StorageError::from)
    }

    fn project_by_root(&self, root_path: &str) -> Result<ProjectSummary, StorageError> {
        self.connection
            .query_row(
                "SELECT id, root_path, display_name, remote_origin, pinned_order
                 FROM projects
                 WHERE root_path = ?1",
                [root_path],
                read_project_summary,
            )
            .map_err(StorageError::from)
    }

    fn worktree_id_by_path(&self, path: &str) -> Result<Option<String>, StorageError> {
        self.connection
            .query_row("SELECT id FROM worktrees WHERE path = ?1", [path], |row| {
                row.get(0)
            })
            .optional()
            .map_err(StorageError::from)
    }

    fn worktree_by_path(&self, path: &str) -> Result<WorktreeSummary, StorageError> {
        self.connection
            .query_row(
                "SELECT id, project_id, session_id, path, branch, head_sha, is_bare, is_detached,
                        is_prunable, dirty, merged, stale_suggestion
                 FROM worktrees
                 WHERE path = ?1",
                [path],
                read_worktree_summary,
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

fn read_history_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<HistoryEntrySummary> {
    let metadata_json: String = row.get(11)?;
    Ok(HistoryEntrySummary {
        id: row.get(0)?,
        agent_kind: row.get(1)?,
        external_id: row.get(2)?,
        cwd: row.get(3)?,
        title: row.get(4)?,
        title_source: row.get(5)?,
        transcript_path: row.get(6)?,
        last_active_at: row.get(7)?,
        created_at: row.get(8)?,
        cwd_exists: row.get::<_, i64>(9)? == 1,
        tracked_session_id: row.get(10)?,
        metadata: parse_json_column(metadata_json, 11)?,
    })
}

fn read_project_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectSummary> {
    Ok(ProjectSummary {
        id: row.get(0)?,
        root_path: row.get(1)?,
        display_name: row.get(2)?,
        remote_origin: row.get(3)?,
        pinned_order: row.get(4)?,
    })
}

fn read_worktree_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<WorktreeSummary> {
    Ok(WorktreeSummary {
        id: row.get(0)?,
        project_id: row.get(1)?,
        session_id: row.get(2)?,
        path: row.get(3)?,
        branch: row.get(4)?,
        head_sha: row.get(5)?,
        is_bare: row.get::<_, i64>(6)? == 1,
        is_detached: row.get::<_, i64>(7)? == 1,
        is_prunable: row.get::<_, i64>(8)? == 1,
        dirty: row.get::<_, i64>(9)? == 1,
        merged: row.get::<_, i64>(10)? == 1,
        stale_suggestion: row.get::<_, i64>(11)? == 1,
    })
}

fn read_session_core_metadata(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionCoreMetadata> {
    let needs_input_payload_json: String = row.get(8)?;
    Ok(SessionCoreMetadata {
        id: row.get(0)?,
        project_id: row.get(1)?,
        worktree_path: row.get(2)?,
        git_branch: row.get(3)?,
        title_source: row.get(4)?,
        agent_session_id: row.get(5)?,
        transcript_path: row.get(6)?,
        needs_input_kind: row.get(7)?,
        needs_input_payload: parse_json_column(needs_input_payload_json, 8)?,
        resumability: row.get(9)?,
        parent_session_id: row.get(10)?,
        pinned: row.get::<_, i64>(11)? == 1,
        archived_at: row.get(12)?,
        remote_active: row.get::<_, i64>(13)? == 1,
        host_id: row.get(14)?,
        foreground_agent_kind: row.get(15)?,
        memory_bytes: row.get(16)?,
    })
}

fn read_usage_scan_file_state(row: &rusqlite::Row<'_>) -> rusqlite::Result<UsageScanFileState> {
    Ok(UsageScanFileState {
        path: row.get(0)?,
        provider: row.get(1)?,
        profile_id: row.get(2)?,
        size: row.get(3)?,
        offset: row.get(4)?,
        modified_ns: row.get(5)?,
        device: row.get(6)?,
        inode: row.get(7)?,
        tail_hash: row.get(8)?,
        model: row.get(9)?,
        scanned_at: row.get(10)?,
    })
}

fn parse_json_column(value: String, column: usize) -> rusqlite::Result<Value> {
    serde_json::from_str(&value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            column,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

fn bool_to_i64(value: bool) -> i64 {
    i64::from(value)
}

fn transcript_event_to_record(
    event: &homie_llm::TranscriptUsageEvent,
    defaults: &UsageImportDefaults,
) -> RecordUsage {
    RecordUsage {
        request_id: event.source_event_id.clone(),
        session_id: event.session_id.clone(),
        agent_profile_id: defaults.agent_profile_id.clone(),
        runtime_id: defaults.runtime_id.clone(),
        provider_id: defaults.provider_id.clone(),
        llm_profile_id: defaults.llm_profile_id.clone(),
        model: event.model.clone().unwrap_or_else(|| "unknown".to_string()),
        request_kind: defaults.request_kind.clone(),
        status: defaults.status.clone(),
        input_tokens: event.input_tokens,
        output_tokens: event.output_tokens,
        cached_input_tokens: 0,
        cache_read_tokens: event.cache_read_tokens,
        cache_write_tokens: event.cache_write_tokens,
        cache_write_5m_tokens: event.cache_write_5m_tokens,
        cache_write_1h_tokens: event.cache_write_1h_tokens,
        reasoning_tokens: 0,
        unit_price_input: None,
        unit_price_output: None,
        currency: Some("USD".to_string()),
        pricing_snapshot_id: None,
        estimated_cost: event.estimated_cost.map(format_cost),
        billed_cost: event.billed_cost.map(format_cost),
        first_token_latency_ms: None,
        total_latency_ms: None,
        started_at: event.occurred_at,
        completed_at: event.occurred_at,
        safe_error_code: None,
        value_kind: event.value_kind.as_str().to_string(),
        source: event.source.as_str().to_string(),
        source_event_id: event.source_event_id.clone(),
    }
}

fn format_cost(cost: f64) -> String {
    format!("{cost:.12}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn validate_usage_scan_file(state: &UsageScanFileState) -> Result<(), StorageError> {
    for (field, value) in [
        ("path", state.path.as_str()),
        ("provider", state.provider.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(StorageError::InvalidInput(format!("{field} is required")));
        }
    }
    for (field, value) in [
        ("size", state.size),
        ("offset", state.offset),
        ("modified_ns", state.modified_ns),
        ("tail_hash", state.tail_hash),
        ("scanned_at", state.scanned_at),
    ] {
        if value < 0 {
            return Err(StorageError::InvalidInput(format!(
                "{field} cannot be negative"
            )));
        }
    }
    Ok(())
}

fn validate_usage(usage: &RecordUsage) -> Result<(), StorageError> {
    for (field, value) in [
        ("request_id", usage.request_id.as_str()),
        ("agent_profile_id", usage.agent_profile_id.as_str()),
        ("runtime_id", usage.runtime_id.as_str()),
        ("provider_id", usage.provider_id.as_str()),
        ("llm_profile_id", usage.llm_profile_id.as_str()),
        ("model", usage.model.as_str()),
        ("request_kind", usage.request_kind.as_str()),
        ("status", usage.status.as_str()),
        ("value_kind", usage.value_kind.as_str()),
        ("source", usage.source.as_str()),
        ("source_event_id", usage.source_event_id.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(StorageError::InvalidInput(format!("{field} is required")));
        }
    }

    let token_fields = [
        ("input_tokens", usage.input_tokens),
        ("output_tokens", usage.output_tokens),
        ("cached_input_tokens", usage.cached_input_tokens),
        ("cache_read_tokens", usage.cache_read_tokens),
        ("cache_write_tokens", usage.cache_write_tokens),
        ("cache_write_5m_tokens", usage.cache_write_5m_tokens),
        ("cache_write_1h_tokens", usage.cache_write_1h_tokens),
        ("reasoning_tokens", usage.reasoning_tokens),
    ];
    if let Some((field, _)) = token_fields.iter().find(|(_, value)| *value < 0) {
        return Err(StorageError::InvalidInput(format!(
            "{field} cannot be negative"
        )));
    }
    validate_non_negative_decimal("estimated_cost", usage.estimated_cost.as_deref())?;
    validate_non_negative_decimal("billed_cost", usage.billed_cost.as_deref())?;
    validate_non_negative_decimal("unit_price_input", usage.unit_price_input.as_deref())?;
    validate_non_negative_decimal("unit_price_output", usage.unit_price_output.as_deref())?;
    Ok(())
}

fn validate_non_negative_decimal(field: &str, value: Option<&str>) -> Result<(), StorageError> {
    let Some(value) = value else {
        return Ok(());
    };
    let parsed = value
        .parse::<f64>()
        .map_err(|_| StorageError::InvalidInput(format!("{field} must be numeric")))?;
    if !parsed.is_finite() || parsed < 0.0 {
        return Err(StorageError::InvalidInput(format!(
            "{field} cannot be negative or non-finite"
        )));
    }
    Ok(())
}

fn now_unix() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn stable_project_id(root_path: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in root_path.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    format!("p_{:012x}", hash & 0xFFFF_FFFF_FFFF)
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

const SCHEMA_V2: &str = r#"
CREATE TABLE preferences (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    root_path TEXT NOT NULL UNIQUE,
    display_name TEXT,
    remote_origin TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE worktrees (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    path TEXT NOT NULL UNIQUE,
    branch TEXT,
    dirty INTEGER NOT NULL CHECK(dirty IN (0, 1)),
    merged INTEGER NOT NULL CHECK(merged IN (0, 1)),
    stale_suggestion INTEGER NOT NULL CHECK(stale_suggestion IN (0, 1)),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE session_artifacts (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    url TEXT NOT NULL,
    label TEXT,
    metadata_json TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(session_id, kind, url)
);

CREATE TABLE listening_ports (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    port INTEGER NOT NULL,
    protocol TEXT NOT NULL,
    label TEXT,
    url TEXT,
    discovered_at INTEGER NOT NULL,
    UNIQUE(session_id, port, protocol)
);

CREATE TABLE pull_request_statuses (
    id TEXT PRIMARY KEY,
    artifact_id TEXT NOT NULL REFERENCES session_artifacts(id) ON DELETE CASCADE,
    url TEXT NOT NULL UNIQUE,
    state TEXT NOT NULL,
    review_decision TEXT,
    mergeable TEXT,
    checks_passed INTEGER NOT NULL DEFAULT 0,
    checks_failed INTEGER NOT NULL DEFAULT 0,
    checks_pending INTEGER NOT NULL DEFAULT 0,
    comment_count INTEGER NOT NULL DEFAULT 0,
    review_count INTEGER NOT NULL DEFAULT 0,
    additions INTEGER NOT NULL DEFAULT 0,
    deletions INTEGER NOT NULL DEFAULT 0,
    metadata_json TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE history_entries (
    id TEXT PRIMARY KEY,
    agent_kind TEXT NOT NULL,
    cwd TEXT NOT NULL,
    title TEXT,
    transcript_path TEXT NOT NULL,
    last_active_at INTEGER NOT NULL,
    created_at INTEGER,
    cwd_exists INTEGER NOT NULL CHECK(cwd_exists IN (0, 1)),
    tracked_session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    UNIQUE(agent_kind, id)
);

CREATE TABLE hosts (
    id TEXT PRIMARY KEY,
    name TEXT,
    ssh TEXT NOT NULL,
    default_cwd TEXT,
    node_endpoint TEXT,
    node_token_file TEXT,
    node_id TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE node_accounts (
    id TEXT PRIMARY KEY,
    host_id TEXT NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    profile_id TEXT NOT NULL,
    label TEXT NOT NULL,
    is_default INTEGER NOT NULL CHECK(is_default IN (0, 1)),
    status TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(host_id, provider, profile_id)
);

CREATE TABLE handoff_records (
    id TEXT PRIMARY KEY,
    source_session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    target_host_id TEXT NOT NULL REFERENCES hosts(id) ON DELETE RESTRICT,
    mode TEXT NOT NULL,
    status TEXT NOT NULL,
    checkpoint_manifest_json TEXT NOT NULL,
    safe_error_code TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE memory_candidates (
    id TEXT PRIMARY KEY,
    source_event_id TEXT NOT NULL REFERENCES context_events(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    sensitivity TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
"#;

const SCHEMA_V3: &str = r#"
ALTER TABLE projects ADD COLUMN pinned_order INTEGER;

ALTER TABLE worktrees ADD COLUMN head_sha TEXT;
ALTER TABLE worktrees ADD COLUMN is_bare INTEGER NOT NULL DEFAULT 0 CHECK(is_bare IN (0, 1));
ALTER TABLE worktrees ADD COLUMN is_detached INTEGER NOT NULL DEFAULT 0 CHECK(is_detached IN (0, 1));
ALTER TABLE worktrees ADD COLUMN is_prunable INTEGER NOT NULL DEFAULT 0 CHECK(is_prunable IN (0, 1));

ALTER TABLE sessions ADD COLUMN project_id TEXT REFERENCES projects(id) ON DELETE SET NULL;
ALTER TABLE sessions ADD COLUMN worktree_path TEXT;
ALTER TABLE sessions ADD COLUMN git_branch TEXT;
ALTER TABLE sessions ADD COLUMN title_source TEXT NOT NULL DEFAULT 'placeholder';
ALTER TABLE sessions ADD COLUMN agent_session_id TEXT;
ALTER TABLE sessions ADD COLUMN transcript_path TEXT;
ALTER TABLE sessions ADD COLUMN needs_input_kind TEXT;
ALTER TABLE sessions ADD COLUMN needs_input_payload_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE sessions ADD COLUMN resumability TEXT NOT NULL DEFAULT 'not_resumable';
ALTER TABLE sessions ADD COLUMN parent_session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL;
ALTER TABLE sessions ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0 CHECK(pinned IN (0, 1));
ALTER TABLE sessions ADD COLUMN archived_at INTEGER;
ALTER TABLE sessions ADD COLUMN remote_active INTEGER NOT NULL DEFAULT 0 CHECK(remote_active IN (0, 1));
ALTER TABLE sessions ADD COLUMN host_id TEXT REFERENCES hosts(id) ON DELETE SET NULL;
ALTER TABLE sessions ADD COLUMN foreground_agent_kind TEXT;
ALTER TABLE sessions ADD COLUMN memory_bytes INTEGER;

ALTER TABLE history_entries RENAME TO history_entries_v2;

CREATE TABLE history_entries (
    id TEXT PRIMARY KEY,
    agent_kind TEXT NOT NULL,
    external_id TEXT NOT NULL,
    cwd TEXT NOT NULL,
    title TEXT,
    title_source TEXT NOT NULL DEFAULT 'placeholder',
    transcript_path TEXT NOT NULL,
    last_active_at INTEGER NOT NULL,
    created_at INTEGER,
    cwd_exists INTEGER NOT NULL CHECK(cwd_exists IN (0, 1)),
    tracked_session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    UNIQUE(agent_kind, external_id)
);

INSERT INTO history_entries(
    id, agent_kind, external_id, cwd, title, title_source, transcript_path,
    last_active_at, created_at, cwd_exists, tracked_session_id, metadata_json
)
SELECT
    id, agent_kind, id, cwd, title, 'placeholder', transcript_path,
    last_active_at, created_at, cwd_exists, tracked_session_id, '{}'
FROM history_entries_v2;

DROP TABLE history_entries_v2;

ALTER TABLE model_pricing ADD COLUMN cache_write_5m_price_per_million TEXT;
ALTER TABLE model_pricing ADD COLUMN cache_write_1h_price_per_million TEXT;
ALTER TABLE pricing_snapshots ADD COLUMN cache_write_5m_price_per_million TEXT;
ALTER TABLE pricing_snapshots ADD COLUMN cache_write_1h_price_per_million TEXT;

ALTER TABLE usage_records ADD COLUMN cache_write_5m_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE usage_records ADD COLUMN cache_write_1h_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE usage_records ADD COLUMN billed_cost TEXT;
ALTER TABLE usage_records ADD COLUMN value_kind TEXT NOT NULL DEFAULT 'estimated_api_equivalent';
ALTER TABLE usage_records ADD COLUMN source TEXT NOT NULL DEFAULT 'app_server';
ALTER TABLE usage_records ADD COLUMN source_event_id TEXT;

UPDATE usage_records
SET source_event_id = request_id
WHERE source_event_id IS NULL;

CREATE UNIQUE INDEX usage_records_provider_source_event
ON usage_records(provider_id, source, source_event_id);

CREATE TABLE usage_scan_files (
    path TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    profile_id TEXT,
    size INTEGER NOT NULL,
    offset INTEGER NOT NULL,
    modified_ns INTEGER NOT NULL,
    device INTEGER,
    inode INTEGER,
    tail_hash INTEGER NOT NULL,
    model TEXT,
    scanned_at INTEGER NOT NULL
);

CREATE TABLE usage_hourly_rollups (
    id TEXT PRIMARY KEY,
    hour_start INTEGER NOT NULL,
    provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    profile_id TEXT,
    session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    model TEXT NOT NULL,
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    cached_input_tokens INTEGER NOT NULL DEFAULT 0,
    cache_read_tokens INTEGER NOT NULL DEFAULT 0,
    cache_write_tokens INTEGER NOT NULL DEFAULT 0,
    cache_write_5m_tokens INTEGER NOT NULL DEFAULT 0,
    cache_write_1h_tokens INTEGER NOT NULL DEFAULT 0,
    reasoning_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    estimated_cost TEXT,
    billed_cost TEXT,
    currency TEXT,
    updated_at INTEGER NOT NULL,
    UNIQUE(hour_start, provider_id, profile_id, session_id, model)
);

CREATE INDEX projects_pinned_order
ON projects(pinned_order)
WHERE pinned_order IS NOT NULL;

CREATE UNIQUE INDEX worktrees_project_branch
ON worktrees(project_id, branch)
WHERE branch IS NOT NULL;

CREATE INDEX worktrees_project_updated
ON worktrees(project_id, updated_at DESC);

CREATE INDEX worktrees_session
ON worktrees(session_id);

CREATE INDEX sessions_project_updated
ON sessions(project_id, updated_at DESC);

CREATE INDEX sessions_agent_session
ON sessions(agent_session_id)
WHERE agent_session_id IS NOT NULL;

CREATE INDEX sessions_status_updated
ON sessions(status, updated_at DESC);

CREATE INDEX history_entries_recent
ON history_entries(last_active_at DESC, id);

CREATE INDEX history_entries_cwd
ON history_entries(cwd);

CREATE INDEX model_pricing_lookup
ON model_pricing(provider_id, model, effective_at DESC);

CREATE INDEX pricing_snapshots_lookup
ON pricing_snapshots(provider_id, model, captured_at DESC);

CREATE INDEX usage_records_time
ON usage_records(started_at);

CREATE INDEX usage_records_session_time
ON usage_records(session_id, started_at);

CREATE INDEX usage_records_provider_model_time
ON usage_records(provider_id, model, started_at);

CREATE INDEX usage_scan_files_provider
ON usage_scan_files(provider, profile_id);

CREATE INDEX usage_hourly_rollups_time
ON usage_hourly_rollups(hour_start);

CREATE INDEX usage_hourly_rollups_session
ON usage_hourly_rollups(session_id, hour_start);
"#;
