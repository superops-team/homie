# Functional Verification Report: Diri Proto Host Sync Prefs Fixtures

```yaml
change_id: diri-proto-host-sync-fixtures
beads: homie-vuz
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DPHS-001 | `cargo test -p homie-proto host_sync_prefs_round_trips_diri_wire -- --nocapture` | failed: `HostSyncPrefsParams`, `PrefsSyncToolReport`, and `HostSyncPrefsResult` did not exist. |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DPHS-001 | `cargo test -p homie-proto host_sync_prefs_round_trips_diri_wire -- --nocapture` | pass |
| FC-DPHS-002 | `cargo test -p homie-proto --tests` | pass |
| FC-DPHS-002 | `cargo check -p homie-proto` | pass |
| FC-DPHS-002 | `cargo clippy -p homie-proto --all-targets -- -D warnings` | pass |
| FC-DPHS-002 | `cargo fmt --all -- --check` | pass after running `cargo fmt --all` |

## Scope Notes

- Implements DTO wire contract only.
- Does not implement remote sync execution or real remote E2E.
