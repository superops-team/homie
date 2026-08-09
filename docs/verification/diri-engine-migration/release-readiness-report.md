# diri-engine-migration Release Readiness Report

```yaml
change_id: diri-engine-migration
report_type: release-readiness
status: pass
beads: homie-cj5
source_prd:
  - prd-spec/features/diri-engine-migration/2026-08-06-diri-engine-migration-design.md
  - prd-spec/features/diri-engine-migration/2026-08-06-diri-engine-migration-gap-closure-design.md
openspec:
  - openspec/changes/diri-engine-migration/plan.md
  - openspec/changes/diri-engine-migration/tasks.md
  - openspec/changes/diri-engine-migration/alignment-report.md
```

## Delivered Scope

- `homie-runtime` now starts a real local `/bin/sh` PTY session through `homie-runtime-holder`; the holder owns PTY/output log, `RuntimeSupervisor` reopens can adopt live holders, and missing/terminated holders restore as `detached`/`exited` instead of fake `running`.
- `homie-runtime` now supports offset-addressed output replay and headless screen-line/status projection from the holder-produced output log through `homie-agents::StatusReducer`.
- `homie-runtime` now terminates detached child processes through holder process-tree enumeration instead of only signaling the root shell process group.
- `homie-runtime` holder `Stat` now exposes geometry, tree size, epoch offset, and current log offset; `RuntimeSupervisor::resize_session` updates holder geometry through IPC.
- `homie-runtime` now exposes `session_snapshot`, combining persisted session metadata, holder stat, status report, and exact output replay after supervisor reopen.
- `homie-runtime` now persists and restores screen checkpoints containing output offset, content seq, and headless screen lines.
- `homie-runtime` `archive`/`hibernate` now stop holder processes, and `wake` restarts a holder-owned PTY before marking the session running.
- `homie-cli session snapshot` now exposes the runtime snapshot as JSON for CLI attach/resume smoke coverage.
- `homie-agents` now exposes status reducer and hook/notify parser APIs with stable events and structured redaction.
- `homie-cli hook` and `homie-cli notify` now call the same parsers and emit structured, redacted JSON instead of placeholder `{}`/empty output.
- `homie-term::scrollback` now has a real result model, geometry/cache behavior, row-count validation, alt-screen reset, and mode-aware wheel routing.
- `homie-ui` now includes Diri-aligned token coverage for typography, toolbar metrics, motion springs, semantic colors, fill, space, and memory formatting.
- `homie-app` now renders a Diri-style preview shell and no longer shows implementation-roadmap placeholder copy.
- OpenSpec, component specs, functional cases, and verification reports now reflect actual delivery state.

## Gate Results

| Gate | Command / Evidence | Status |
|------|--------------------|--------|
| Spec review | `docs/verification/diri-engine-migration/spec-review-report.md` | pass |
| Functional case design | `docs/verification/diri-engine-migration/functional-cases.md` | pass |
| OpenSpec alignment | `openspec/changes/diri-engine-migration/alignment-report.md` | pass |
| SDD/TDD task evidence | `docs/verification/diri-engine-migration/sdd-tdd-task-report.md` | pass |
| Functional verification | `docs/verification/diri-engine-migration/functional-verification-report.md` | pass |
| Code review | `docs/verification/diri-engine-migration/code-review-report.md` | pass |
| E2E | `docs/verification/diri-engine-migration/e2e-report.md` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Workspace tests | `cargo test --workspace` | pass |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| Runtime holder/status/process-tree/stat/snapshot/checkpoint/lifecycle follow-up | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass |
| Runtime/agents/proto/storage follow-up lint | `cargo clippy -p homie-runtime -p homie-agents -p homie-proto -p homie-storage --all-targets -- -D warnings` | pass |
| Storage regression | `cargo test -p homie-storage --tests -- --nocapture` | pass |
| CLI hook/notify follow-up | `cargo test -p homie-cli --tests -- --nocapture` and `cargo run -p homie-cli -- hook/notify ...` | pass |
| CLI session snapshot | `cargo test -p homie-cli --test session_snapshot_cli -- --nocapture` | pass |

## Residual Risks

| Risk | Status | Follow-up |
|------|--------|-----------|
| `homie-client`/protocol live UI wiring is not present | accepted | tracked by `docs/research/diri-parity-lock.md` API-002 |
| holder-owned PTY survival and detached child kill are implemented, but full holder-manager/resource-governor crash matrix is not | accepted | tracked by `docs/research/diri-parity-lock.md` RT-007/RT-009 |
| Diri remote node, MCP, updater, and full RootView/StoreRuntime parity remain incomplete | accepted | split follow-up Beads/OpenSpec before claiming full Diri product parity |
| Real GPUI screenshot/visual regression harness is absent | accepted | add screenshot harness before final UI fidelity release gate |

## Beads State

`homie-cj5` should remain open or in progress unless the project owner decides this local gap-closure slice is sufficient for that issue. This report does not claim full Diri parity; it claims the scoped local gap-closure items passed.

## Parity Lock

The authoritative remaining Diri parity checklist is `docs/research/diri-parity-lock.md`.
The command `make parity-lock` must continue to pass, and Homie must not be described as Diri-parity-complete while that command lists incomplete rows.

## Gate Decision

Decision: pass

Reason:

- All scoped P0/P1 functional cases pass.
- Workspace fmt/test/clippy gates pass.
- Remaining Diri parity gaps are explicitly documented as residual risks and are not reported as complete.
