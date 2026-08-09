# Diri Sidebar Session Model Functional Verification Report

```yaml
change_id: diri-sidebar-session-model
beads: homie-f02
status: pass
validated_at: 2026-08-07
```

## Summary

This slice advances `UI-002`:

- `homie-ui` now has a tested `SidebarSessionModel`.
- The model supports selection, multi-select, rename, pin/archive, drag-order helpers, and status glyph names.
- Existing app sidebar continues to render real runtime session rows.

`UI-002` remains `partial` until hover cards, drag UI, and screenshot/manual E2E are complete.

## Functional Cases

| Case | Command | Result |
|------|---------|--------|
| FC-DSSM-001 | `cargo test -p homie-ui --tests -- --nocapture` | pass |
| FC-DSSM-002 | `cargo clippy -p homie-ui --all-targets -- -D warnings` | pass |

