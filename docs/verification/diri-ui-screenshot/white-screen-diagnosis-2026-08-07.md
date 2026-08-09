# Homie White Screen / Screenshot Diagnosis

```yaml
change_id: diri-ui-screenshot
status: partial
validated_at: 2026-08-07
```

## Summary

While refreshing UI evidence after `diri-sidebar-visible-interactions`, current macOS screenshot capture became unreliable:

- `cargo run -p homie-app` produced a WindowServer-visible `homie-app` window.
- A minimal `HOMIE_RENDER_PROBE=1` root view executed `load()` and `render()`.
- `screencapture -l <window-id>` failed with `could not create image from window` for both debug and packaged Homie windows.
- Full-screen captures were black or unrelated to Homie because the active frontmost process remained Feishu and WindowServer reported a high-level `Display 1 Shield`.

Therefore the new screenshots captured during this diagnosis are rejected as evidence. Existing accepted screenshots remain the latest valid visual evidence.

## Code Fixes Kept

Two code fixes were retained because they address real first-frame and runtime safety risks discovered during diagnosis:

1. `homie-runtime` holder IPC now sets read/write timeouts before reading a holder response.
2. `homie-app` no longer calls `sync_terminal_geometry` from `Render::render`; resize is triggered during load/session selection instead, so rendering remains side-effect-light.

The temporary render debug logs and render probe branch were removed.

## Verification

| Command | Result |
|---------|--------|
| `cargo check -p homie-app` | pass |
| `cargo test -p homie-app --test app_shell_copy_regression -- --nocapture` | pass |
| `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass |
| `cargo clippy -p homie-app -p homie-runtime --all-targets -- -D warnings` | pass |

## Rejected Artifacts

These files are not accepted as parity evidence:

- `docs/verification/diri-ui-screenshot/homie-window-2026-08-07-current.png`
- `docs/verification/diri-ui-screenshot/homie-window-2026-08-07-sidebar-visible.png`
- `docs/verification/diri-ui-screenshot/homie-window-2026-08-07-render-fix.png`
- `docs/verification/diri-ui-screenshot/homie-window-2026-08-07-probe.png`
- `docs/verification/diri-ui-screenshot/homie-window-2026-08-07-probe2.png`
- `docs/verification/diri-ui-screenshot/homie-window-2026-08-07-coregraphics.png`
- `docs/verification/diri-ui-screenshot/homie-packaged-window-2026-08-07-current.png`
- `docs/verification/diri-ui-screenshot/homie-workbench-2026-08-07-current.png`
- `docs/verification/diri-ui-screenshot/homie-workbench-2026-08-07-probe.png`
- `docs/verification/diri-ui-screenshot/homie-workbench-2026-08-07-probe2.png`
- `docs/verification/diri-ui-screenshot/homie-packaged-workbench-2026-08-07-current.png`

## Remaining Blocker

Refresh screenshot evidence only after the local macOS capture environment can bring Homie frontmost and `screencapture -l <window-id>` succeeds. Until then, `UI-001..UI-009` remain partial.
