# OpenSpec Plan: Diri Proto Host Sync Prefs Fixtures

```yaml
change_id: diri-proto-host-sync-fixtures
beads: homie-vuz
source_prd: prd-spec/features/diri-proto-host-sync-fixtures/2026-08-08-diri-proto-host-sync-fixtures-design.md
```

## 1. Scope

Add Diri-compatible `host.sync_prefs` DTOs and serde fixtures to `homie-proto`.

## 2. Target State

```text
HostSyncPrefsParams { host }
PrefsSyncToolReport { tool, ok, synced, error? }
HostSyncPrefsResult { tools }
```

## 3. Verification

- FC-DPHS-001 DTO wire contract.
- FC-DPHS-002 quality gates.
