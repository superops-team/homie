# Diri Notification Center OpenSpec Plan

> Change ID: `diri-notification-center`  
> Beads: `homie-5pe`  
> Source PRD: `prd-spec/features/diri-notification-center/2026-08-07-diri-notification-center-design.md`  
> Status: `in_progress`

## 1. Summary

Implement the first notification-center slice for Homie UI parity. This adds a reusable notification model, status rollup, quick action descriptors, macOS notification command builder, and app inspector rollup display.

## 2. Scope

In scope:

- `homie-ui` notification model.
- Rollup from session status rows.
- Quick approve/deny descriptors from known agent capability.
- Safe macOS `osascript` display notification command builder.
- App inspector notification summary.

Out of scope:

- Actually dispatching approve/deny keystrokes.
- Menu bar resident app.
- Native notification E2E.

## 3. Verification Cases

| Case | Purpose | Command |
|------|---------|---------|
| FC-DNC-001 | Notification rollup and quick actions | `cargo test -p homie-ui --tests -- --nocapture` |
| FC-DNC-002 | App references notification rollup | `cargo test -p homie-app --tests -- --nocapture` |
| FC-DNC-003 | Touched crates pass clippy | `cargo clippy -p homie-ui -p homie-app --all-targets -- -D warnings` |
| FC-DNC-004 | Parity lock remains truthful | `make parity-lock` |

