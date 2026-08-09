# Release Readiness Report: Diri Codex Notify Runtime

```yaml
change_id: diri-notify-runtime
beads: homie-qki
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- `homie notify --data-dir`.
- Runtime/client `report_turn_complete`.
- Persisted idle status visible through `session snapshot`.
- Adoption logic preserves persisted idle/needs-input.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Notify runtime E2E | `cargo test -p homie-cli --test notify_runtime_cli -- --nocapture` | pass |
| Notify parse-only regression | `cargo test -p homie-cli notify_command_outputs_codex_turn_complete -- --nocapture` | pass |
| Build | `cargo check -p homie-runtime -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-runtime -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |

## Remaining Work

- Full notify event bus.
- Additional notify event types and status transitions.
