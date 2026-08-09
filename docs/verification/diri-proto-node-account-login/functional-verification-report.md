# Functional Verification Report: Diri Proto Node Account Login Fixtures

```yaml
change_id: diri-proto-node-account-login
beads: homie-05q
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DPNA-001 | `cargo test -p homie-proto node_account_login_match_diri_wire -- --nocapture` | failed: account/login/provider-call DTOs were missing. |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DPNA-001 | `cargo test -p homie-proto node_account_login_match_diri_wire -- --nocapture` | pass |
| FC-DPNA-002 | `cargo test -p homie-proto --tests` | pass |
| FC-DPNA-002 | `cargo check -p homie-proto` | pass |
| FC-DPNA-002 | `cargo clippy -p homie-proto --all-targets -- -D warnings` | pass |
| FC-DPNA-002 | `cargo fmt --all -- --check` | pass after running `cargo fmt --all` |

## Scope Notes

- Implements account/login/provider-call DTO wire fixtures only.
- Does not implement account storage, login polling, or provider call runtime.
