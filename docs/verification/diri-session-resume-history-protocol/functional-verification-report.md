# Functional Verification Report: Diri Session Resume From History Protocol

```yaml
change_id: diri-session-resume-history-protocol
beads: homie-tpq
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Summary

This slice adds a first `session.resume_from_history` path:

- `SessionResumeFromHistoryRequest` DTO.
- `HomieClient::resume_from_history`.
- `Method::SESSION_RESUME_FROM_HISTORY` dispatch.
- `homie session resume-history` CLI command.
- CLI integration test proving a history identity creates a runtime session.

Native Claude/Codex resume E2E remains pending.

## Results

| Case | Command | Result |
|------|---------|--------|
| FC-DSRHP-001 | `cargo test -p homie-cli --test session_resume_history_cli -- --nocapture` | pass |
| Regression | `cargo test -p homie-cli --test session_history_cli -- --nocapture` | pass |
| FC-DSRHP-002 | `cargo check -p homie-client -p homie-cli` | pass |
| FC-DSRHP-003 | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DSRHP-004 | scoped `git diff --check`; `make parity-lock` | pass / pass_with_remaining_gaps |

