# Functional Verification Report: Diri Usage Scan File Offset Cache

```yaml
change_id: diri-usage-scan-cache
beads: homie-xaz
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DUSC-001..002 | `cargo test -p homie-storage --test usage_scan_cache -- --nocapture` | failed: `UsageScanFileState`, `UsageScanFileQuery`, and repository methods did not exist. |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DUSC-001 | `cargo test -p homie-storage --test usage_scan_cache -- upserts_and_reads_usage_scan_file_state --nocapture` | pass |
| FC-DUSC-002 | `cargo test -p homie-storage --test usage_scan_cache -- lists_usage_scan_files_by_provider_and_profile --nocapture` | pass |
| FC-DUSC-001..002 | `cargo test -p homie-storage --test usage_scan_cache -- --nocapture` | pass: 2 passed |
| FC-DUSC-003 | `cargo test -p homie-storage --test diri_storage_indexing` | pass: 6 passed |
| FC-DUSC-003 | `cargo check -p homie-storage` | pass |
| FC-DUSC-003 | `cargo clippy -p homie-storage --all-targets -- -D warnings` | pass |
| FC-DUSC-003 | `cargo fmt --all -- --check` | pass after running `cargo fmt --all` |
| FC-DUSC-003 | scoped `git diff --check` | pass |
| FC-DUSC-003 | `make parity-lock` | pass; remaining unrelated partial rows listed honestly |

## Scope Notes

- Implements durable scan-file state repository only.
- Does not implement filesystem watcher, tail hash calculation, or transcript scanning.
