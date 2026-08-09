# Functional Cases: Diri Proto Host Catalog and Remote Config

```yaml
change_id: diri-proto-host-remote-config
beads: homie-5kx
```

## FC-DPHR-001: Host catalog and remote config wire fixtures

- Command: `cargo test -p homie-proto host_catalog_and_remote_config_match_diri_wire -- --nocapture`
- Expected:
  - Host catalog decodes documented schema.
  - Minimal host entry only requires id and ssh.
  - Empty object decodes to empty hosts.
  - Remote config decodes current schema with bindHost/forwardAnyPort.
  - Remote config decodes legacy schema without optional fields.

## FC-DPHR-002: Quality gates

- Commands:
  - `cargo test -p homie-proto --tests`
  - `cargo check -p homie-proto`
  - `cargo clippy -p homie-proto --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - scoped `git diff --check`
  - `make parity-lock`
- Expected: all pass.
