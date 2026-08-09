# Functional Cases: Diri Proto Host Sync Prefs Fixtures

```yaml
change_id: diri-proto-host-sync-fixtures
beads: homie-vuz
```

## FC-DPHS-001: Host sync prefs DTO wire contract

- Command: `cargo test -p homie-proto host_sync_prefs_round_trips_diri_wire -- --nocapture`
- Expected:
  - `HostSyncPrefsParams` serializes as `{ "host": "forge" }`.
  - `PrefsSyncToolReport.error=None` is omitted.
  - Failure report preserves `error`.
  - `HostSyncPrefsResult` round-trips Diri fixture.

## FC-DPHS-002: Quality gates

- Commands:
  - `cargo test -p homie-proto --tests`
  - `cargo check -p homie-proto`
  - `cargo clippy -p homie-proto --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - scoped `git diff --check`
  - `make parity-lock`
- Expected: all pass.
