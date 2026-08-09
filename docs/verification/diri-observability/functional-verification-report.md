# Diri observability 第一阶段功能验证执行报告

```yaml
change_id: diri-observability
beads: homie-wm7
report_type: functional-verification
status: pass
source_cases: docs/verification/diri-observability/functional-cases.md
executed_at: 2026-08-07
```

## 1. 执行环境

| 字段 | 值 |
|------|----|
| cwd | `/Users/bytedance/workspace/github/homie` |
| Rust crate | `crates/homie-observability` |
| 验证方式 | 独立 `--manifest-path` focused tests |
| 运行边界 | 未启动 runtime/LLM/storage/client/CLI |

## 2. Case 结果

### FC-OBS-001: Safe field whitelist

| 字段 | 结果 |
|------|------|
| Status | pass |
| Command | `cargo test --manifest-path crates/homie-observability/Cargo.toml safe_field -- --nocapture` |
| Exit code | 0 |
| Actual output summary | `tests/safe_fields.rs` 3 tests passed: allowlisted fields retained, unknown fields dropped, top-level and nested dangerous fields fail closed |
| Evidence | `crates/homie-observability/tests/safe_fields.rs` |

### FC-OBS-002: Diri EventBus envelope/filter/drop marker

| 字段 | 结果 |
|------|------|
| Status | pass |
| Command | `cargo test --manifest-path crates/homie-observability/Cargo.toml event -- --nocapture` |
| Exit code | 0 |
| Actual output summary | `tests/events.rs` 3 tests passed: Diri wire names, session/kind filter semantics, `events.dropped` `seq=0` and always visible |
| Evidence | `crates/homie-observability/tests/events.rs` |

### FC-OBS-003: metrics.write_failed safe projection

| 字段 | 结果 |
|------|------|
| Status | pass |
| Command | `cargo test --manifest-path crates/homie-observability/Cargo.toml metrics -- --nocapture` |
| Exit code | 0 |
| Actual output summary | `tests/metrics.rs` 1 test passed: business result remains success while `metrics.write_failed` event exposes only safe fields and normal caller-provided seq |
| Evidence | `crates/homie-observability/tests/metrics.rs` |

### FC-OBS-004: Usage evidence projection

| 字段 | 结果 |
|------|------|
| Status | pass |
| Command | `cargo test --manifest-path crates/homie-observability/Cargo.toml usage -- --nocapture` |
| Exit code | 0 |
| Actual output summary | `tests/usage.rs` 3 tests passed: Diri-aligned usage summary fields retained, raw prompt/tool result absent, negative token and invalid cost rejected |
| Evidence | `crates/homie-observability/tests/usage.rs` |

### FC-OBS-005: Evidence helper gate status honesty

| 字段 | 结果 |
|------|------|
| Status | pass |
| Command | `cargo test --manifest-path crates/homie-observability/Cargo.toml evidence -- --nocapture` |
| Exit code | 0 |
| Actual output summary | `tests/evidence.rs` 2 tests passed: `not_run`/`blocked` remain distinct from `pass`; functional case event includes safe evidence fields |
| Evidence | `crates/homie-observability/tests/evidence.rs` |

### FC-OBS-006: PRD/spec/OpenSpec/evidence traceability

| 字段 | 结果 |
|------|------|
| Status | pass |
| Command | `test -f prd-spec/features/diri-observability/2026-08-07-diri-observability-design.md && test -f docs/verification/diri-observability/spec-review-report.md && test -f docs/verification/diri-observability/functional-cases.md && test -f openspec/changes/diri-observability/plan.md && test -f openspec/changes/diri-observability/tasks.md && test -f openspec/changes/diri-observability/alignment-report.md` |
| Exit code | 0 |
| Actual output summary | Required PRD, verification, and OpenSpec files exist |
| Evidence | `openspec/changes/diri-observability/alignment-report.md` |

## 3. Aggregate Gates

| Command | Exit code | Result | Summary |
|---------|-----------|--------|---------|
| `cargo fmt --manifest-path crates/homie-observability/Cargo.toml -- --check` | 0 | pass | Formatting clean |
| `cargo check --manifest-path crates/homie-observability/Cargo.toml` | 0 | pass | Crate compiles |
| `cargo clippy --manifest-path crates/homie-observability/Cargo.toml --all-targets -- -D warnings` | 0 | pass | No clippy warnings |
| `cargo test --manifest-path crates/homie-observability/Cargo.toml` | 0 | pass | 12 integration tests passed |
| `git diff --check -- prd-spec/features/diri-observability openspec/changes/diri-observability docs/verification/diri-observability specs/observability/README.md crates/homie-observability` | 0 | pass | No whitespace errors in scoped diff |
| `.githooks/pre-commit` after staging scoped files | 0 | pass | Secret/path hook accepted staged scoped content |

## 4. Failure And Fix Record

| Finding during verification | Fix |
|-----------------------------|-----|
| `usage.input_tokens` was initially misclassified as dangerous because `token` substring matched broadly | Dangerous-field detection now allows explicit whitelist fields before matching secret patterns |
| `metrics.write_failed` initially used `seq=0` | Changed to caller-provided normal seq; `seq=0` remains reserved for `events.dropped` |
| Allowed object fields could contain nested dangerous keys | Added recursive nested key scan and regression test |
| `serde` direct dependency was unused | Removed direct `serde` dependency; `serde_json` remains the only dependency |

## 5. Residual Risk

- This phase does not prove runtime/client event streaming, LLM metrics sink, storage persistence, CLI output, or UI usage rendering. Those are deliberately out of scope and must consume this contract in later lane PRDs.
- `crates/homie-observability` is not registered in the root workspace by design for this worker scope; future integration must decide root workspace ownership.
