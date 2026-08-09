# diri-engine-migration Functional Verification Report

```yaml
change_id: diri-engine-migration
report_type: functional-verification
status: pass
beads: homie-cj5
functional_cases: docs/verification/diri-engine-migration/functional-cases.md
```

## Summary

All designed P0/P1 functional cases FC-DIRI-001 through FC-DIRI-018 passed on the current worktree. Several commands emitted non-fatal warnings before cleanup; final clippy gate passed after warning cleanup.

## Case Results

| Case | Command | Result | Evidence summary |
|------|---------|--------|------------------|
| FC-DIRI-001 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass | shell PTY output `homie-live-pty` was read from holder-produced runtime output |
| FC-DIRI-002 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass | invalid cwd failed and session list stayed empty |
| FC-DIRI-003 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass | missing holder input returned `SessionNotLive` and did not append fake output |
| FC-DIRI-004 | `cargo test -p homie-agents status_reducer -- --nocapture` | pass | 5 reducer tests passed: turn completion, anti-flicker, needs input, subagent isolation, process-only exit |
| FC-DIRI-005 | `cargo test -p homie-agents hook_parser -- --nocapture` | pass | 4 hook parser tests passed: Claude permission, subagent prompt, unknown fail-open, Codex turn complete |
| FC-DIRI-006 | `cargo test -p homie-term --test scrollback -- --nocapture` | pass | 4 scrollback tests passed: fetch/compose, row mismatch, alt screen reset, wheel route |
| FC-DIRI-007 | `cargo test -p homie-ui --test tokens -- --nocapture` | pass | 5 token tests passed: radius/metrics, typography, motion, status colors, semantic/fill/space/memory |
| FC-DIRI-008 | `cargo test -p homie-app --test app_shell_copy_regression -- --nocapture` and `cargo check -p homie-app` | pass | app source no longer contains roadmap placeholders; app compiles |
| FC-DIRI-009 | `rg -n "状态: ✅ 完成|Status: complete|⏭️" openspec/changes/diri-engine-migration docs/verification/diri-engine-migration` | pass | only matched the functional-case command text itself; no OpenSpec state drift found |
| FC-DIRI-010 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass | holder-owned PTY survives supervisor drop/reopen, explicit terminate cleans socket/pid and marks `exited`, holder status restores `exited`, missing holder evidence restores `detached` |
| FC-DIRI-011 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass | `runtime_status_report_uses_headless_screen_and_reducer` passed; real PTY output drove holder log -> headless screen -> screen observation -> `StatusReducer` for `running`, `needs_input`, `idle`, `exited` |
| FC-DIRI-012 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass | `runtime_terminate_kills_detached_child_tree` passed; holder process-tree termination kills a child that called `setsid` and ignored `SIGTERM` |
| FC-DIRI-013 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass | `runtime_holder_stat_tracks_resize_and_log_offsets` passed; holder `Stat` exposes geometry, epoch/log offsets, and resize updates via holder IPC |
| FC-DIRI-014 | `cargo test -p homie-cli --tests -- --nocapture`; `cargo run -p homie-cli -- hook/notify ...` | pass | 4 CLI tests passed; real `homie hook PermissionRequest` outputs redacted structured JSON, `homie notify` outputs Codex turn-complete JSON, unknown hook fail-open output redacts authorization |
| FC-DIRI-015 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass | `runtime_reopen_snapshot_combines_registry_holder_status_and_replay` passed; snapshot combines persisted session, holder stat, status report, and exact offset replay after supervisor reopen |
| FC-DIRI-016 | `cargo test -p homie-cli --test session_snapshot_cli -- --nocapture` | pass | `session_snapshot_command_reads_runtime_snapshot_json` passed; CLI `session snapshot` calls runtime snapshot and emits JSON status/output |
| FC-DIRI-017 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass | `runtime_screen_checkpoint_survives_supervisor_reopen` passed; checkpoint persists output offset, content seq, and headless screen lines across supervisor reopen |
| FC-DIRI-018 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass | `runtime_hibernate_stops_holder_and_wake_restarts_it` passed; hibernate stops holder and wake restarts a live interactive PTY |

## Workspace Gates

| Gate | Command | Result | Notes |
|------|---------|--------|-------|
| Format | `cargo fmt --all -- --check` | pass | initial check failed; `cargo fmt --all` applied formatting, final check passed |
| Tests | `cargo test --workspace` | pass | workspace tests passed |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | pass | final clippy run passed |
| Runtime holder/status/process-tree/stat/snapshot/checkpoint/lifecycle follow-up | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass | 12 focused runtime lifecycle/holder/status/process-tree/stat/snapshot/checkpoint/resource tests passed |
| Runtime/agents/proto/storage follow-up clippy | `cargo clippy -p homie-runtime -p homie-agents -p homie-proto -p homie-storage --all-targets -- -D warnings` | pass | focused clippy gate passed after holder status and runtime reducer pipeline changes |
| Storage regression | `cargo test -p homie-storage --tests -- --nocapture` | pass | 9 storage tests passed |
| CLI hook/notify follow-up | `cargo test -p homie-cli --tests -- --nocapture` | pass | 4 tests passed |
| CLI hook/notify lint | `cargo clippy -p homie-cli -p homie-agents --all-targets -- -D warnings` | pass | focused clippy gate passed |
| CLI session snapshot | `cargo test -p homie-cli --test session_snapshot_cli -- --nocapture` | pass | CLI snapshot integration test passed |

## Residual Warnings

- `cargo check -p homie-app` previously reported warnings before cleanup. Final clippy gate passed with `-D warnings`, so no warning-level Rust diagnostics remain in the clippy gate.

## Gate Decision

Decision: pass

Reason:

- Every designed functional case has a passing execution result.
- Workspace format, test, and clippy gates pass.
- The implementation still intentionally excludes Diri remote node, MCP, updater, full RootView/StoreRuntime, holder-manager/resource-governor crash matrix, and app/client/protocol live UI wiring; these remain outside this focused runtime follow-up scope.
