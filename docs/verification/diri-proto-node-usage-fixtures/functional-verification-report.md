# Functional Verification Report: Diri Proto Node Hello Usage Fixtures

```yaml
change_id: diri-proto-node-usage-fixtures
beads: homie-7i0
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DPNU-001 | `cargo test -p homie-proto node_hello_and_usage_match_diri_wire -- --nocapture` | failed: node method/capability/provider/hello/status/usage DTOs were missing. |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DPNU-001 | `cargo test -p homie-proto node_hello_and_usage_match_diri_wire -- --nocapture` | pass |
| FC-DPNU-002 | `cargo test -p homie-proto --tests` | pass |
| FC-DPNU-002 | `cargo check -p homie-proto` | pass |
| FC-DPNU-002 | `cargo clippy -p homie-proto --all-targets -- -D warnings` | pass |
| FC-DPNU-002 | `cargo fmt --all -- --check` | pass after running `cargo fmt --all` |

## Scope Notes

- Implements node hello/status/usage DTO wire fixtures only.
- Does not implement account login, checkpoint, move lease, or node runtime/network behavior.
