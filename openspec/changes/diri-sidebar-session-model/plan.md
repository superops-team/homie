# Diri Sidebar Session Model OpenSpec Plan

> Change ID: `diri-sidebar-session-model`
> Beads: `homie-f02`

## Summary

Add a tested sidebar session state model for Diri-style selection, multi-select, rename, pin/archive, reorder, and status glyph projection.

## Verification

| Case | Command |
|------|---------|
| FC-DSSM-001 | `cargo test -p homie-ui --tests -- --nocapture` |
| FC-DSSM-002 | `cargo clippy -p homie-ui --all-targets -- -D warnings` |
| FC-DSSM-003 | `make parity-lock` |

