# OpenSpec Plan: Diri Sidebar Visible Interactions

> Change ID: `diri-sidebar-visible-interactions`  
> Beads: `homie-5r1`  
> Source PRD: `prd-spec/features/diri-sidebar-visible-interactions/2026-08-07-diri-sidebar-visible-interactions-design.md`

## 1. Scope

Wire the existing sidebar session model into `homie-app` so Diri-style sidebar interactions become visible and clickable in the desktop shell. This is an app-surface slice only: it does not remove runtime sessions or claim complete UI parity.

## 2. Modules

| Module | Change |
|--------|--------|
| `crates/homie-app/src/main.rs` | Add app sidebar model state, sync helpers, and visible pin/archive/multi-select controls |
| `crates/homie-app/tests/app_shell_copy_regression.rs` | Add source regression checks for visible interaction wiring |
| `docs/verification/diri-sidebar-visible-interactions/` | Record spec review, functional cases, code review, release readiness |

## 3. Functional Cases

| Case | Purpose | Command |
|------|---------|---------|
| FC-DSVI-001 | App source exposes visible sidebar interactions and helper methods | `cargo test -p homie-app --test app_shell_copy_regression -- --nocapture` |
| FC-DSVI-002 | Existing sidebar model behavior remains valid | `cargo test -p homie-ui --test workbench_state -- --nocapture` |
| FC-DSVI-003 | App compiles after GPUI wiring | `cargo check -p homie-app` |
| FC-DSVI-004 | App lint gate | `cargo clippy -p homie-app --all-targets -- -D warnings` |
| FC-DSVI-005 | Scoped diff and parity honesty | scoped `git diff --check`, `make parity-lock` |

## 4. Risks

| Risk | Mitigation |
|------|------------|
| Archive accidentally kills runtime sessions | App helper only marks sidebar row archived and refreshes local projection |
| Sidebar model drifts from runtime sessions | `sync_sidebar_model` updates rows from `HomieClient` session list while preserving local interaction state |
| Source regression tests are weaker than UI E2E | Keep `UI-002` partial and list hover/drag/manual E2E as remaining |

## 5. Acceptance

- App sidebar rows have visible `pin`, `select`, and `archive` controls.
- Controls call helpers that update sidebar model state.
- Existing runtime selection remains clickable.
- Quality gates pass.
