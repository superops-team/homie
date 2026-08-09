# Diri Workbench Runtime Wiring Functional Verification Report

```yaml
change_id: diri-workbench-runtime-wiring
beads: homie-3tz
status: pass
validated_at: 2026-08-07
```

## Summary

This slice advances `UI-001` and `UI-003` from a single live-shell preview toward real workbench runtime wiring:

- `AppState` now stores runtime session rows from `HomieClient::list_sessions`.
- command palette `SpawnShell` dispatches to `spawn_runtime_shell`, which calls `HomieClient::spawn_shell`.
- sidebar session rows render from runtime session projection and use a click handler to select a real session.
- terminal refresh reads `HomieClient::session_snapshot` for the selected session.
- terminal geometry is synchronized through `HomieClient::resize_session`.

`UI-001` and `UI-003` remain `partial` because full GPUI interaction E2E and screenshot evidence are still pending.

## Functional Cases

| Case | Command | Result | Evidence |
|------|---------|--------|----------|
| FC-DWRW-001 | `cargo test -p homie-app --tests -- --nocapture` | pass | App regression confirms `SpawnShell` dispatches to runtime spawn helper, not local notice placeholder |
| FC-DWRW-002 | `cargo test -p homie-app --tests -- --nocapture` | pass | App regression confirms sidebar rows use `HomieClient::list_sessions` and click selection |
| FC-DWRW-003 | `cargo test -p homie-app --tests -- --nocapture` | pass | App regression confirms terminal geometry calls `HomieClient::resize_session` |
| FC-DWRW-004 | `cargo clippy -p homie-app --all-targets -- -D warnings` | pass | touched app crate passes clippy |
| FC-DWRW-005 | `cargo test -p homie-client --tests -- --nocapture` | pass | client runtime/session/transport behavior remains intact |

## Gate Decision

Decision: pass

Reason:

- The app no longer treats workbench spawn/select/resize as local-only UI notices.
- Runtime ownership remains behind `homie-client`; `homie-app` does not directly manage PTY/runtime/storage lifecycle.
- UI parity completion still requires future screenshot and real interaction E2E gates.

