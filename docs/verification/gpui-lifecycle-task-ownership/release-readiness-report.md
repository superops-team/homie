# GPUI lifecycle ownership Release Readiness Report

## Conclusion

`gpui-lifecycle-task-ownership` is ready to land. RootView child entity
subscriptions are now explicitly owned by `_subscriptions`.

## Delivered

- Child PRD/spec.
- Functional verification cases.
- OpenSpec plan/tasks/alignment.
- RootView child subscription ownership change.
- Functional verification report and two review rounds.

## Verification

| Gate | Result |
|------|--------|
| FC-01 through FC-04 | pass |
| `cargo test --manifest-path homie/Cargo.toml -p homie-app root -- --nocapture` | pass |
| `cargo test --manifest-path homie/Cargo.toml -p homie-app surface_shell::tests -- --nocapture` | pass |
| `(cd homie && cargo fmt --check)` | pass |
| `git diff --check` | pass |
| scope guard for `homie-ui`, `homie-engine`, `homie-client` | pass |

## Not Run

- Full workspace tests were not run. This slice only changes RootView
  subscription ownership and uses targeted root/surface_shell coverage.
