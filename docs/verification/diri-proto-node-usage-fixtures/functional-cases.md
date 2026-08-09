# Functional Cases: Diri Proto Node Hello Usage Fixtures

```yaml
change_id: diri-proto-node-usage-fixtures
beads: homie-7i0
```

## FC-DPNU-001: Node hello/status/usage wire fixtures

- Command: `cargo test -p homie-proto node_hello_and_usage_match_diri_wire -- --nocapture`
- Expected:
  - Node method/capability constants match Diri names.
  - Node hello params serialize token only in params, not result.
  - Node hello/status result use camelCase.
  - ProviderKind map keys serialize as `claude`/`codex`.
  - Usage event/query/result round-trip with Diri camelCase fields.

## FC-DPNU-002: Quality gates

- Commands:
  - `cargo test -p homie-proto --tests`
  - `cargo check -p homie-proto`
  - `cargo clippy -p homie-proto --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - scoped `git diff --check`
  - `make parity-lock`
