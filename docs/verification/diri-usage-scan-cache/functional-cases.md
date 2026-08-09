# Functional Cases: Diri Usage Scan File Offset Cache

```yaml
change_id: diri-usage-scan-cache
beads: homie-xaz
```

## FC-DUSC-001: Upsert and read scan file state

- Command: `cargo test -p homie-storage --test usage_scan_cache -- upserts_and_reads_usage_scan_file_state --nocapture`
- Expected:
  - Initial upsert can be read by path.
  - Second upsert on same path overwrites offset, tail hash, model, and scanned_at.

## FC-DUSC-002: List by provider/profile

- Command: `cargo test -p homie-storage --test usage_scan_cache -- lists_usage_scan_files_by_provider_and_profile --nocapture`
- Expected:
  - Provider filter returns only matching provider.
  - Profile filter returns only matching profile.
  - Empty profile filter can include null-profile rows.

## FC-DUSC-003: Quality gates

- Commands:
  - `cargo test -p homie-storage --test diri_storage_indexing`
  - `cargo check -p homie-storage`
  - `cargo clippy -p homie-storage --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - scoped `git diff --check`
  - `make parity-lock`
- Expected: all commands pass.
