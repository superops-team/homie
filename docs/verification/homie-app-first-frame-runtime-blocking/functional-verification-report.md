# Functional Verification Report: Homie App First-frame Runtime Blocking

```yaml
change_id: homie-app-first-frame-runtime-blocking
beads: homie-7tb
status: pass_with_screenshot_blocker
validated_at: 2026-08-08
```

## Summary

The code-level first-frame/runtime blocking risks found during UI screenshot refresh were fixed:

- holder IPC now uses bounded read/write timeouts;
- `homie-app` no longer calls runtime resize from `Render::render`;
- invalid screenshot refresh artifacts were documented as rejected.

The local macOS screenshot/frontmost environment remains unreliable and is not treated as solved by this code fix.

## Results

| Case | Command | Result |
|------|---------|--------|
| FC-HAFRB-001 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass |
| FC-HAFRB-002 | `cargo test -p homie-app --test app_shell_copy_regression -- --nocapture` | pass |
| FC-HAFRB-003 | `cargo check -p homie-app` | pass |
| FC-HAFRB-004 | `cargo clippy -p homie-app -p homie-runtime --all-targets -- -D warnings` | pass |
| FC-HAFRB-005 | `make ui-screenshot-gate` | pass against accepted baseline screenshots |
| FC-HAFRB-005 | `make parity-lock` | pass_with_remaining_gaps |
| FC-HAFRB-005 | `loopx --registry .loopx/registry.json check --scan-root /Users/bytedance/workspace/github/homie` | pass |

## Screenshot Status

New screenshot refresh artifacts from this diagnosis are rejected. See `docs/verification/diri-ui-screenshot/white-screen-diagnosis-2026-08-07.md`.
