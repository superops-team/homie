# Storage & Indexing 组件规格

## 1. 组件定位

`homie-storage` 是 Homie V1 的本地事实源组件，负责 SQLite 数据库创建、schema migration、关系约束、健康检查和 repository API。

## 2. 来源需求映射

- PRD: `prd-spec/features/workspace-storage-bootstrap/2026-08-05-workspace-storage-bootstrap-design.md`
- V1 架构: `prd-spec/features/homie-v1-architecture/2026-08-05-homie-v1-architecture-design.md`
- OpenSpec: `openspec/changes/workspace-storage-bootstrap/`
- 功能验证: `docs/verification/workspace-storage-bootstrap/functional-cases.md`

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | `homie-cli` | 调用 storage health/migration 用于 `doctor` |
| 上游 | `homie-runtime` | 后续读写 session/context |
| 上游 | `homie-llm` | 后续读写 provider/usage/virtual key |
| 下游 | SQLite | 本地数据库文件 |
| 下游 | output log files | 大流式输出文件，由 SQLite 索引 |

## 4. 职责边界

负责：

- data dir 解析后的 SQLite path 生成。
- SQLite 打开/创建。
- `PRAGMA foreign_keys = ON`。
- WAL 模式。
- forward-only migration。
- schema health check。
- schema 约束测试。

不负责：

- provider raw key 加密实现。
- runtime process 生命周期。
- LLM proxy 请求转发。
- GPUI UI。
- MCP server proxy。

## 5. 核心接口

```rust
pub struct StorageConfig {
    pub data_dir: PathBuf,
}

pub struct Storage;

pub struct MigrationReport {
    pub schema_version: i64,
    pub applied: Vec<i64>,
}

pub struct StorageHealth {
    pub database_path: PathBuf,
    pub schema_version: i64,
    pub foreign_keys: bool,
    pub journal_mode: String,
}

pub fn open_or_create(config: StorageConfig) -> Result<Storage, StorageError>;

impl Storage {
    pub fn migrate(&self) -> Result<MigrationReport, StorageError>;
    pub fn health_check(&self) -> Result<StorageHealth, StorageError>;
}
```

## 6. 数据模型

V1 schema version: `1`

核心表：

- `schema_migrations`
- `providers`
- `llm_profiles`
- `model_pricing`
- `pricing_snapshots`
- `runtime_descriptors`
- `permission_profiles`
- `agent_profiles`
- `skills`
- `agent_profile_skills`
- `mcp_servers`
- `agent_profile_mcp_servers`
- `effective_agent_configs`
- `sessions`
- `context_events`
- `virtual_keys`
- `usage_records`
- `tool_call_metrics`
- `tasks`
- `config_events`
- `metrics_write_failures`

## 7. 运行模型与状态机

```text
open_or_create
  -> create data dir
  -> open homie.sqlite
  -> set PRAGMAs
  -> migrate
  -> health_check
```

Migration 是 forward-only。不存在 downgrade、fallback 或兼容旧 schema。

## 8. 安全与权限

- SQLite 文件在用户数据目录内创建。
- raw provider key 不进入 SQLite。
- secret 只保存 `api_key_ref` 或 envelope ref。
- metrics 表不得保存 raw request/response、Authorization、cookie、完整 tool args/result。

## 9. 可观测性

`health_check` 输出：

- database path
- schema version
- foreign keys enabled
- journal mode

CLI doctor 使用该输出生成 human/json report。

## 10. 失败与恢复

| 场景 | 行为 |
|------|------|
| data dir 不存在 | 创建 |
| SQLite 无法打开 | 返回 error，不吞掉 |
| migration 中途失败 | transaction rollback |
| schema version 过新 | fail closed |
| corrupt database | 当前切片只返回 error；后续组件 spec 增加 quarantine |

## 11. 测试计划与验收引用

- FC-001: doctor 创建 SQLite。
- FC-002: doctor 幂等。
- FC-003: SQLite 关系约束。
- FC-004: usage schema 支持 token/cache/cost/latency。
- FC-005: workspace 质量门禁。

## Diri Parity Mapping

This section is mandatory for Diri-to-Homie parity planning and fixes the review gap recorded in `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`.

| Field | Value |
|-------|-------|
| Owned feature atoms | M02-F002, M05-F002, M06-F001, M07-F001, M07-F002, M17-F001, M19-F001 |
| Required Diri test mapping | migration/idempotency, preferences, history, worktrees, artifacts, usage schema tests |
| Pre-implementation gaps | table-by-table schema/API inventory |

Rules:

- A PRD/OpenSpec change touching this component must cite the owned feature atom ids above.
- Implementation is not complete until the required Diri test mapping has an equivalent Homie verification gate.
- If a new Diri source/test is discovered for this component, update `docs/research/diri-module-inventory.md` and this section before coding.

## 12. Diri Storage/Indexing Phase 1 Schema Inventory

This inventory closes the P0 review gap from `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`: M05/M06/M07/M17/M19 must have table fields, unique constraints, query API ownership, and verification gates before dependent lanes implement against storage.

Scope rule: this component stores durable facts and exposes repository/query APIs. It does not perform filesystem scans, git shell commands, transcript parsing, LLM proxying, runtime process management, or UI rendering.

### 12.1 Feature Atom Ownership

| Atom | Diri source | Storage responsibility | Non-storage owner |
|------|-------------|------------------------|-------------------|
| M05-F002 | `HistoryScanner.swift`, `CodexTranscript.swift`, `CursorChatStore.swift`, `TitleWatcher.swift` | Store scanned history entries, transcript references, cwd existence, title source, tracked session link, last active ordering | `session-context-store`, runtime scanner, app history UI |
| M06-F001 | `settings.rs`, `PrefsSync.swift`, resource governor settings tests | Persist settings/preferences payloads by stable key with safe JSON only | desktop settings UI, remote prefs sync |
| M07-F001 | `RepoLocator.swift`, `GitWorktrees.swift`, `WorktreeDetectionTests.swift` | Store project roots and discovered worktree rows with branch and worktree flags | runtime git detector |
| M07-F002 | `WorktreeCommands.swift`, `worktrees.rs` | Store worktree/session linkage and cleanup suggestion flags | CLI/runtime worktree commands |
| M17-F001 | `SessionRecord.swift`, `SessionStatus.swift`, `Identifiers.swift`, `Attention.swift` | Store core session/project identifiers, status labels, title source, agent session id, transcript path, needs-input/resumability/parent/pin/archive/remote/host/foreground-agent metadata | `homie-proto`, `homie-agents`, runtime reducer, UI projection |
| M19-F001 | `diri-usage/src/lib.rs`, `diri-node/src/usage.rs`, app usage parser/store | Store usage events, pricing snapshots, transcript scan checkpoints, hourly rollups, queryable totals | `homie-llm`, usage parser, usage UI |

### 12.2 Table Inventory

| Table | Atom | Required fields | Required unique constraints | Required indexes |
|-------|------|-----------------|-----------------------------|------------------|
| `preferences` | M06-F001 | `key`, `value_json`, `updated_at` | `PRIMARY KEY(key)` | none in phase 1 |
| `projects` | M07-F001, M17-F001 | `id`, `root_path`, `display_name`, `remote_origin`, `pinned_order`, `created_at`, `updated_at` | `PRIMARY KEY(id)`, `UNIQUE(root_path)` | `projects_pinned_order` on `pinned_order` where non-null |
| `worktrees` | M07-F001, M07-F002 | `id`, `project_id`, `session_id`, `path`, `branch`, `head_sha`, `is_bare`, `is_detached`, `is_prunable`, `dirty`, `merged`, `stale_suggestion`, `created_at`, `updated_at` | `PRIMARY KEY(id)`, `UNIQUE(path)`, partial unique `(project_id, branch)` where `branch IS NOT NULL` | `worktrees_project_updated` on `(project_id, updated_at DESC)`, `worktrees_session` on `session_id` |
| `sessions` | M17-F001 | existing session launch fields plus `project_id`, `worktree_path`, `git_branch`, `title_source`, `agent_session_id`, `transcript_path`, `needs_input_kind`, `needs_input_payload_json`, `resumability`, `parent_session_id`, `pinned`, `archived_at`, `remote_active`, `host_id`, `foreground_agent_kind`, `memory_bytes` | `PRIMARY KEY(id)` | `sessions_project_updated`, `sessions_agent_session`, `sessions_status_updated` |
| `history_entries` | M05-F002 | `id`, `agent_kind`, `external_id`, `cwd`, `title`, `title_source`, `transcript_path`, `last_active_at`, `created_at`, `cwd_exists`, `tracked_session_id`, `metadata_json` | `PRIMARY KEY(id)`, `UNIQUE(agent_kind, external_id)` | `history_entries_recent` on `(last_active_at DESC, id)`, `history_entries_cwd` on `cwd` |
| `model_pricing` | M19-F001 | `id`, `provider_id`, `model`, `input_price_per_million`, `output_price_per_million`, `cached_input_price_per_million`, `cache_write_5m_price_per_million`, `cache_write_1h_price_per_million`, `currency`, `effective_at`, `created_at` | `PRIMARY KEY(id)`, `UNIQUE(provider_id, model, effective_at)` | `model_pricing_lookup` on `(provider_id, model, effective_at DESC)` |
| `pricing_snapshots` | M19-F001 | pricing fields copied from `model_pricing`, `source_pricing_id`, `captured_at` | `PRIMARY KEY(id)` | `pricing_snapshots_lookup` on `(provider_id, model, captured_at DESC)` |
| `usage_records` | M19-F001 | request/session/profile/provider/model/status fields, token/cache/reasoning totals, `cache_write_5m_tokens`, `cache_write_1h_tokens`, unit prices, cost, latency, `value_kind`, `source`, `source_event_id`, `safe_error_code`, timestamps | `PRIMARY KEY(id)`, `UNIQUE(provider_id, source, source_event_id)` | `usage_records_time`, `usage_records_session_time`, `usage_records_provider_model_time` |
| `usage_scan_files` | M19-F001 | `path`, `provider`, `profile_id`, `size`, `offset`, `modified_ns`, `device`, `inode`, `tail_hash`, `model`, `scanned_at` | `PRIMARY KEY(path)` | `usage_scan_files_provider` on `(provider, profile_id)` |
| `usage_hourly_rollups` | M19-F001 | `hour_start`, `provider_id`, `profile_id`, `session_id`, `model`, token/cache totals, `estimated_cost`, `billed_cost`, `currency`, `updated_at` | `UNIQUE(hour_start, provider_id, profile_id, session_id, model)` | `usage_hourly_rollups_time`, `usage_hourly_rollups_session` |

### 12.3 Repository And Query API Ownership

| API | Owner atom | Contract |
|-----|------------|----------|
| `load_settings_preferences` / `save_settings_preferences` | M06-F001 | Typed settings payload round-trips through `preferences.settings`; missing row returns defaults. |
| `upsert_history_entry` / `list_history_entries` / `mark_history_entry_tracked` | M05-F002 | Upsert by `(agent_kind, external_id)`, preserve transcript path/cwd/title metadata, list newest first, mark tracked by Homie session id. |
| `upsert_project` / `upsert_worktree` / `list_worktrees_for_project` | M07-F001/M07-F002 | Upsert repo/worktree facts without shelling out to git, enforce path/branch uniqueness, expose cleanup flags. |
| `create_session` / `update_session_status` / `update_session_core_metadata` / `list_sessions` | M17-F001 | Keep Diri core session fields in SQLite and expose stable summaries for downstream runtime/UI. |
| `record_usage` / `query_usage_totals` | M19-F001 | Deduplicate source events, store safe token/cost/latency facts, aggregate by optional time/session/provider/model filters. |
| `schema_inventory` | cross-cutting | Return table, column, index and unique constraint facts used by tests and dependent lane readiness checks. |

### 12.4 Verification Gates

| Gate | Covers | Evidence file |
|------|--------|---------------|
| `FC-STOR-001` schema inventory | Required tables, fields, indexes and unique constraints for M05/M06/M07/M17/M19 | `docs/verification/diri-storage-indexing/functional-case-results.md` |
| `FC-STOR-002` preferences API | M06 settings persistence and default behavior | `docs/verification/diri-storage-indexing/functional-case-results.md` |
| `FC-STOR-003` history API | M05 history upsert/list/mark tracked semantics | `docs/verification/diri-storage-indexing/functional-case-results.md` |
| `FC-STOR-004` project/worktree API | M07 repo/worktree uniqueness, flags and session linkage | `docs/verification/diri-storage-indexing/functional-case-results.md` |
| `FC-STOR-005` session core metadata | M17 Diri SessionRecord subset fields | `docs/verification/diri-storage-indexing/functional-case-results.md` |
| `FC-STOR-006` usage ledger API | M19 usage dedupe and aggregate query | `docs/verification/diri-storage-indexing/functional-case-results.md` |
| `FC-STOR-007` local quality gates | formatting, check, focused tests, diff/security gates | `docs/verification/diri-storage-indexing/release-readiness-report.md` |

### 12.5 Security Rules

- `preferences.value_json` may contain settings and local paths, but must not contain provider raw keys, virtual key material, Authorization headers, cookies, or pairing token plaintext.
- `usage_records` and rollups store counts, cost, value kind, source, safe error code and timestamps only. They must not store raw request/response bodies or prompts.
- `history_entries.metadata_json`, `sessions.needs_input_payload_json` and `session_artifacts.metadata_json` must stay safe-field payloads; full tool args/results belong outside storage or in redacted context events only.
- Schema changes are forward-only. If database schema is newer than `homie-storage` supports, `open_or_create` + `migrate` must fail closed through `SchemaTooNew`.

## 13. Diri 7ba3407 重基线修订

权威来源：

- PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`
- 能力矩阵: `docs/research/diri-7ba3407-capability-matrix.md`
- Requirements: FR-05, FR-13, FR-14, FR-16
- Beads: `homie-t3u`

### 13.1 Ownership 修订

- SQLite 是 durable facts 的唯一事实源，但不是 UI 的直接依赖。
- `homie-app` 不得打开 `Storage`；UI 写操作必须经过 client 和 owning service。
- runtime/service 按领域调用 repository API，不能向 client/UI 暴露 connection、SQL 或 transaction。
- output log、terminal bytes、browser image 和 checkpoint blob 不写 SQLite；SQLite 只保存路径、hash、offset、epoch 和 safe metadata。
- session live 状态必须由 runtime/holder 证据决定；storage row 只能作为恢复输入，不能单独证明 running。

### 13.2 重基线数据要求

除第 12 节表外，后续 wave 必须明确：

- runtime endpoint/instance/epoch 和 last event sequence 的恢复事实；
- attachment/output/checkpoint 的 offset/index；
- agent `EffectiveAgentConfig` immutable snapshot；
- MCP lineage、parent/child ownership 和 permission decision audit；
- remote node/account/checkpoint/lease metadata；
- updater feed/stage/install/rollback receipt；
- context/memory/task 的真实 repository，而非 crate-local model。

每组字段必须由对应 wave PRD 决定 migration；本重基线不预先增加空表。

### 13.3 完成门禁

- 每个 migration 有 empty DB、previous version、rollback-on-failure 和 schema-too-new 测试。
- service repository 有 transaction、uniqueness、concurrency 和 corrupt-state 测试。
- app crate dependency scan 证明不直接依赖 `homie-storage`。
- runtime restart、history resume、usage incremental scan、handoff 和 updater recovery 都从 repository 恢复并通过 E2E。
- raw key、Authorization、cookie、raw prompt、raw tool args/result 的 schema/fixture scan 为零。
