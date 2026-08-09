# Diri Terminal Find Surface OpenSpec Plan

> Change ID: `diri-terminal-find-surface`
> Beads: `homie-42v`

## Summary

Expose terminal find as an app-visible surface backed by `homie-term` find model state.

## Verification

| Case | Command |
|------|---------|
| FC-DTFS-001 | `cargo test -p homie-term --test grid_input_find -- --nocapture` |
| FC-DTFS-002 | `cargo test -p homie-app --tests -- --nocapture` |
| FC-DTFS-003 | `cargo clippy -p homie-app --all-targets -- -D warnings` |

