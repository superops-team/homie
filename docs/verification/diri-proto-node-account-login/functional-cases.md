# Functional Cases: Diri Proto Node Account Login Fixtures

```yaml
change_id: diri-proto-node-account-login
beads: homie-05q
```

## FC-DPNA-001: Account/login/provider call wire fixture

- Command: `cargo test -p homie-proto node_account_login_match_diri_wire -- --nocapture`
- Expected:
  - Account catalog/default provider map round-trips.
  - Installation status and login challenge use Diri camelCase.
  - Optional fields are omitted when None.
  - Provider call params/result round-trip.

## FC-DPNA-002: Quality gates

- Commands:
  - `cargo test -p homie-proto --tests`
  - `cargo check -p homie-proto`
  - `cargo clippy -p homie-proto --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - scoped `git diff --check`
  - `make parity-lock`
