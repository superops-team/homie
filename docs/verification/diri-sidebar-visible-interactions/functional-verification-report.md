# Functional Verification Report: Diri Sidebar Visible Interactions

```yaml
change_id: diri-sidebar-visible-interactions
beads: homie-5r1
status: pass
validated_at: 2026-08-07
```

## Summary

This slice advances `UI-002` by wiring the existing `SidebarSessionModel` into `homie-app` and exposing visible Diri-style sidebar controls for pin, multi-select, and archive.

`UI-002` remains `partial`; hover cards, drag UI, and real pointer/screenshot E2E are still pending.

## TDD Evidence

RED:

- `cargo test -p homie-app --test app_shell_copy_regression -- --nocapture` failed because app sidebar did not consume `SidebarSessionModel`.

GREEN:

- Added `sidebar_model` to `AppState`.
- Added `sync_sidebar_model`, `pin_sidebar_session`, `archive_sidebar_session`, and `toggle_sidebar_multi_select`.
- Session rows now render visible `pin`/`select`/`archive` controls wired to click handlers.
- Archive only hides the local sidebar row and does not terminate the runtime session.

## Results

| Case | Command | Result |
|------|---------|--------|
| FC-DSVI-001 | `cargo test -p homie-app --test app_shell_copy_regression -- --nocapture` | pass |
| FC-DSVI-002 | `cargo test -p homie-ui --test workbench_state -- --nocapture` | pass |
| FC-DSVI-003 | `cargo check -p homie-app` | pass |
| FC-DSVI-004 | `cargo clippy -p homie-app --all-targets -- -D warnings` | pass |
| FC-DSVI-005 | scoped `git diff --check`; `make parity-lock` | pass / pass_with_remaining_gaps |
| LoopX | `loopx --registry .loopx/registry.json check --scan-root /Users/bytedance/workspace/github/homie` | pass |

## Remaining Gaps

- Hover card UI and screenshot evidence.
- Real drag reorder pointer E2E.
- Manual or automated GPUI click E2E for pin/select/archive.
- Runtime-backed archive semantics, if product later requires actual session lifecycle action.
