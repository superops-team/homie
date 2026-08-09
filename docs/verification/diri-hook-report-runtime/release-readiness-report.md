# Release Readiness Report: Diri Hook Report Runtime

```yaml
change_id: diri-hook-report-runtime
beads: homie-b6o
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- `homie hook --data-dir`.
- Storage API for session needs-input metadata.
- Runtime persisted needs-input projection in `session_status_report`.
- Client `report_needs_input`.
- CLI E2E proving `PermissionRequest` appears in `session snapshot`.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Hook runtime E2E | `cargo test -p homie-cli --test hook_report_runtime_cli -- --nocapture` | pass |
| Hook parse-only regression | `cargo test -p homie-cli hook_command_outputs_redacted_structured_event -- --nocapture` | pass |
| Build | `cargo check -p homie-storage -p homie-runtime -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-storage -p homie-runtime -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |

## Remaining Work

- Full hook event bus and runtime status matrix.
- Codex notify persistence and turn-complete state transitions.
