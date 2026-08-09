# Release Readiness Report: Diri Session Resume From History Protocol

```yaml
change_id: diri-session-resume-history-protocol
beads: homie-tpq
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- `session.resume_from_history` request DTO.
- Client protocol dispatch and CLI command.
- Runtime-backed session creation from history identity.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Resume history CLI | `cargo test -p homie-cli --test session_resume_history_cli -- --nocapture` | pass |
| History CLI regression | `cargo test -p homie-cli --test session_history_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Parity lock | `make parity-lock` | pass_with_remaining_gaps |

## Remaining Work

- Native Claude/Codex resume E2E.
- App history surface integration.
