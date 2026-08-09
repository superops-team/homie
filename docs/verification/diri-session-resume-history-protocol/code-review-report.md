# Code Review Report: Diri Session Resume From History Protocol

```yaml
change_id: diri-session-resume-history-protocol
beads: homie-tpq
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| medium | Correctness | `homie-client` / `homie-cli` | `session.resume_from_history` existed in the method catalog but had no executable path. | fixed: added client dispatch and CLI command. |
| low | Scope | parity lock | Shell-backed command queuing does not prove native Claude/Codex resume E2E. | accepted: rows remain partial. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test session_resume_history_cli -- --nocapture` | pass |
| `cargo test -p homie-cli --test session_history_cli -- --nocapture` | pass |
| `cargo check -p homie-client -p homie-cli` | pass |
| `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| `make parity-lock` | pass_with_remaining_gaps |

