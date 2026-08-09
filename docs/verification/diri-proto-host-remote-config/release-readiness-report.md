# Release Readiness Report: Diri Proto Host Catalog and Remote Config

```yaml
change_id: diri-proto-host-remote-config
beads: homie-5kx
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- `HostNodeConfig`.
- `HostEntry`.
- `HostsConfig`.
- `RemoteConfig`.
- Diri-compatible serde fixture for documented/minimal host catalog and current/legacy remote config.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Host/remote fixture | `cargo test -p homie-proto host_catalog_and_remote_config_match_diri_wire -- --nocapture` | pass |
| Proto tests | `cargo test -p homie-proto --tests` | pass |
| Build | `cargo check -p homie-proto` | pass |
| Lint | `cargo clippy -p homie-proto --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |

## Remaining Work

- Owner-only load/save behavior in config layer.
- App settings wiring.
- Real remote node/SSH E2E.
