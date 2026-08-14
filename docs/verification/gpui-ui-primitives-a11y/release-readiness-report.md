# GPUI UI primitives Release Readiness Report

## Conclusion

`gpui-ui-primitives-a11y` first slice is ready to land.

## Delivered

- Added `Button`, `ButtonVariant`, `ButtonSize`, and `ButtonStyle` to `homie-ui`.
- Exported the new primitive from `homie-ui`.
- Migrated Settings dialog `close-settings` to the new Button primitive.
- Added Button focused tests and close-settings regression verification.

## Verification

| Gate | Result |
|------|--------|
| `cargo test --manifest-path homie/Cargo.toml -p homie-ui button -- --nocapture` | pass |
| `cargo test --manifest-path homie/Cargo.toml -p homie-app settings_content_close_control_dismisses_the_surface -- --nocapture` | pass |
| `(cd homie && cargo fmt --check)` | pass |
| `git diff --check` | pass |
| scope guard for `homie-engine`, `homie-client` | pass |

## Not Run

- Full workspace tests were not run. This is a small first primitive slice with
  targeted component and app regression tests.
