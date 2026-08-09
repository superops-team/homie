# OpenSpec Plan: Homie App First-frame Runtime Blocking

> Change ID: `homie-app-first-frame-runtime-blocking`  
> Beads: `homie-7tb`

## Scope

Fix runtime I/O hazards discovered while diagnosing Homie screenshot refresh. The app must not perform holder I/O in `Render::render`, and holder IPC must not wait forever.

## Modules

| Module | Change |
|--------|--------|
| `crates/homie-runtime/src/holder.rs` | Add read/write timeout to holder socket request |
| `crates/homie-app/src/main.rs` | Move terminal geometry resize out of render path |
| `docs/verification/diri-ui-screenshot/` | Record rejected screenshot artifacts and diagnosis |

## Functional Cases

| Case | Command |
|------|---------|
| FC-HAFRB-001 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` |
| FC-HAFRB-002 | `cargo test -p homie-app --test app_shell_copy_regression -- --nocapture` |
| FC-HAFRB-003 | `cargo check -p homie-app` |
| FC-HAFRB-004 | `cargo clippy -p homie-app -p homie-runtime --all-targets -- -D warnings` |
| FC-HAFRB-005 | `make ui-screenshot-gate`; `make parity-lock`; `loopx --registry .loopx/registry.json check --scan-root /Users/bytedance/workspace/github/homie` |

## Acceptance

- Holder IPC has bounded read/write time.
- `Render::render` does not call `sync_terminal_geometry`.
- Rejected screenshots are explicitly documented and not used as evidence.
