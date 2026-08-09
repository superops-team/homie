# Functional Verification Report: Diri Proto Host Catalog and Remote Config

```yaml
change_id: diri-proto-host-remote-config
beads: homie-5kx
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DPHR-001 | `cargo test -p homie-proto host_catalog_and_remote_config_match_diri_wire -- --nocapture` | failed: `HostEntry`, `HostNodeConfig`, `HostsConfig`, and `RemoteConfig` were missing. |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DPHR-001 | `cargo test -p homie-proto host_catalog_and_remote_config_match_diri_wire -- --nocapture` | pass |
| FC-DPHR-002 | `cargo test -p homie-proto --tests` | pass |
| FC-DPHR-002 | `cargo check -p homie-proto` | pass |
| FC-DPHR-002 | `cargo clippy -p homie-proto --all-targets -- -D warnings` | pass |
| FC-DPHR-002 | `cargo fmt --all -- --check` | pass after running `cargo fmt --all` |

## Scope Notes

- Implements protocol DTO/serde contract only.
- Does not implement load/save permissions, remote node execution, or settings UI wiring.
