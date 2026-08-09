# Code Review Report: Diri Session History Protocol

```yaml
change_id: diri-session-history-protocol
beads: homie-ehm
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| medium | Correctness | `homie-client` / `homie-cli` | History scanner existed only as runtime library code and had no client/CLI path. | fixed: added `session.history` dispatch and CLI `session history`. |
| low | Scope | parity lock | Protocol exposure does not complete app history UI or real resume E2E. | accepted: `AG-004` remains partial. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test session_history_cli -- --nocapture` | pass |
| `cargo test -p homie-runtime --test history_scanner -- --nocapture` | pass |
| `cargo check -p homie-client -p homie-cli` | pass |
| `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| scoped `git diff --check` | pass |
| `make parity-lock` | pass_with_remaining_gaps |

