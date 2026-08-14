# GPUI render purity Release Readiness Report

## Conclusion

`gpui-render-path-purity` first slice is ready to land.

## Delivered

- Extracted `shortcut_ranks` pure helper from `Sidebar::render`.
- Added test for Cmd+1..9 rank mapping.
- Added PRD/OpenSpec/evidence for this first render-purity slice.

## Verification

| Gate | Result |
|------|--------|
| `cargo test --manifest-path homie/Cargo.toml -p homie-app shortcut_rank -- --nocapture` | pass |
| `cargo test --manifest-path homie/Cargo.toml -p homie-app sidebar -- --nocapture` | pass |
| `(cd homie && cargo fmt --check)` | pass |
| `git diff --check` | pass |
| scope guard for `homie-ui`, `homie-engine`, `homie-client` | pass |

## Not Run

- Full workspace tests were not run. This slice is pure helper extraction plus
  targeted sidebar tests.
