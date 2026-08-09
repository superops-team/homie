# Diri observability 第一阶段 Release Readiness Report

```yaml
change_id: diri-observability
beads: homie-wm7
report_type: release-readiness
status: pass_with_scope_limit
updated_at: 2026-08-07
```

## 1. Source Links

| Item | Path |
|------|------|
| PRD | `prd-spec/features/diri-observability/2026-08-07-diri-observability-design.md` |
| Component spec | `specs/observability/README.md` |
| OpenSpec plan | `openspec/changes/diri-observability/plan.md` |
| OpenSpec tasks | `openspec/changes/diri-observability/tasks.md` |
| Alignment report | `openspec/changes/diri-observability/alignment-report.md` |
| Functional cases | `docs/verification/diri-observability/functional-cases.md` |
| Functional verification | `docs/verification/diri-observability/functional-verification-report.md` |
| SDD/TDD report | `docs/verification/diri-observability/sdd-tdd-task-report.md` |
| Code review | `docs/verification/diri-observability/code-review-report.md` |
| E2E report | `docs/verification/diri-observability/e2e-report.md` |

## 2. Risk Tier

Tier: Tier 3 high-stakes foundation contract.

Reason:

- Touches logs/events/metrics/evidence and anti-leak policy.
- Establishes a contract that later runtime, LLM proxy, usage, MCP, and UI lanes will consume.
- Does not touch production runtime/LLM/storage code in this phase.

## 3. Gate Results

| Gate | Command | Exit code | Status | Notes |
|------|---------|-----------|--------|-------|
| Spec Gate | Review PRD/spec/OpenSpec/evidence mapping | 0 | pass | `spec-review-report.md`, `alignment-report.md` |
| Functional Case Gate | FC-OBS-001 through FC-OBS-006 | 0 | pass | `functional-verification-report.md` |
| Format Gate | `cargo fmt --manifest-path crates/homie-observability/Cargo.toml -- --check` | 0 | pass | Focused crate formatting clean |
| Build Gate | `cargo check --manifest-path crates/homie-observability/Cargo.toml` | 0 | pass | Focused crate compiles |
| Lint Gate | `cargo clippy --manifest-path crates/homie-observability/Cargo.toml --all-targets -- -D warnings` | 0 | pass | No warnings |
| Unit/Integration Gate | `cargo test --manifest-path crates/homie-observability/Cargo.toml` | 0 | pass | 12 integration tests |
| Whitespace Gate | `git diff --check -- prd-spec/features/diri-observability openspec/changes/diri-observability docs/verification/diri-observability specs/observability/README.md crates/homie-observability` | 0 | pass | Scoped diff clean |
| Security Gate | `.githooks/pre-commit` after staging scoped files | 0 | pass | Staged scoped files passed hook |

## 4. Not Run Gates

| Gate | Status | Reason |
|------|--------|--------|
| `cargo test --workspace` | not_run | Repository already has many unrelated dirty/untracked lane files; this change intentionally verifies independent crate via `--manifest-path` |
| Runtime/app E2E | not_run | Runtime/client/app integration is out of scope |
| LLM proxy E2E | not_run | LLM proxy metrics sink integration is out of scope |
| Storage migration tests | not_run | No storage schema/repository change |
| UI screenshot gate | not_run | No UI change |

## 5. New Dependencies

| Dependency | Reason | Scope |
|------------|--------|-------|
| `serde_json` | Structured safe field and event payload model | Local to `crates/homie-observability` |

No network/process/filesystem/secret capability dependency was added.

## 6. Delivered State

- `specs/observability/README.md` now defines safe field whitelist, Diri event catalog/filter/drop marker semantics, `metrics.write_failed`, usage evidence projection, and FC-OBS validation gates.
- `crates/homie-observability` provides pure model APIs:
  - `SafeFields`
  - `EventName`, `SafeEvent`, `EventFilter`
  - `MetricsWriteFailure`
  - `UsageEvidence`, `UsageSource`, `UsageValueKind`
  - `GateStatus`, `CommandEvidence`
- Tests cover safe field stripping, nested dangerous keys, EventBus parity, metrics failure projection, usage projection validation, and evidence status honesty.

## 7. Residual Risk

- The new crate is not registered in the root workspace by design; future integration must decide root workspace ownership.
- Later runtime, LLM proxy, storage, CLI, MCP, and UI lanes still need to consume this contract through their own PRD/OpenSpec changes.
- This report does not close broader Diri observability parity beyond the phase-1 foundation model and specs.

## 8. Decision

Decision: pass_with_scope_limit

Reason:

- All P0/P1 functional cases for this phase passed.
- Focused Rust gates and security hook passed.
- Out-of-scope E2E paths are honestly marked `not_run`, not `pass`.
