# Code Review Report: Homie App First-frame Runtime Blocking

```yaml
change_id: homie-app-first-frame-runtime-blocking
beads: homie-7tb
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| medium | Correctness | `crates/homie-runtime/src/holder.rs` | `holder::request` read from a Unix socket without timeout. If a holder accepted a connection but stopped responding, UI/runtime callers could block indefinitely. | fixed: added 350ms read/write timeouts. |
| medium | Architecture | `crates/homie-app/src/main.rs` | `Render::render` called `sync_terminal_geometry`, which performs runtime holder I/O. Rendering should be side-effect-light and must not block first-frame paint. | fixed: resize is triggered from load/session selection. |
| low | Verification | `docs/verification/diri-ui-screenshot` | New screenshot attempts were black/white or uncapturable in the current macOS environment. Treating them as evidence would hide a parity gap. | fixed: rejected artifacts recorded in diagnosis report. |

## Brooks Review

Mode: PR Review  
Scope: app first-frame path and runtime holder IPC  
Health Score: 90/100

No critical issue remains in the reviewed code. The retained changes reduce blocking risk and keep UI evidence honest. The remaining risk is environmental: screenshot refresh still needs a working macOS capture session before UI rows can be promoted.

## Verification

| Command | Result |
|---------|--------|
| `cargo check -p homie-app` | pass |
| `cargo test -p homie-app --test app_shell_copy_regression -- --nocapture` | pass |
| `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass |
| `cargo clippy -p homie-app -p homie-runtime --all-targets -- -D warnings` | pass |
| `make ui-screenshot-gate` | pass against accepted baseline |
| `make parity-lock` | pass_with_remaining_gaps |
