# Diri Observability/Event/Evidence Parity Phase 1 OpenSpec Tasks

> Change ID: `diri-observability`
> Beads: `homie-wm7`
> Source PRD: `prd-spec/features/diri-observability/2026-08-07-diri-observability-design.md`
> Functional cases: `docs/verification/diri-observability/functional-cases.md`

## Task Status

| Status | Meaning |
|--------|---------|
| todo | Not started |
| red | Failing test or contract written |
| green | Minimal implementation passes focused verification |
| refactor | Cleanup while tests stay green |
| done | Task evidence recorded and accepted |

## Task To Functional Case Mapping

| OpenSpec task | PRD requirement | Functional cases |
|---------------|-----------------|------------------|
| T-001 | FR-6 | FC-OBS-006 |
| T-002 | FR-1 | FC-OBS-001 |
| T-003 | FR-2 | FC-OBS-002 |
| T-004 | FR-3 | FC-OBS-003 |
| T-005 | FR-4 | FC-OBS-004 |
| T-006 | FR-5 | FC-OBS-005 |
| T-007 | FR-6 | FC-OBS-001 through FC-OBS-006 |

## Tasks

### T-001: Observability component spec hardening

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-6 |
| Component spec | `specs/observability/README.md` |
| Files | `specs/observability/README.md`, `docs/verification/diri-observability/*`, `openspec/changes/diri-observability/*` |

Objective:

- Make the durable observability spec precise enough for later runtime/LLM/client lanes to consume without inventing their own event or redaction rules.

RED:

- Document review identifies missing safe field whitelist, event catalog, metrics write failure contract, usage projection, and phase-1 verification gates.

GREEN:

- Update `specs/observability/README.md` with:
  - Diri EventBus parity contract.
  - Safe field whitelist.
  - Metrics write failure contract.
  - Usage evidence projection.
  - Phase 1 verification gates.

Acceptance:

- FC-OBS-006 passes.
- `alignment-report.md` maps every FR to at least one task and functional case.

Evidence:

- `docs/verification/diri-observability/spec-review-report.md`
- `docs/verification/diri-observability/functional-verification-report.md#fc-obs-006`

### T-002: Safe field whitelist model

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-1 |
| Component spec | `specs/observability/README.md` |
| Files | `crates/homie-observability/src/lib.rs`, `crates/homie-observability/tests/safe_fields.rs` |

Objective:

- Provide an allowlist-first field projector that strips unknown and dangerous fields before data enters logs/events/metrics/evidence.

RED:

- Add tests showing safe fields are retained while `authorization`, `cookie`, `raw_prompt`, `raw_request`, `raw_response`, `tool_args`, `tool_result`, `env`, and unknown fields are not emitted.

GREEN:

- Implement `SafeFields`, whitelist lookups, dangerous field detection, and projection errors.

Refactor:

- Keep API string-key based for phase 1; do not add macros or schema generation until consumers exist.

Acceptance:

- FC-OBS-001 passes.

Evidence:

- `docs/verification/diri-observability/sdd-tdd-task-report.md#t-002`
- `docs/verification/diri-observability/functional-verification-report.md#fc-obs-001`

### T-003: Diri EventBus envelope/filter/drop marker model

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-2 |
| Component spec | `specs/observability/README.md` |
| Files | `crates/homie-observability/src/lib.rs`, `crates/homie-observability/tests/events.rs` |

Objective:

- Model the Diri event envelope and filtering semantics so later runtime/client event buses have a stable contract.

RED:

- Add tests for known event names, event projection through safe fields, session filter, kind filter, combined filter, and `events.dropped` visibility.

GREEN:

- Implement `EventName`, `SafeEvent`, `EventFilter`, `EventsDropped`, and `visible_to`.

Refactor:

- Keep event catalog explicit and finite; do not accept arbitrary event names silently.

Acceptance:

- FC-OBS-002 passes.

Evidence:

- `docs/verification/diri-observability/sdd-tdd-task-report.md#t-003`
- `docs/verification/diri-observability/functional-verification-report.md#fc-obs-002`

### T-004: Metrics write failure projection

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-3 |
| Component spec | `specs/observability/README.md` |
| Files | `crates/homie-observability/src/lib.rs`, `crates/homie-observability/tests/metrics.rs` |

Objective:

- Encode the contract that metrics persistence failure reports are safe and do not change the already-completed business outcome.

RED:

- Add tests where a simulated successful business result is paired with a metrics sink failure containing unsafe details.

GREEN:

- Implement `MetricsWriteFailure` and `to_event`, keeping only `metrics.kind`, `metrics.scope`, `component`, `operation`, `safe_error_code`, `retryable`, and `occurred_at`.

Refactor:

- Keep raw error messages out of the API; downstream must map errors to safe codes before reporting.

Acceptance:

- FC-OBS-003 passes.

Evidence:

- `docs/verification/diri-observability/sdd-tdd-task-report.md#t-004`
- `docs/verification/diri-observability/functional-verification-report.md#fc-obs-003`

### T-005: Usage evidence projection

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-4 |
| Component spec | `specs/observability/README.md` |
| Files | `crates/homie-observability/src/lib.rs`, `crates/homie-observability/tests/usage.rs` |

Objective:

- Provide a safe summary model for Diri-aligned usage evidence.

RED:

- Add tests for valid Claude/Codex-like usage projection, negative token rejection, non-finite or negative cost rejection, and unsafe field stripping.

GREEN:

- Implement `UsageEvidence`, `UsageValueKind`, `UsageSource`, projection validation, and safe field emission.

Refactor:

- Do not copy Diri's pricing tables into this crate; pricing remains LLM/usage lane scope.

Acceptance:

- FC-OBS-004 passes.

Evidence:

- `docs/verification/diri-observability/sdd-tdd-task-report.md#t-005`
- `docs/verification/diri-observability/functional-verification-report.md#fc-obs-004`

### T-006: Functional evidence model

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-5 |
| Component spec | `specs/observability/README.md` |
| Files | `crates/homie-observability/src/lib.rs`, `crates/homie-observability/tests/evidence.rs` |

Objective:

- Keep dev-loop/release gate evidence honest and machine-checkable.

RED:

- Add tests proving `not_run` and `blocked` remain distinct from `pass`, and functional case execution emits a safe `verification.functional_case_executed` event.

GREEN:

- Implement `GateStatus`, `CommandEvidence`, and functional case event projection.

Refactor:

- Keep output summaries short strings and route extra data through safe fields only.

Acceptance:

- FC-OBS-005 passes.

Evidence:

- `docs/verification/diri-observability/sdd-tdd-task-report.md#t-006`
- `docs/verification/diri-observability/functional-verification-report.md#fc-obs-005`

### T-007: Verification, review, and release evidence

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-6 |
| Component spec | `specs/observability/README.md` |
| Files | `docs/verification/diri-observability/*` |

Objective:

- Execute the designed functional cases, run local gates, perform two code-review passes, and record release readiness.

RED:

- Verification report starts with every case as `not_run`.

GREEN:

- Run focused commands:
  - `cargo fmt --manifest-path crates/homie-observability/Cargo.toml -- --check`
  - `cargo check --manifest-path crates/homie-observability/Cargo.toml`
  - `cargo clippy --manifest-path crates/homie-observability/Cargo.toml --all-targets -- -D warnings`
  - `cargo test --manifest-path crates/homie-observability/Cargo.toml`
  - `git diff --check`

Refactor:

- Fix only issues in allowed scope.

Acceptance:

- FC-OBS-001 through FC-OBS-006 results are recorded.
- Code review report includes two passes and any fixes.
- Release readiness is not `pass` if any required P0 case fails.

Evidence:

- `docs/verification/diri-observability/functional-verification-report.md`
- `docs/verification/diri-observability/code-review-report.md`
- `docs/verification/diri-observability/release-readiness-report.md`
