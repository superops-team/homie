# diri-engine-migration E2E Report

```yaml
change_id: diri-engine-migration
report_type: e2e
status: pass
beads: homie-cj5
```

## Scope

This E2E pass validates the local Diri gap-closure slice that can be proven in the current repository without enabling full app/client/runtime protocol wiring:

- Runtime holder-owned live PTY shell path.
- Holder adoption, explicit terminate cleanup, exited restore, and detached recovery.
- Runtime headless status report from output log through screen observation and status reducer.
- Runtime process-tree termination for detached child processes.
- Holder stat metadata, geometry resize, and epoch/log offset reporting.
- Runtime attach snapshot combining registry, holder stat, status report, and offset replay.
- Runtime screen checkpoint persistence across supervisor reopen.
- Runtime hibernate/wake resource lifecycle with real holder stop/restart.
- CLI session snapshot command that reads runtime snapshot JSON.
- CLI hook/notify parser entrypoints with redacted structured JSON output.
- Agent status reducer and hook parser.
- Terminal scrollback model.
- UI token parity.
- App preview shell copy and compile smoke.
- Workspace-wide regression tests.

## Executed End-To-End Gates

| Gate | Command | Result | Notes |
|------|---------|--------|-------|
| Runtime local PTY + holder/status/process-tree/stat/snapshot/checkpoint/lifecycle restore | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass | 12 tests cover live shell output, offset replay, screen lines, spawn failure cleanup, non-live input fail-closed, holder adoption, terminate cleanup, exited restore, detached recovery, screen reducer status report, detached child process-tree termination, resize, holder log offsets, reopen snapshot composition, checkpoint restore, and hibernate/wake holder lifecycle |
| CLI session snapshot | `cargo test -p homie-cli --test session_snapshot_cli -- --nocapture` | pass | covers `homie session snapshot` JSON output through runtime snapshot |
| Agent status/hooks | `cargo test -p homie-agents --tests -- --nocapture` | pass | covers reducer, hook parser, catalog |
| CLI hook/notify | `cargo test -p homie-cli --tests -- --nocapture`; `cargo run -p homie-cli -- hook/notify ...` | pass | covers Claude permission hook, Codex notify, unknown hook fail-open, and secret redaction |
| Terminal scrollback | `cargo test -p homie-term --test scrollback -- --nocapture` | pass | covers fetch, row mismatch, alt screen, wheel routing |
| UI token parity | `cargo test -p homie-ui --test tokens -- --nocapture` | pass | covers Diri token values and semantic helpers |
| App preview shell | `cargo test -p homie-app --test app_shell_copy_regression -- --nocapture` | pass | verifies placeholder copy is absent |
| App compile smoke | `cargo check -p homie-app` | pass | verifies GPUI app compiles after preview-shell change |
| Workspace regression | `cargo test --workspace` | pass | all workspace tests pass |
| Clippy gate | `cargo clippy --workspace --all-targets -- -D warnings` | pass | no warning-level clippy issues |
| Runtime/agents/proto/storage follow-up clippy | `cargo clippy -p homie-runtime -p homie-agents -p homie-proto -p homie-storage --all-targets -- -D warnings` | pass | focused gate after holder status/cleanup and runtime reducer pipeline follow-up |
| Storage regression | `cargo test -p homie-storage --tests -- --nocapture` | pass | 9 tests pass |

## Not Run

| Gate | Status | Reason |
|------|--------|--------|
| Real GPUI screenshot comparison | not_run | No repository screenshot harness exists yet; PRD allowed source-text and compile smoke as the minimum UI gate for this iteration |
| Live Homie app session creation from UI | not_run | `homie-client`/protocol live UI wiring is outside this gap-closure scope; app remains preview-only for live operations |
| Remote node/MCP/updater E2E | not_run | Explicit PRD non-goals for this local gap-closure iteration |
| Full holder-manager/resource-governor crash matrix | not_run | Minimal holder-owned PTY adoption and detached child kill are verified; Diri-level holder manager/resource governor crash matrix remains a later runtime parity task |

## Gate Decision

Decision: pass

Reason:

- All executable P0/P1 functional cases passed.
- Workspace fmt/test/clippy gates passed.
- Unrun gates are explicitly outside scope and documented as residual risks rather than counted as complete.
