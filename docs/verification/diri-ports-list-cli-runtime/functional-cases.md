# Functional Cases: Diri Ports List CLI Runtime

```yaml
change_id: diri-ports-list-cli-runtime
beads: homie-979
```

## FC-DPLC-001: Runtime-backed ports list

- Command: `cargo test -p homie-cli --test ports_cli -- lists_ports_from_runtime_session_output --nocapture`
- Expected: real session output containing `http://localhost:5173` appears in `homie ports --json`.

## FC-DPLC-002: Empty state

- Command: `cargo test -p homie-cli --test ports_cli -- ports_cli_reports_empty_state --nocapture`
- Expected: JSON output has empty `ports` array when no sessions contain ports.

## FC-DPLC-003: Quality gates

- Commands:
  - `cargo check -p homie-client -p homie-cli`
  - `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
  - scoped `git diff --check`
  - `make parity-lock`

