# Diri Usage Transcript Parser 设计文档

```yaml
change_id: diri-usage-transcript-parser
beads: homie-hd1
target_rows:
  - USAGE-001
  - AG-004
feature_atoms:
  - M19-F001
  - M19-F002
```

## 1. 概述

### 1.1 问题/背景

Diri 的 usage 能力从 Claude/Codex transcript 中解析 token usage，并在本地/远端 usage ledger 中生成 estimated API-equivalent usage event。Homie 已完成 storage usage schema、usage summary CLI、Diri-compatible pricing helper，但尚缺 Claude/Codex transcript parser，导致后续 watcher/storage import 还没有稳定输入模型。

该切片不依赖截图、UI 或真实 provider。目标是先在 `homie-llm` 中实现纯解析函数，将 Claude/Codex JSONL 行转成 neutral `TranscriptUsageEvent`，供后续 storage importer 使用。

### 1.2 目标

- 解析 Claude assistant message usage。
- 解析 Codex token_count event，并保留 turn_context model。
- 生成稳定 transcript event id。
- 使用现有 Diri-compatible pricing helper 计算 estimated cost。
- 忽略坏 JSON、非 usage 行、未知模型 cost。

## 2. 用户场景

### 场景 1：Claude transcript 导入

**Given** Claude JSONL 中有 assistant message、timestamp、message.id、requestId、usage。  
**When** Homie 解析 transcript。  
**Then** 生成 provider=`claude` 的 usage event，包含 input/output/cache tokens、session id、model 和 estimated cost。

### 场景 2：Codex transcript 导入

**Given** Codex JSONL 中先出现 session_meta/turn_context model，再出现 event_msg token_count。  
**When** Homie 解析 transcript。  
**Then** 生成 provider=`codex` 的 usage event，包含 input/output/cache read tokens 和 estimated cost。

### 场景 3：异常输入

**Given** transcript 包含坏 JSON、非 usage 行、未知模型或负 token。  
**When** Homie 解析 transcript。  
**Then** 坏行被跳过，未知模型保留 usage 但 estimated_cost 为 None，负 token 被 clamp 到 0。

## 3. 功能需求

### FR-1：Neutral event model

提供 `TranscriptUsageEvent`，字段覆盖 provider、session_id、model、tokens、estimated_cost、source_event_id、occurred_at。

### FR-2：Claude parser

解析 Diri Claude usage 字段：

- `usage.input_tokens`
- `usage.output_tokens`
- `usage.cache_read_input_tokens`
- `usage.cache_creation_input_tokens`
- `usage.cache_creation.ephemeral_5m_input_tokens`
- `usage.cache_creation.ephemeral_1h_input_tokens`

### FR-3：Codex parser

解析 Codex `payload.type=token_count` 和 `payload.info.last_token_usage`，并从 `session_meta` 或 `turn_context` 保留 model。

### FR-4：Timestamp and id

支持 RFC3339 timestamp 和整数 Unix seconds；source event id 必须由 path、line offset 和 provider message id 组成，保证同一文件/offset 稳定。

### FR-5：范围诚实

本阶段不扫描目录、不维护 offset cache、不写 storage。

## 4. 实现方案

### 4.1 模块位置

在 `crates/homie-llm/src/lib.rs` 增加：

- `UsageProviderKind`
- `UsageValueKind`
- `UsageSourceKind`
- `TranscriptUsageEvent`
- `parse_transcript_usage_events(path, provider, profile_id)`

### 4.2 测试策略

新增 `crates/homie-llm/tests/usage_transcript_parser.rs`：

- Claude fixture 解析 token/cache/cost；
- Codex fixture 解析 model/cache read/cost；
- unknown/bad/negative safety；
- source_event_id 稳定性。

## 5. 非目标

- 不实现目录 watcher。
- 不写 `homie-storage::RecordUsage`。
- 不实现 usage UI/fleet merge。
- 不实现 offset cache。

## 6. 涉及文件

- `crates/homie-llm/src/lib.rs`
- `crates/homie-llm/tests/usage_transcript_parser.rs`
- `specs/llm-proxy/README.md`
- `docs/research/diri-parity-lock.md`
- `docs/verification/diri-usage-transcript-parser/`

## 7. 验收标准

- `cargo test -p homie-llm --test usage_transcript_parser -- --nocapture`
- `cargo test -p homie-llm`
- `cargo check -p homie-llm`
- `cargo clippy -p homie-llm --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- scoped `git diff --check`
- `make parity-lock`

## 8. 受影响长期规格

- `specs/llm-proxy/README.md`：补充 transcript parser 是 usage/cost metrics 的本地导入前置。

## 9. Beads 跟踪

- Bead: `homie-hd1`
- 完成验证后关闭。
