# Code Review Report: Diri Proto Host Catalog and Remote Config

```yaml
change_id: diri-proto-host-remote-config
beads: homie-5kx
status: pass
reviewed_at: 2026-08-08
```

## Round 1: Explicit Defect Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| medium | Missing DTO | `crates/homie-proto/src/lib.rs` | Host catalog and remote config protocol models were absent from `homie-proto`. | fixed: added HostNodeConfig, HostEntry, HostsConfig, and RemoteConfig. |
| medium | Wire contract | serde attributes | Diri uses camelCase for `defaultCwd`, `tokenFile`, `bindHost`, and `forwardAnyPort`. | fixed and tested. |

## Round 2: Hidden Risk Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| low | Scope | proto crate | File permission and atomic save behavior belongs in remote/config layer, not proto DTOs. | accepted: this slice only adds DTOs. |
| low | Minimal schema | HostEntry | Minimal Diri host requires only `id` and `ssh`; other fields must be optional. | pass: test covers minimal host. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-proto host_catalog_and_remote_config_match_diri_wire -- --nocapture` | pass |
| `cargo test -p homie-proto --tests` | pass |
| `cargo check -p homie-proto` | pass |
| `cargo clippy -p homie-proto --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
