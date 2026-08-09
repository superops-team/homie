# Functional Cases: Diri Hook Report Runtime

```yaml
change_id: diri-hook-report-runtime
beads: homie-b6o
```

## FC-DHRR-001: Hook report persists needs-input

- Command: `cargo test -p homie-cli --test hook_report_runtime_cli -- persists_permission_request_to_session_snapshot --nocapture`
- Expected: a real session receives `PermissionRequest`; snapshot shows `status.needsInput.kind=approval`.

## FC-DHRR-002: Parse-only fallback

- Command: `cargo test -p homie-cli hook_command_outputs_redacted_structured_event -- --nocapture`
- Expected: existing no-data-dir hook output remains redacted and succeeds.

## FC-DHRR-003: Quality gates

- Commands: check, clippy, diff, parity lock.

