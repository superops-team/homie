# Diri Notification Center Functional Verification Report

```yaml
change_id: diri-notification-center
beads: homie-5pe
status: pass
validated_at: 2026-08-07
```

## Summary

This slice moves `UI-008` from missing to partial:

- `homie-ui` now provides notification severity, item, action, rollup, redaction, and macOS notification command builder.
- Rollup counts total/running/exited/needs-input sessions.
- Quick approve/deny action descriptors are exposed only when the agent capability is known.
- `homie-app` inspector renders notification rollup from session state.

`UI-008` remains `partial` until native notification delivery, sound playback, menu bar behavior, and real quick approve/deny execution have E2E evidence.

## Functional Cases

| Case | Command | Result | Evidence |
|------|---------|--------|----------|
| FC-DNC-001 | `cargo test -p homie-ui --tests -- --nocapture` | pass | 15 tests passed, including notification rollup, quick actions, redaction, and macOS command builder |
| FC-DNC-002 | `cargo test -p homie-app --tests -- --nocapture` | pass | App regression confirms inspector renders notification rollup |
| FC-DNC-003 | `cargo clippy -p homie-ui -p homie-app --all-targets -- -D warnings` | pass | touched UI/app crates pass clippy |

## Gate Decision

Decision: pass

Reason:

- Notification center is no longer absent.
- Native/system behavior remains explicitly tracked as pending.

