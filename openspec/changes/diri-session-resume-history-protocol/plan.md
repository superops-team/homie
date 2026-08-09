# OpenSpec Plan: Diri Session Resume From History Protocol

> Change ID: `diri-session-resume-history-protocol`

## Scope

Expose a first `session.resume_from_history` path using the history scanner's resume command projection.

## Cases

| Case | Command |
|------|---------|
| FC-DSRHP-001 | `cargo test -p homie-cli --test session_resume_history_cli -- --nocapture` |
| FC-DSRHP-002 | `cargo check -p homie-client -p homie-cli` |
| FC-DSRHP-003 | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` |
| FC-DSRHP-004 | scoped `git diff --check`; `make parity-lock` |
