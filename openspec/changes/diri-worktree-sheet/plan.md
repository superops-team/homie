# Diri Worktree Sheet OpenSpec Plan

> Change ID: `diri-worktree-sheet`
> Beads: `homie-hm1`

## Summary

Expose runtime worktree overview data in `homie-client` and render a first app worktree sheet with cleanup suggestion state.

## Verification

| Case | Command |
|------|---------|
| FC-DWS-001 | `cargo test -p homie-runtime --test worktree_safety -- --nocapture` |
| FC-DWS-002 | `cargo test -p homie-client --tests -- --nocapture` |
| FC-DWS-003 | `cargo test -p homie-app --tests -- --nocapture` |
| FC-DWS-004 | `cargo clippy -p homie-client -p homie-app --all-targets -- -D warnings` |

