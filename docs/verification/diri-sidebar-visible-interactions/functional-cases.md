# Functional Cases: Diri Sidebar Visible Interactions

```yaml
change_id: diri-sidebar-visible-interactions
beads: homie-5r1
```

## FC-DSVI-001: App source exposes visible sidebar controls

- Command: `cargo test -p homie-app --test app_shell_copy_regression -- --nocapture`
- Expected:
  - Source contains sidebar model state and sync helper.
  - Source contains `pin_sidebar_session`, `archive_sidebar_session`, and `toggle_sidebar_multi_select`.
  - Source contains visible `pin`, `select`, and `archive` row controls wired through click handlers.

## FC-DSVI-002: Existing sidebar model remains valid

- Command: `cargo test -p homie-ui --test workbench_state -- --nocapture`
- Expected:
  - Selection, multi-select, rename, pin/archive, reorder tests pass.

## FC-DSVI-003: App compile gate

- Command: `cargo check -p homie-app`
- Expected:
  - Exit code 0.

## FC-DSVI-004: App lint gate

- Command: `cargo clippy -p homie-app --all-targets -- -D warnings`
- Expected:
  - Exit code 0.

## FC-DSVI-005: Hygiene and parity lock

- Commands:
  - `git diff --check -- crates/homie-app/src/main.rs crates/homie-app/tests/app_shell_copy_regression.rs prd-spec/features/diri-sidebar-visible-interactions openspec/changes/diri-sidebar-visible-interactions docs/verification/diri-sidebar-visible-interactions`
  - `make parity-lock`
- Expected:
  - Diff check passes.
  - Parity lock remains valid and keeps UI rows partial.
