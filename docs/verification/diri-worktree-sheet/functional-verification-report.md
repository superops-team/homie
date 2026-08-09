# Diri Worktree Sheet Functional Verification Report

```yaml
change_id: diri-worktree-sheet
beads: homie-hm1
status: pass
validated_at: 2026-08-07
```

## Summary

This slice advances `UI-007`:

- `HomieClient::worktree_overview` projects runtime sessions into a worktree overview.
- `homie-app` now has a Worktrees surface with status and cleanup suggestion badges.
- The former notice-only ToggleSidebar branch now opens the worktree sheet.

`UI-007` and `GIT-002` remain `partial` until create/remove/cleanup E2E is complete.

## Functional Cases

| Case | Command | Result |
|------|---------|--------|
| FC-DWS-001 | `cargo test -p homie-runtime --test worktree_safety -- --nocapture` | pass |
| FC-DWS-002 | `cargo test -p homie-client --tests -- --nocapture` | pass |
| FC-DWS-003 | `cargo test -p homie-app --tests -- --nocapture` | pass |
| FC-DWS-004 | `cargo clippy -p homie-client -p homie-app --all-targets -- -D warnings` | pass |

