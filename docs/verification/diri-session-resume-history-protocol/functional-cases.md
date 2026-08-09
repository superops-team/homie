# Functional Cases: Diri Session Resume From History Protocol

```yaml
change_id: diri-session-resume-history-protocol
beads: homie-tpq
```

## FC-DSRHP-001

- Command: `cargo test -p homie-cli --test session_resume_history_cli -- --nocapture`

## FC-DSRHP-002

- Command: `cargo check -p homie-client -p homie-cli`

## FC-DSRHP-003

- Command: `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`

## FC-DSRHP-004

- Command: scoped `git diff --check`; `make parity-lock`
