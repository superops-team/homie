# Diri Usage Transcript Storage Import 设计文档

```yaml
change_id: diri-usage-transcript-import
beads: homie-dh8
target_rows:
  - USAGE-001
feature_atoms:
  - M19-F001
```

## 1. 概述

### 1.1 问题/背景

Homie 已完成 Diri-compatible pricing helper、Claude/Codex transcript parser 和 usage storage ledger，但 parser 输出还不能直接导入 `homie-storage`。Diri node 的 usage ledger 会将 transcript events 导入 usage table，并依赖 event id 去重。Homie 需要一个最小 importer，把 `homie-llm::TranscriptUsageEvent` 映射成 `homie_storage::RecordUsage` 并调用 `Storage::record_usage`。

该切片不依赖截图，不做 watcher 或 UI，只完成 parser -> storage 的可测桥接。

### 1.2 目标

- 将 `TranscriptUsageEvent` 映射为 `RecordUsage`。
- 复用 storage 层 `(provider_id, source, source_event_id)` 去重。
- 保留 estimated cost、token/cache 字段和 transcript source。
- 支持批量导入并返回 inserted/skipped 统计。

## 2. 用户场景

### 场景 1：导入 parsed Claude/Codex usage events

**Given** transcript parser 已输出 neutral usage events。  
**When** Homie importer 将 events 导入 storage。  
**Then** `usage summary` 能查询到 token 和 estimated cost totals。

### 场景 2：重复导入

**Given** 相同 transcript event 再次导入。  
**When** Homie importer 调用 storage。  
**Then** storage 去重生效，重复 event 被 skipped，不重复累加。

## 3. 功能需求

### FR-1：Mapping contract

Importer 必须将 `TranscriptUsageEvent` 映射为 `RecordUsage`，包括：

- request_id/source_event_id 使用 event.source_event_id；
- provider_id 使用 `provider_<provider>`；
- runtime_id/agent_profile_id/llm_profile_id 可由 caller 提供默认 metadata；
- value_kind/source 来自 event；
- estimated_cost 使用 decimal string。

### FR-2：Batch result

Importer 返回 `UsageImportResult { inserted, skipped }`。

### FR-3：Storage dedupe

重复导入同一 source_event_id 时不重复累计。

## 4. 实现方案

在 `homie-storage` 增加：

- `UsageImportDefaults`
- `UsageImportResult`
- `Storage::record_transcript_usage_event`
- `Storage::record_transcript_usage_events`

不引入新表，不改变 schema。

## 5. 非目标

- 不实现目录 watcher。
- 不实现 offset cache。
- 不实现 pricing snapshot persistence。
- 不实现 usage UI/fleet merge。

## 6. 涉及文件

- `crates/homie-storage/Cargo.toml`
- `crates/homie-storage/src/lib.rs`
- `crates/homie-storage/tests/usage_transcript_import.rs`
- `docs/research/diri-parity-lock.md`
- `docs/verification/diri-usage-transcript-import/`

## 7. 验收标准

- `cargo test -p homie-storage --test usage_transcript_import -- --nocapture`
- `cargo test -p homie-storage --test diri_storage_indexing`
- `cargo check -p homie-storage`
- `cargo clippy -p homie-storage --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- scoped `git diff --check`
- `make parity-lock`

## 8. Beads 跟踪

- Bead: `homie-dh8`
- 完成验证后关闭。
