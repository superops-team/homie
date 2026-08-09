# OpenSpec Plan: Diri Session History Protocol

> Change ID: `diri-session-history-protocol`

## Scope

Expose first-stage transcript scanner through `session.history` in Homie client and CLI.

## Cases

| Case | Command |
|------|---------|
| FC-DSHP-001 | `cargo test -p homie-client --test runtime_client history -- --nocapture` |
| FC-DSHP-002 | `cargo test -p homie-cli --test session_history_cli -- --nocapture` |
| FC-DSHP-003 | `cargo check -p homie-client -p homie-cli` |
| FC-DSHP-004 | scoped `git diff --check`; `make parity-lock` |
