# Release Readiness Report: Diri Proto Node Account Login Fixtures

```yaml
change_id: diri-proto-node-account-login
beads: homie-05q
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- Account profile/catalog/upsert/default DTOs.
- Account installation status DTO.
- Login start/challenge/session/input DTOs.
- Provider call params/result DTOs.
- Diri-compatible serde fixture for camelCase fields and optional omission.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Account/login fixture | `cargo test -p homie-proto node_account_login_match_diri_wire -- --nocapture` | pass |
| Proto tests | `cargo test -p homie-proto --tests` | pass |
| Build | `cargo check -p homie-proto` | pass |
| Lint | `cargo clippy -p homie-proto --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |

## Remaining Work

- Account storage/runtime.
- Login polling/input execution.
- Provider call runtime.
