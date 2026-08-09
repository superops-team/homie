# Release Readiness Report: Diri Session History Protocol

```yaml
change_id: diri-session-history-protocol
beads: homie-ehm
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- `SessionHistoryRequest` DTO.
- `HomieClient::session_history`.
- `Method::SESSION_HISTORY` protocol dispatch.
- `homie session history` CLI command.
- Fixture-backed CLI integration test.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| CLI history | `cargo test -p homie-cli --test session_history_cli -- --nocapture` | pass |
| Runtime scanner | `cargo test -p homie-runtime --test history_scanner -- --nocapture` | pass |
| Build | `cargo check -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Parity lock | `make parity-lock` | pass_with_remaining_gaps |

## Remaining Work

- App history surface.
- `session.resume_from_history`.
- Real Claude/Codex resume E2E.
