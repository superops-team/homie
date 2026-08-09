# Diri observability 第一阶段 SDD/TDD Task Report

```yaml
change_id: diri-observability
beads: homie-wm7
report_type: sdd-tdd
status: pass
source_tasks: openspec/changes/diri-observability/tasks.md
updated_at: 2026-08-07
```

## 1. SDD 边界

本阶段严格按 `specs/observability/README.md` 和 `openspec/changes/diri-observability/tasks.md` 实现，只新增独立纯模型 crate `crates/homie-observability`。没有修改 runtime、LLM proxy、storage、client、CLI 或 UI lane 文件。

## 2. Task 执行记录

| Task | RED | GREEN | Refactor | Evidence |
|------|-----|-------|----------|----------|
| T-001 component spec hardening | `spec-review-report.md` 记录缺少 whitelist/EventBus/metrics/usage/evidence 合同 | `specs/observability/README.md` 增加第一阶段合同 | 保持对其他 specs 只读 | FC-OBS-006 |
| T-002 safe field whitelist | `SafeFields`/`SafeFieldError` 未实现导致 `safe_fields` 测试编译失败；后续 nested dangerous field 测试先失败 | 实现 allowlist projector、危险字段检测、嵌套危险 key fail-closed | `input_tokens` 不再被误判为 secret token；补 `rawPrompt/toolArgs` 变体检测 | FC-OBS-001 |
| T-003 EventBus envelope/filter/drop marker | `EventName`/`SafeEvent`/`EventFilter` 未实现导致 `events` 测试编译失败 | 实现 Diri event catalog、event envelope、session/kind filter、`events.dropped` `seq=0` | 明确 `events.dropped` 是唯一 `seq=0` synthetic marker | FC-OBS-002 |
| T-004 metrics write failure | `MetricsWriteFailure` 未实现导致 `metrics` 测试编译失败 | 实现 `metrics.write_failed` safe event projection | `metrics.write_failed` 改为调用方提供正常 seq，避免和 `events.dropped` 混淆 | FC-OBS-003 |
| T-005 usage evidence | `UsageEvidence`/`UsageSource`/`UsageValueKind` 未实现导致 `usage` 测试编译失败 | 实现 Diri-aligned usage summary projection、negative token 和 invalid cost rejection | 不复制 Diri pricing tables，pricing 保持后续 usage lane 范围 | FC-OBS-004 |
| T-006 evidence model | `GateStatus`/`CommandEvidence` 未实现导致 `evidence` 测试编译失败 | 实现 gate status vocabulary 和 `verification.functional_case_executed` event | 保持 `not_run`/`blocked` 与 `pass` 明确分离 | FC-OBS-005 |
| T-007 verification/review/release evidence | 初始报告不存在 | 运行 focused gates、pre-commit、两轮 review 并记录报告 | 清理 `crates/homie-observability/target` 构建产物 | FC-OBS-001..006 |

## 3. 测试先行证据

| Slice | RED 结果 | GREEN 结果 |
|-------|----------|------------|
| safe fields | unresolved imports；嵌套危险字段测试先失败 | `cargo test --manifest-path crates/homie-observability/Cargo.toml safe_field -- --nocapture` pass |
| events | unresolved imports | `cargo test --manifest-path crates/homie-observability/Cargo.toml event -- --nocapture` pass |
| metrics | unresolved import `MetricsWriteFailure` | `cargo test --manifest-path crates/homie-observability/Cargo.toml metrics -- --nocapture` pass |
| usage | unresolved imports for usage types | `cargo test --manifest-path crates/homie-observability/Cargo.toml usage -- --nocapture` pass |
| evidence | unresolved imports for evidence types | `cargo test --manifest-path crates/homie-observability/Cargo.toml evidence -- --nocapture` pass |

## 4. Final Gate Snapshot

| Command | Result |
|---------|--------|
| `cargo fmt --manifest-path crates/homie-observability/Cargo.toml -- --check` | pass |
| `cargo check --manifest-path crates/homie-observability/Cargo.toml` | pass |
| `cargo clippy --manifest-path crates/homie-observability/Cargo.toml --all-targets -- -D warnings` | pass |
| `cargo test --manifest-path crates/homie-observability/Cargo.toml` | pass: 12 integration tests |
| `git diff --check -- prd-spec/features/diri-observability openspec/changes/diri-observability docs/verification/diri-observability specs/observability/README.md crates/homie-observability` | pass |
| `.githooks/pre-commit` after staging scoped files | pass |

## 5. Residual Scope

- `crates/homie-observability` is intentionally independent and verified with `--manifest-path`; root workspace registration is deferred to a later integration PRD/OpenSpec.
- Runtime EventBus, LLM metrics sink, storage repository, CLI commands, and UI display are not implemented in this phase.
