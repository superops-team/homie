# Release Readiness Report: Diri Proto Node Hello Usage Fixtures

```yaml
change_id: diri-proto-node-usage-fixtures
beads: homie-7i0
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- Node method constants for hello/status/usage.
- Node capability constants for accounts/provider/fleet usage subset.
- `ProviderKind`.
- Node hello/status DTOs.
- Usage event/query/result DTOs.
- Diri-compatible serde fixtures for camelCase fields and lowercase provider map keys.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Node fixture | `cargo test -p homie-proto node_hello_and_usage_match_diri_wire -- --nocapture` | pass |
| Proto tests | `cargo test -p homie-proto --tests` | pass |
| Build | `cargo check -p homie-proto` | pass |
| Lint | `cargo clippy -p homie-proto --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |

## Remaining Work

- Account login DTOs.
- Checkpoint and move lease DTOs.
- Real first-party node runtime/E2E.
