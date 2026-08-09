# Diri Settings Surface Functional Verification Report

```yaml
change_id: diri-settings-surface
beads: homie-s0w
status: pass
validated_at: 2026-08-07
```

## Summary

This slice moves `UI-006` from missing to partial:

- `homie-storage` now exposes typed settings preferences with persisted JSON roundtrip through the `preferences` table.
- `homie-app` now has a real settings surface with General, Terminal, Resources, and Remote tabs.
- `PaletteCommand::OpenSettings` opens the settings surface instead of setting a local-only notice.
- Terminal font size and remote companion access write back to persisted settings preferences.

`UI-006` remains `partial` until a full settings interaction/screenshot E2E gate is added.

## Functional Cases

| Case | Command | Result | Evidence |
|------|---------|--------|----------|
| FC-DSS-001 | `cargo test -p homie-storage --test storage_bootstrap -- --nocapture` | pass | 5 tests passed, including settings preferences roundtrip |
| FC-DSS-002 | `cargo test -p homie-app --tests -- --nocapture` | pass | App regression confirms OpenSettings real surface and persisted preferences wiring |
| FC-DSS-003 | `cargo clippy -p homie-storage -p homie-app --all-targets -- -D warnings` | pass | touched storage/app crates pass clippy |

## Gate Decision

Decision: pass

Reason:

- Settings is no longer missing or notice-only.
- The implementation has durable preference storage.
- Full Diri settings parity still needs E2E/screenshot coverage.

