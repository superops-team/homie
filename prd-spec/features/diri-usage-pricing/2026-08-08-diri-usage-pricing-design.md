# Diri Usage Pricing Estimate 设计文档

```yaml
change_id: diri-usage-pricing
beads: homie-t3e
target_rows:
  - USAGE-001
feature_atoms:
  - M19-F001
```

## 1. 概述

### 1.1 问题/背景

Diri 的 `diri-usage` crate 提供 shared API-equivalent pricing，用于 local 和 node transcript fallback 的用量估算。Homie 当前已有 usage storage schema 和 `usage summary` CLI，但 `homie-llm` 尚未提供 Diri 对齐的 provider/model pricing helper，导致后续 transcript watcher、LLM proxy usage 写入、pricing snapshot 不能复用统一规则。

该能力不依赖截图或 UI，可通过纯 Rust 单元测试精确验证。

### 1.2 目标

- 在 `homie-llm` 中实现 Diri-compatible pricing estimate helper。
- 支持 Claude model matching 和 OpenAI model matching。
- 支持 cache read、Claude cache write 5m、Claude cache write 1h 的独立价格倍率。
- 输出 API-equivalent estimated cost，并明确这是估算值，不是 provider authoritative billing。

## 2. 用户场景

### 场景 1：Claude transcript fallback 估算成本

**Given** Claude transcript 中包含 model、input/output/cache tokens。  
**When** Homie 根据 Diri pricing 规则估算成本。  
**Then** 返回与 Diri `claude_estimate` 一致的 USD API-equivalent 结果。

### 场景 2：OpenAI/Codex usage 估算成本

**Given** OpenAI-compatible usage 中包含 model、input/output/cache read tokens。  
**When** Homie 根据 Diri pricing 规则估算成本。  
**Then** 返回与 Diri `openai_estimate` 一致的结果。

### 场景 3：未知模型

**Given** model 不匹配 Diri pricing 表。  
**When** Homie 调用 estimate helper。  
**Then** 返回 None，不制造错误价格。

## 3. 功能需求

### FR-1：Pricing model

提供 `ModelPricing`，包含 input/output 每百万 token 单价，并派生 cache read、cache write 5m、cache write 1h 单价。

### FR-2：Provider model matching

提供 `match_claude` 和 `match_openai`，匹配顺序必须与 Diri 保持一致。

### FR-3：Cost estimate

提供 `claude_estimate` 和 `openai_estimate`，对负 token 使用 `max(0)`，避免输入异常产生负成本。

### FR-4：测试对齐

测试必须覆盖 Claude cache write 5m/1h、OpenAI cache read、specific-to-generic model matching 和 unknown model。

## 4. 实现方案

### 4.1 模块位置

在 `crates/homie-llm/src/lib.rs` 中新增 pricing helper：

- `PRICING_ENTRY_COUNT`
- `ModelPricing`
- `match_claude`
- `match_openai`
- `openai_estimate`
- `claude_estimate`

### 4.2 测试策略

新增 `crates/homie-llm/tests/usage_pricing.rs`：

- Claude sonnet base input 1M = 3.0；
- OpenAI codex cache read 1M = 0.175；
- Claude cache write 5m/1h 使用 1.25x/2.0x；
- `opus-4-1` 优先于 generic `opus`；
- unknown model 返回 None；
- negative tokens 不减少成本。

## 5. 非目标

- 不实现 transcript watcher。
- 不写入 `homie-storage` usage records。
- 不实现 usage UI/fleet merge。
- 不声明 billed cost。

## 6. 涉及文件

- `crates/homie-llm/src/lib.rs`
- `crates/homie-llm/tests/usage_pricing.rs`
- `specs/llm-proxy/README.md`
- `docs/research/diri-parity-lock.md`
- `docs/verification/diri-usage-pricing/`

## 7. 验收标准

- `cargo test -p homie-llm --test usage_pricing`
- `cargo test -p homie-llm`
- `cargo check -p homie-llm`
- `cargo clippy -p homie-llm --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- scoped `git diff --check`
- `make parity-lock`

## 8. 受影响长期规格

- `specs/llm-proxy/README.md`：补充 Diri pricing estimate helper 是 usage/cost metrics 的本地估算源。

## 9. Beads 跟踪

- Bead: `homie-t3e`
- 验证完成后关闭。
