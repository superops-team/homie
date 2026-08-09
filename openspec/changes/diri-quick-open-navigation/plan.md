# Diri Quick Open Navigation OpenSpec Plan

> Change ID: `diri-quick-open-navigation`
> Beads: `homie-5ya`

## Summary

Turn app Quick Open from notice-only into a real surface backed by ranked session/navigation items.

## Verification

| Case | Command |
|------|---------|
| FC-DQO-001 | `cargo test -p homie-app --tests -- --nocapture` |
| FC-DQO-002 | `cargo test -p homie-ui --tests -- --nocapture` |
| FC-DQO-003 | `cargo clippy -p homie-app -p homie-ui --all-targets -- -D warnings` |
| FC-DQO-004 | `make parity-lock` |

