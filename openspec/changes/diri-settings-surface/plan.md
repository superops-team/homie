# Diri Settings Surface OpenSpec Plan

> Change ID: `diri-settings-surface`  
> Beads: `homie-s0w`  
> Source PRD: `prd-spec/features/diri-settings-surface/2026-08-07-diri-settings-surface-design.md`  
> Status: `in_progress`

## 1. Summary

Implement the first real settings surface for Homie UI parity. This turns `OpenSettings` from a local notice into a rendered settings panel backed by persisted preferences.

## 2. Scope

In scope:

- Typed preferences API in `homie-storage`.
- Settings preferences model.
- `homie-app` settings panel with General/Terminal/Resources/Remote tabs.
- Command palette `OpenSettings` wiring.
- Tests and parity lock evidence update.

Out of scope:

- Full remote pairing.
- Native macOS preferences window.
- Screenshot/E2E completion gate.

## 3. Verification Cases

| Case | Purpose | Command |
|------|---------|---------|
| FC-DSS-001 | Preferences persist through storage API | `cargo test -p homie-storage --test storage_bootstrap -- --nocapture` |
| FC-DSS-002 | App settings command opens real surface | `cargo test -p homie-app --tests -- --nocapture` |
| FC-DSS-003 | Touched crates pass clippy | `cargo clippy -p homie-storage -p homie-app --all-targets -- -D warnings` |
| FC-DSS-004 | Parity lock remains truthful | `make parity-lock` |

