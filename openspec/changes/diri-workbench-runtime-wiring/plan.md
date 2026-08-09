# Diri Workbench Runtime Wiring OpenSpec Plan

> Change ID: `diri-workbench-runtime-wiring`  
> Beads: `homie-3tz`  
> Source PRD: `prd-spec/features/diri-workbench-runtime-wiring/2026-08-07-diri-workbench-runtime-wiring-design.md`  
> Status: `in_progress`

## 1. Summary

Move the GPUI workbench from a single live-shell preview toward Diri-style runtime-backed interaction. This slice wires command palette spawn, sidebar session list/selection, selected-session snapshot attach, and runtime resize through `HomieClient`.

## 2. Scope

In scope:

- Runtime session projection in `AppState`.
- Real `SpawnShell` palette action.
- Sidebar session list and selection click wiring.
- Terminal refresh against selected session.
- Runtime resize helper through `HomieClient::resize_session`.
- Desktop shell spec update and regression tests.

Out of scope:

- Full screenshot automation.
- Drag reorder, hover card, rename, pin/archive, multi-select.
- Complete terminal selection/find/scrollback E2E.
- Settings, notifications, remote surfaces.

## 3. Verification Cases

| Case | Purpose | Command |
|------|---------|---------|
| FC-DWRW-001 | App source regression proves `SpawnShell` calls runtime spawn, not local notice placeholder | `cargo test -p homie-app --tests -- --nocapture` |
| FC-DWRW-002 | App source regression proves sidebar renders session rows with click selection | `cargo test -p homie-app --tests -- --nocapture` |
| FC-DWRW-003 | App source regression proves runtime resize path exists | `cargo test -p homie-app --tests -- --nocapture` |
| FC-DWRW-004 | Client runtime tests still pass | `cargo test -p homie-client --tests -- --nocapture` |
| FC-DWRW-005 | Parity lock remains truthful | `make parity-lock` |

## 4. Release Gate

This change may update `UI-001` and `UI-003` evidence, but must keep both rows `partial` until full GPUI interaction/screenshot E2E passes.

