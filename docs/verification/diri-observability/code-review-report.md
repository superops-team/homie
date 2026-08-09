# Diri observability 第一阶段 Code Review Report

```yaml
change_id: diri-observability
beads: homie-wm7
report_type: code-review
status: pass
review_rounds: 2
updated_at: 2026-08-07
```

## 1. 审查范围

| 范围 | 文件 |
|------|------|
| PRD/spec/OpenSpec | `prd-spec/features/diri-observability/*`, `specs/observability/README.md`, `openspec/changes/diri-observability/*` |
| Evidence | `docs/verification/diri-observability/*` |
| Implementation | `crates/homie-observability/Cargo.toml`, `crates/homie-observability/src/lib.rs`, `crates/homie-observability/tests/*` |

参考规则：

- `AGENTS.md` Required Development Workflow。
- `docs/development/standards.md` 安全日志和 TDD 规范。
- `docs/development/quality-gates.md` evidence-first gate。
- `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md` observability review gap。

## 2. 第一轮：显性问题审查与修复

| 严重度 | 类别 | 位置 | 证据与影响 | 处置 |
|---|---|---|---|---|
| medium | Correctness | `crates/homie-observability/src/lib.rs` `is_dangerous_field` | `usage.input_tokens` 被 `token` 子串误判为 secret，导致合法 Diri usage summary 无法记录 | fixed: 明确 whitelist 优先，允许 `usage.input_tokens` 等摘要字段 |
| medium | Correctness | `crates/homie-observability/src/lib.rs` `MetricsWriteFailure::to_event` | `metrics.write_failed` 初版使用 `seq=0`，与 Diri `events.dropped` synthetic marker 的专用 seq 冲突 | fixed: `to_event(seq)` 接收调用方提供的正常 seq，`events.dropped` 独占 `seq=0` |
| medium | Security | `crates/homie-observability/src/lib.rs` `SafeFields::project` | 初版只检查顶层字段，允许 `evidence.output_summary` 对象内嵌 secret-bearing header key | fixed: 增加递归 nested key scan，并补 `safe_field_projection_blocks_dangerous_fields_inside_allowed_objects` |
| low | Dependency | `crates/homie-observability/Cargo.toml` | 直接依赖 `serde` 但未使用，增加无意义依赖面 | fixed: 移除直接 `serde` 依赖，仅保留 `serde_json` |
| low | Hygiene | `crates/homie-observability/tests/safe_fields.rs` | 测试占位使用 secret-shaped header text，容易触发 hook 或形成不良示例 | fixed: 改为 neutral placeholder value |

## 3. 第二轮：对抗式复盘

| 反例/边界 | 结果 |
|-----------|------|
| 允许字段值为 object，内部包含 secret-bearing key | 已覆盖并 fail-closed |
| `events.dropped` 被 session/kind filter 排除 | 测试证明仍可见 |
| project/worktree 事件进入 session filter | 测试证明被过滤 |
| `metrics.write_failed` 伪装成 `events.dropped` marker | 已修正 seq 语义 |
| negative token / NaN / Inf / negative cost | usage tests 覆盖拒绝 |
| `not_run` gate 被当成 `pass` | evidence tests 覆盖状态分离 |
| 代码越界触碰 runtime/LLM/storage | scoped git status 只显示允许范围内新增/修改 |

第二轮未发现新的 P0/P1 问题。

## 4. 修复摘要

- 增强 safe field projector：whitelist 优先、危险字段段级/compact 检测、嵌套 key 递归检测。
- 明确 Diri event model：`events.dropped` 使用 `seq=0`，普通事件包括 `metrics.write_failed` 使用正常 seq。
- 建立 usage evidence 非负/finite 校验。
- 建立 evidence gate status 和 functional case event。
- 去掉未使用依赖和疑似 secret 风格测试占位。

## 5. 验证结果

| 命令 | 结果 | 说明 |
|------|------|------|
| `cargo fmt --manifest-path crates/homie-observability/Cargo.toml -- --check` | pass | Rust formatting clean |
| `cargo check --manifest-path crates/homie-observability/Cargo.toml` | pass | Focused crate compiles |
| `cargo clippy --manifest-path crates/homie-observability/Cargo.toml --all-targets -- -D warnings` | pass | No warnings |
| `cargo test --manifest-path crates/homie-observability/Cargo.toml` | pass | 12 integration tests |
| `git diff --check -- prd-spec/features/diri-observability openspec/changes/diri-observability docs/verification/diri-observability specs/observability/README.md crates/homie-observability` | pass | Scoped diff whitespace clean |
| `.githooks/pre-commit` after staging scoped files | pass | Staged scoped content accepted |

## 6. 剩余风险

- New crate is intentionally not in root workspace; future integration must decide workspace registration and ownership.
- This code is a contract/model layer only; real runtime event bus, LLM metrics sink, storage persistence, CLI, and UI consumers are not implemented here.
