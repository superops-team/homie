# diri-engine-migration Gap Closure OpenSpec Tasks

> Change ID: `diri-engine-migration`  
> Source PRD: `prd-spec/features/diri-engine-migration/2026-08-06-diri-engine-migration-gap-closure-design.md`  
> Functional cases: `docs/verification/diri-engine-migration/functional-cases.md`  
> Beads: `homie-cj5`

## Task Status

| Status | Meaning |
|--------|---------|
| todo | Not started |
| red | Failing test or contract written |
| green | Implementation passes focused verification |
| refactor | Cleanup while tests stay green |
| done | Task evidence recorded and accepted |

## Task To Functional Case Mapping

| OpenSpec task | PRD requirement | Functional cases |
|---------------|-----------------|------------------|
| T-001 | FR-7 | FC-DIRI-009 |
| T-002 | FR-1 | FC-DIRI-001, FC-DIRI-002, FC-DIRI-003, FC-DIRI-010, FC-DIRI-011, FC-DIRI-012, FC-DIRI-013, FC-DIRI-015, FC-DIRI-016, FC-DIRI-017, FC-DIRI-018 |
| T-003 | FR-2, FR-3 | FC-DIRI-004, FC-DIRI-005, FC-DIRI-011, FC-DIRI-014 |
| T-004 | FR-4 | FC-DIRI-006 |
| T-005 | FR-5 | FC-DIRI-007 |
| T-006 | FR-6 | FC-DIRI-008 |
| T-007 | FR-7 | FC-DIRI-001 through FC-DIRI-018 |

## Tasks

### T-001: OpenSpec and component spec state correction

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-7 |
| Component spec | `specs/runtime-supervisor/README.md`, `specs/agent-adapter-contract/README.md`, `specs/desktop-shell/README.md`, `specs/session-context-store/README.md`, `specs/observability/README.md` |
| Beads | `homie-cj5` |
| Files | `openspec/changes/diri-engine-migration/*`, `docs/verification/diri-engine-migration/*`, affected `specs/*/README.md` |

Objective:

- Correct the previous "complete" state drift and make all future implementation work traceable to PRD, functional cases, and evidence.

RED:

- Add a document gate that fails if `diri-engine-migration` still claims complete status while known deferred items remain.

GREEN:

- Update plan/tasks/alignment and affected component specs with gap-closure scope and exact evidence paths.

Acceptance:

- FC-DIRI-009 passes.
- OpenSpec plan status is `in_progress` until implementation and verification finish.
- No unowned P0/P1 requirement remains.

Evidence:

- `docs/verification/diri-engine-migration/openspec-alignment-report.md`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-009`

### T-002: Runtime live PTY supervisor

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-1 |
| Component spec | `specs/runtime-supervisor/README.md`, `specs/session-context-store/README.md` |
| Beads | `homie-cj5` |
| Files | `crates/homie-runtime/src/*`, `crates/homie-runtime/tests/*` |

Objective:

- Replace the file-append fake session path in `RuntimeSupervisor` with real live PTY session ownership for local shell sessions.

RED:

- Add tests that prove current behavior is insufficient:
  - shell output must come from a live PTY.
  - spawn failure must not persist a half-created session.
  - `send_text` must fail closed for non-live sessions.

GREEN:

- Introduce a live session registry backed by holder IPC.
- Validate binary/cwd before persistence.
- Start `homie-runtime-holder` on successful spawn; the holder owns the PTY and output log.
- Route `send_text` to the holder-owned PTY writer.
- Keep `read_output` and `read_output_range` backed by offset-addressed output logs produced by the holder.
- `session_status_report` reads the holder-produced output log, builds a headless screen, classifies screen observations, and feeds `homie-agents::StatusReducer` for `running`/`needs_input`/`idle`/`exited` projection.
- Reopen adopts live holders, restores normal holder exits as `exited`, and marks missing holder evidence as `detached`.
- `terminate_session` sends a holder terminate request, waits for socket/pid cleanup, and persists `exited`.
- Holder termination now uses process-tree enumeration and group signaling to clean detached child processes, not only the root shell process group.
- Holder `Stat` exposes geometry, epoch/log offsets, and tree size; `RuntimeSupervisor::resize_session` updates holder geometry through IPC.
- `session_snapshot` combines persisted session metadata, holder stat, status report, and offset-addressed replay for future client/protocol attach.
- `homie session snapshot` calls `RuntimeSupervisor::session_snapshot` and emits JSON for CLI attach/resume smoke coverage.
- `write_screen_checkpoint` persists output offset, content seq, and headless screen lines; `read_screen_checkpoint` restores them after supervisor reopen.
- `archive`/`hibernate` stop holder processes, and `wake` restarts holder-owned PTY sessions before marking them running.

Refactor:

- Keep runtime APIs small and avoid UI/storage leakage.

Acceptance:

- FC-DIRI-001, FC-DIRI-002, FC-DIRI-003, FC-DIRI-010, FC-DIRI-011, FC-DIRI-012, FC-DIRI-013, FC-DIRI-015, FC-DIRI-016, FC-DIRI-017, and FC-DIRI-018 pass.

Evidence:

- `docs/verification/diri-engine-migration/sdd-tdd-task-report.md#t-002`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-001`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-002`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-003`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-010`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-011`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-012`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-013`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-015`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-016`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-017`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-018`

### T-003: Agent status reducer and hook parser

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-2, FR-3 |
| Component spec | `specs/agent-adapter-contract/README.md`, `specs/observability/README.md` |
| Beads | `homie-cj5` |
| Files | `crates/homie-agents/src/status.rs`, `crates/homie-agents/src/hooks.rs`, `crates/homie-agents/src/lib.rs`, `crates/homie-agents/tests/*` |

Objective:

- Port Diri's status reducer and hook/notify parsing into Homie with stable events and secret redaction.

RED:

- Add reducer tests for hooks-primary, screen-primary, process-only, idle confirmation, needs-input, process exit, and subagent isolation.
- Add hook parser tests for Claude and Codex payloads, unknown fail-open behavior, and secret redaction.

GREEN:

- Implement `StatusReducer`, `StatusSignal`, `ReducerOutcome`, `HookEvent`, `NotifyEvent`, parser functions, and redaction boundaries.
- Wire runtime screen observations into `StatusReducer` through `RuntimeSupervisor::session_status_report`.
- Wire `homie-cli hook` and `homie-cli notify` to the parser so automation entrypoints emit structured, redacted JSON instead of empty placeholders.

Refactor:

- Keep parsing pure and independent from runtime pump.

Acceptance:

- FC-DIRI-004, FC-DIRI-005, FC-DIRI-011, and FC-DIRI-014 pass.

Evidence:

- `docs/verification/diri-engine-migration/sdd-tdd-task-report.md#t-003`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-004`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-005`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-011`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-014`

### T-004: Terminal scrollback real model

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-4 |
| Component spec | `specs/desktop-shell/README.md`, `specs/runtime-supervisor/README.md` |
| Beads | `homie-cj5` |
| Files | `crates/homie-term/src/scrollback.rs`, `crates/homie-term/tests/*` |

Objective:

- Replace scrollback stub types with a real viewport, fetch result, cache, geometry, and wheel routing model.

RED:

- Add tests for request generation, fetch completion, row count mismatch, cached row lookup, alt screen reset, and wheel routing.

GREEN:

- Implement `ReadScrollbackCellsResult`, request range calculation, cache apply, geometry max offset, alt-screen reset, and mouse/alt screen aware wheel route.

Refactor:

- Preserve a small codec trait only where needed for future RLE integration.

Acceptance:

- FC-DIRI-006 passes.

Evidence:

- `docs/verification/diri-engine-migration/sdd-tdd-task-report.md#t-004`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-006`

### T-005: Diri design token parity

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-5 |
| Component spec | `specs/desktop-shell/README.md` |
| Beads | `homie-cj5` |
| Files | `crates/homie-ui/src/lib.rs`, `crates/homie-ui/tests/*` |

Objective:

- Complete Homie's design token layer so app surfaces can consume Diri-aligned constants instead of hard-coded dimensions and colors.

RED:

- Add token parity tests for Diri radius, typography, metrics, motion, semantic colors, fill, space, and memory formatting.

GREEN:

- Add missing token structs/enums/functions to `homie-ui`.

Refactor:

- Avoid introducing visual behavior into token-only APIs.

Acceptance:

- FC-DIRI-007 passes.

Evidence:

- `docs/verification/diri-engine-migration/sdd-tdd-task-report.md#t-005`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-007`

### T-006: Diri-style app preview shell without placeholders

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-6 |
| Component spec | `specs/desktop-shell/README.md` |
| Beads | `homie-cj5` |
| Files | `crates/homie-app/src/main.rs`, `crates/homie-app/src/palette.rs`, `crates/homie-app/tests/*` |

Objective:

- Remove implementation roadmap copy from the app and present a Diri-style sidebar + terminal/workbench + inspector preview shell.

RED:

- Add source text regression tests for forbidden placeholder copy.
- Add command palette ranking/action tests if missing.

GREEN:

- Replace placeholder cards and copy with preview shell surfaces driven by existing storage/runtime health and terminal buffer.
- Do not open live session UI operations until client/protocol wiring exists.

Refactor:

- Consume `homie-ui` tokens instead of adding new hard-coded design constants where practical.

Acceptance:

- FC-DIRI-008 passes.

Evidence:

- `docs/verification/diri-engine-migration/sdd-tdd-task-report.md#t-006`
- `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-008`

### T-007: Functional verification, review, and release readiness

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-7 |
| Component spec | all impacted specs |
| Beads | `homie-cj5` |
| Files | `docs/verification/diri-engine-migration/*` |

Objective:

- Execute all functional cases, run local gates, perform two review passes, and record final readiness without overstating unverified Diri parity.

RED:

- Treat any missing functional case result as blocked.

GREEN:

- Execute FC-DIRI-001 through FC-DIRI-018.
- Run required Rust checks.
- Produce SDD/TDD task report, functional verification report, code review reports, E2E report, and release readiness report.

Acceptance:

- All P0/P1 functional cases pass or are explicitly blocked with reason.
- No report marks an unverified feature complete.

Evidence:

- `docs/verification/diri-engine-migration/sdd-tdd-task-report.md`
- `docs/verification/diri-engine-migration/functional-verification-report.md`
- `docs/verification/diri-engine-migration/code-review-report.md`
- `docs/verification/diri-engine-migration/e2e-report.md`
- `docs/verification/diri-engine-migration/release-readiness-report.md`
