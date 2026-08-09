# OpenSpec Plan: Diri Proto Host Catalog and Remote Config

```yaml
change_id: diri-proto-host-remote-config
beads: homie-5kx
source_prd: prd-spec/features/diri-proto-host-remote-config/2026-08-08-diri-proto-host-remote-config-design.md
```

## 1. Scope

Add Diri-compatible host catalog and remote config DTOs to `homie-proto`.

## 2. Target State

```text
HostNodeConfig
HostEntry
HostsConfig
RemoteConfig
```

## 3. Verification

- FC-DPHR-001 serde fixtures.
- FC-DPHR-002 quality gates.
