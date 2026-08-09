# Diri Storage/Indexing Parity 第一阶段设计文档

```yaml
change_id: diri-storage-indexing
beads: homie-q7n
lane: lane-foundation-storage
status: ready_for_spec_review
source:
  - docs/research/diri-module-inventory.md
  - docs/verification/diri-module-inventory/spec-dependency-analysis.md
  - docs/verification/diri-module-inventory/bingo-component-spec-review-report.md
  - specs/storage-indexing/README.md
```

## 1. 概述

### 1.1 背景

`storage-indexing` 是 Diri parity 的 L0 foundation contract。当前长期规格已经声明 `homie-storage` 负责 SQLite、migration、关系约束、健康检查和 repository API，但审查报告指出仍缺少 M05/M06/M07/M17/M19 的表字段、唯一约束和查询 API inventory。若直接让 runtime、context、LLM 或 UI lane 继续实现，后续会反复修改 migration，导致 Diri 行为只在模块名上对齐，数据事实源并未对齐。

Diri 参考行为来自：

- M05 Navigation/History：历史扫描、transcript resume、title watcher。
- M06 Settings/Preferences：General/Terminal/Resources/Remote 偏好持久化。
- M07 Worktrees/Git：repo locate、worktree overview、create/remove/cleanup 的事实记录。
- M17 Core Models：SessionRecord、Project、status、needs input、attention、title source、resumability。
- M19 Usage Accounting：usage/pricing/token/cache/cost/latency 记录与查询。

### 1.2 目标

- 固化 `specs/storage-indexing/README.md` 中 M05/M06/M07/M17/M19 的表级 schema inventory、唯一约束、索引和 repository/query API ownership。
- 在 `crates/homie-storage` 内落地第一阶段最小实现：SQLite schema/migration 与公开 repository/query API，供后续模块调用。
- 增加 storage tests，覆盖 migration 幂等、唯一约束、history/worktree/preferences/session/usage 查询 API 和 Diri 对齐字段。
- 只实现 storage/indexing 基础，不实现 UI、runtime、git shell、history scanner、usage parser 或 remote sync。

### 1.3 非目标

- 不实现 Diri command palette、quick open UI、settings UI、worktree sheet、runtime session lifecycle 或 LLM proxy。
- 不扫描真实 `~/.claude`、`~/.codex`、git repo 或 transcript 文件。
- 不新增 Makefile/scripts，不修改其他 lane 文件。
- 不做向后兼容 migration/fallback；本仓库规则要求当前需求下直接前进。

## 2. 用户场景

### 场景 1: 后续 History lane 可以通过 storage API 固化可恢复会话

**Given** runtime/history scanner 已解析出 agent kind、agent session id、cwd、title、transcript path、last active time  
**When** 写入 `homie-storage` history repository  
**Then** 同一 `(agent_kind, external_id)` 不会重复，列表按最近活跃时间倒序返回，并可标记已被当前 session 跟踪。

### 场景 2: Settings lane 可以保存 Diri 设置页需要的偏好

**Given** 用户修改 General/Terminal/Resources/Remote 偏好  
**When** 调用 storage preference API 保存 `settings` JSON  
**Then** storage 返回同一结构；缺省值稳定；raw credential/token 不进入 SQLite 偏好表。

### 场景 3: Worktree lane 可以建立 project/worktree/session 关系

**Given** runtime 发现 repo root 和 worktree path  
**When** 调用 storage project/worktree API upsert 记录  
**Then** project root 与 worktree path 全局唯一，同一路径不会重复，worktree 可关联 session 并带 branch/detached/bare/prunable/dirty/merged/stale flags。

### 场景 4: Core model lane 可以保存 Diri SessionRecord 的第一阶段字段

**Given** session 已创建或更新  
**When** storage 写入核心 session metadata  
**Then** title source、agent session id、transcript path、needs input、resumability、parent、pinned、archived、remote active、host、foreground agent、memory bytes 等第一阶段字段有明确列或 JSON 承载位置。

### 场景 5: Usage lane 可以记录并查询 token/cost 基础账本

**Given** LLM proxy 或 transcript fallback 生成 usage event  
**When** storage 写入 usage record  
**Then** request id/source/value kind 唯一去重，token/cache/cost/latency 字段可按 session、provider、model 和时间范围查询聚合，且不保存 raw request/response。

## 3. 功能需求

### FR-001: 表级 inventory

`specs/storage-indexing/README.md` 必须列出每个 Diri 原子项归属的表、关键字段、唯一约束、索引、owner API 和验证 case。

### FR-002: Schema migration

`crates/homie-storage` 必须将第一阶段 schema 落到 SQLite migration，包含：

- M05：`history_entries`。
- M06：`preferences`。
- M07：`projects`、`worktrees`。
- M17：`sessions`、`context_events`、agent/runtime/profile 支撑表。
- M19：`model_pricing`、`pricing_snapshots`、`usage_records`、`usage_scan_files`、`usage_hourly_rollups`。

### FR-003: 唯一约束与关系约束

必须在 SQLite 层保证：

- `preferences.key` 唯一。
- `projects.root_path` 唯一。
- `worktrees.path` 唯一；`worktrees(project_id, branch)` 在 branch 非空时唯一。
- `history_entries(agent_kind, external_id)` 唯一。
- `model_pricing(provider_id, model, effective_at)` 唯一。
- `usage_records(provider_id, source, source_event_id)` 唯一；`usage_records.request_id` 不作为全局唯一，因为不同 provider 可能复用 request id。
- default enabled agent profile 只能有一个。

### FR-004: Repository/query API inventory 与最小实现

第一阶段公开 API 必须覆盖：

- preferences：读取/写入 `settings`。
- sessions：创建、列表、状态更新、Diri core metadata update。
- history：upsert、按最近活跃列表、mark tracked。
- project/worktree：upsert project、upsert worktree、按 project 列表。
- usage：record usage、按条件聚合 usage totals。
- schema inventory：查询表/列/索引/唯一约束，供测试和后续 lane 验证。

### FR-005: 安全边界

- raw provider key、Authorization、cookie、raw request/response、完整 tool args/result 不进入这些表。
- settings/remote 偏好只允许保存引用、布尔值、路径或安全配置，不保存 token 明文。
- usage 只保存 token/cost/latency、安全错误码、source/value kind，不保存 prompt/body。

## 4. 实现方案

### 4.1 长期规格更新

更新 `specs/storage-indexing/README.md`，增加：

- Diri feature atom 到表/API/验证的映射。
- 表字段 inventory。
- 唯一约束和查询索引 inventory。
- Repository/query API ownership。
- Phase 1 验证门禁。

### 4.2 Storage schema 与 repository

在 `crates/homie-storage/src/lib.rs` 中保持单 crate 最小实现：

- 将 schema version 前进到下一版本。
- 用 forward-only migration 增加缺失字段、索引和 usage scan/rollup 表。
- 继续使用 `rusqlite` 手写 SQL，不引入 ORM 或新依赖。
- 添加 typed request/summary structs，暴露最小公开 API。

### 4.3 测试策略

新增或更新 storage integration tests：

- 使用 `tempfile` 空 data dir，跑真实 `open_or_create` + `migrate`。
- 通过 public API 验证 behavior。
- 对唯一约束使用 SQLite 插入或 API 重复写入验证失败/去重。
- 对 schema inventory 使用 `PRAGMA table_info/index_list/index_info` 校验字段和索引。

## 5. 涉及文件

- `prd-spec/features/diri-storage-indexing/2026-08-07-diri-storage-indexing-design.md`
- `specs/storage-indexing/README.md`
- `docs/verification/diri-storage-indexing/*`
- `openspec/changes/diri-storage-indexing/*`
- `crates/homie-storage/src/lib.rs`
- `crates/homie-storage/tests/*`

## 6. 验收标准

- `specs/storage-indexing/README.md` 明确 M05/M06/M07/M17/M19 表字段、唯一约束、查询 API 和验证 case。
- OpenSpec plan/tasks/alignment 与本 PRD 逐项对应，无未映射 P0/P1 需求。
- `cargo test -p homie-storage` 通过。
- `cargo fmt --all -- --check` 通过或如存在其他 lane 影响，在 evidence 中如实标注。
- `cargo check -p homie-storage` 通过。
- `.githooks/pre-commit`、`git diff --check` 按质量门禁执行并记录结果；若失败，必须说明是否由本 lane 引入。

## 7. Beads 跟踪

- Beads: `homie-q7n`
- `change_id`: `diri-storage-indexing`
- Priority: P0
- Lane: `lane-foundation-storage`
- 关闭条件：release readiness report 证明 PRD/spec/OpenSpec/functional cases/storage tests/code review/evidence 均完成。
