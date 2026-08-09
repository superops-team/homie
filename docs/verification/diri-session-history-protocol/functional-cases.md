# Functional Cases: Diri Session History Protocol

```yaml
change_id: diri-session-history-protocol
beads: homie-ehm
```

## FC-DSHP-001

- Command: `cargo test -p homie-client --test runtime_client history -- --nocapture`

## FC-DSHP-002

- Command: `cargo test -p homie-cli --test session_history_cli -- --nocapture`

## FC-DSHP-003

- Command: `cargo check -p homie-client -p homie-cli`

## FC-DSHP-004

- Command: scoped `git diff --check`; `make parity-lock`
