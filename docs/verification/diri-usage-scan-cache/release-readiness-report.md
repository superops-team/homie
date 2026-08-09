# Release Readiness Report: Diri Usage Scan File Offset Cache

```yaml
change_id: diri-usage-scan-cache
beads: homie-xaz
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- `UsageScanFileState`.
- `UsageScanFileQuery`.
- `Storage::upsert_usage_scan_file`.
- `Storage::usage_scan_file`.
- `Storage::list_usage_scan_files`.
- Validation for required path/provider and non-negative numeric state.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Scan cache tests | `cargo test -p homie-storage --test usage_scan_cache -- --nocapture` | pass |
| Storage regression | `cargo test -p homie-storage --test diri_storage_indexing` | pass |
| Build | `cargo check -p homie-storage` | pass |
| Lint | `cargo clippy -p homie-storage --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |
| Parity lock | `make parity-lock` | pass |

## Remaining Work

- Filesystem watcher.
- Tail hash calculation.
- Incremental parser using saved offset.
- Pricing snapshot persistence.
- Usage UI and fleet merge.
