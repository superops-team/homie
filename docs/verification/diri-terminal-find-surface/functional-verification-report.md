# Diri Terminal Find Surface Functional Verification Report

```yaml
change_id: diri-terminal-find-surface
beads: homie-42v
status: pass
validated_at: 2026-08-07
```

## Summary

This slice advances `UI-003` and `TERM-004`:

- `homie-app` now has an app-visible Find surface.
- The Find surface is opened from command palette through `OpenFind`.
- Query input updates `TerminalFindModel`, applies the current terminal buffer snapshot, and syncs highlights to `TerminalElement`.

Full terminal interaction E2E remains pending.

## Functional Cases

| Case | Command | Result |
|------|---------|--------|
| FC-DTFS-001 | `cargo test -p homie-term --test grid_input_find -- --nocapture` | pass |
| FC-DTFS-002 | `cargo test -p homie-app --tests -- --nocapture` | pass |
| FC-DTFS-003 | `cargo clippy -p homie-app --all-targets -- -D warnings` | pass |

