# Functional Cases: Diri Storage/Indexing Parity Phase 1

```yaml
change_id: diri-storage-indexing
beads: homie-q7n
risk_tier: Tier 3 high-stakes SQLite migration/repository
status: designed_before_implementation
```

## 1. 执行原则

- 所有 case 必须运行开发完成后的真实 `homie-storage` 代码路径。
- 不使用真实用户 HOME、真实 provider key、真实 transcript 或真实 git repo。
- 测试数据使用 `tempfile` 空目录和显式 fixture row。
- 每个 case 的结果写入 `docs/verification/diri-storage-indexing/functional-case-results.md`。

## 2. Case 清单

### FC-STOR-001: Schema inventory 固化 M05/M06/M07/M17/M19 表字段与约束

前置环境：

- 本地 Rust toolchain 可用。
- 从 repo root 执行。

命令：

```bash
cargo test -p homie-storage storage_schema_inventory_covers_diri_phase_one -- --nocapture
```

输入数据：

- `tempfile` 空 data dir。
- `open_or_create` + `migrate` 创建 SQLite。

预期结果：

- schema version 为本阶段目标版本。
- `preferences/projects/worktrees/sessions/history_entries/model_pricing/pricing_snapshots/usage_records/usage_scan_files/usage_hourly_rollups` 存在。
- `sessions` 包含 Diri SessionRecord 第一阶段字段。
- `history_entries(agent_kind, external_id)`、`worktrees(path)`、`worktrees(project_id, branch)`、`usage_records(provider_id, source, source_event_id)`、`model_pricing(provider_id, model, effective_at)` 等唯一约束/索引可通过 SQLite metadata 证明。

通过标准：

- 命令退出码 0。
- 测试断言不通过时不得进入 final verification。

证据路径：

- `docs/verification/diri-storage-indexing/functional-case-results.md`

失败处理：

- 回到 OpenSpec task T2/T3 修正 schema 或 test inventory，再重新执行。

### FC-STOR-002: Preferences API 保存和读取 Diri settings payload

命令：

```bash
cargo test -p homie-storage settings_preferences_round_trip_through_preferences_table -- --nocapture
```

输入数据：

- `SettingsPreferences { startup_behavior, terminal_font_size, hibernate_idle_minutes, remote_companion_access }`

预期结果：

- 缺失 `settings` preference 时返回默认值。
- 保存后 typed API 返回同一结构。
- `preferences.key='settings'` 唯一，重复保存为 update。

通过标准：

- 命令退出码 0。

证据路径：

- `docs/verification/diri-storage-indexing/functional-case-results.md`

失败处理：

- 回到 T4 修正 preferences API。

### FC-STOR-003: History API upsert/list/mark tracked

命令：

```bash
cargo test -p homie-storage history_repository_upserts_lists_and_tracks_entries -- --nocapture
```

输入数据：

- 两条 history entry：Codex、Claude，各含 `external_id`、cwd、title、title_source、transcript_path、last_active_at、cwd_exists。
- 对同一 `(agent_kind, external_id)` 进行第二次 upsert。

预期结果：

- 重复 upsert 不新增重复行，只更新 title/metadata/last_active。
- `list_history_entries` 按 `last_active_at DESC, id ASC` 返回。
- `mark_history_entry_tracked` 能把 history entry 关联到已存在 session。

通过标准：

- 命令退出码 0。

证据路径：

- `docs/verification/diri-storage-indexing/functional-case-results.md`

失败处理：

- 回到 T5 修正 history API 或唯一约束。

### FC-STOR-004: Project/Worktree API 保证 repo/worktree 唯一性和 session linkage

命令：

```bash
cargo test -p homie-storage project_worktree_repository_enforces_identity_and_lists_by_project -- --nocapture
```

输入数据：

- 一个 project root。
- 两个 worktree path，其中一个带 branch、head、bare/detached/prunable/dirty/merged/stale flags。
- 重复 upsert 同一路径和同一 `(project_id, branch)`。

预期结果：

- project root 全局唯一，重复 upsert 返回同一 project。
- worktree path 全局唯一。
- 同一 project 下非空 branch 唯一。
- worktree 可关联 session 并按 project 查询。

通过标准：

- 命令退出码 0。

证据路径：

- `docs/verification/diri-storage-indexing/functional-case-results.md`

失败处理：

- 回到 T6 修正 project/worktree API 或索引。

### FC-STOR-005: Session core metadata 保存 Diri SessionRecord 第一阶段字段

命令：

```bash
cargo test -p homie-storage session_core_metadata_round_trips_diri_record_subset -- --nocapture
```

输入数据：

- `create_session` 创建的 session。
- `SessionCoreMetadataUpdate` 设置 project/worktree/git/title source/agent session/transcript/needs input/resumability/parent/pin/archive/remote/host/foreground agent/memory bytes。

预期结果：

- `session_by_id` 或 summary API 返回更新后的核心 metadata。
- 布尔字段保存为 SQLite CHECK 约束下的 0/1。
- parent session id 必须引用存在的 session。

通过标准：

- 命令退出码 0。

证据路径：

- `docs/verification/diri-storage-indexing/functional-case-results.md`

失败处理：

- 回到 T7 修正 sessions schema/API。

### FC-STOR-006: Usage ledger API 去重并聚合 token/cache/cost

命令：

```bash
cargo test -p homie-storage usage_repository_deduplicates_source_events_and_queries_totals -- --nocapture
```

输入数据：

- provider/LLM/profile/session fixture。
- 两条 usage record，包含 source/value_kind/source_event_id、input/output/cache read/cache write/cache write 5m/1h、reasoning、cost、latency。
- 重复写入同一 `(provider_id, source, source_event_id)`。

预期结果：

- 重复 source event 不新增重复记录。
- `query_usage_totals` 可按 session/provider/model/time filter 聚合 totals。
- 结果包含事件数、token/cache/cost、authoritative billing availability。
- 不需要 raw prompt/request/response 字段。

通过标准：

- 命令退出码 0。

证据路径：

- `docs/verification/diri-storage-indexing/functional-case-results.md`

失败处理：

- 回到 T8 修正 usage schema/API。

### FC-STOR-007: Local quality gates

命令：

```bash
cargo fmt --all -- --check
cargo check -p homie-storage
cargo test -p homie-storage
git diff --check
.githooks/pre-commit
```

预期结果：

- `homie-storage` focused gates 通过。
- 若 workspace 或 hook 因其他 lane 文件失败，必须在 `release-readiness-report.md` 标注失败命令、退出码、失败来源和是否由本 lane 引入。

证据路径：

- `docs/verification/diri-storage-indexing/release-readiness-report.md`

失败处理：

- 本 lane 引入的失败必须修复后重跑。
- 非本 lane 阻塞不得冒充通过。

## 3. 覆盖矩阵

| PRD Requirement | Functional Case | OpenSpec Task |
|-----------------|-----------------|---------------|
| FR-001 表级 inventory | FC-STOR-001 | T1, T2 |
| FR-002 Schema migration | FC-STOR-001, FC-STOR-007 | T2, T3 |
| FR-003 唯一约束与关系约束 | FC-STOR-001, FC-STOR-003, FC-STOR-004, FC-STOR-006 | T3, T5, T6, T8 |
| FR-004 Repository/query API inventory 与最小实现 | FC-STOR-002..006 | T4, T5, T6, T7, T8 |
| FR-005 安全边界 | FC-STOR-001, FC-STOR-006, FC-STOR-007 | T2, T8, T10 |

## 4. 执行顺序

1. FC-STOR-001：先证明 schema 基础。
2. FC-STOR-002：preferences 最小 API。
3. FC-STOR-003：history API。
4. FC-STOR-004：project/worktree API。
5. FC-STOR-005：session core metadata。
6. FC-STOR-006：usage ledger。
7. FC-STOR-007：质量门禁与 release readiness。
