# Code Review Report: Diri Usage Scan File Offset Cache

```yaml
change_id: diri-usage-scan-cache
beads: homie-xaz
status: pass
reviewed_at: 2026-08-08
```

## Round 1: Explicit Defect Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| high | Missing behavior | `crates/homie-storage/src/lib.rs` | `usage_scan_files` schema existed without repository API. | fixed: added state/query structs and upsert/get/list methods. |
| medium | Upsert semantics | `upsert_usage_scan_file` | Same path must overwrite old offset/hash/model. | fixed and tested with overwrite assertions. |
| medium | Query semantics | `list_usage_scan_files` | Provider/profile filters must not accidentally exclude null-profile rows when no profile filter is requested. | fixed: `None` query values do not filter. |

## Round 2: Hidden Risk Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| medium | Scope | repository API | This slice should not scan the filesystem or compute hashes. | pass: API only persists caller-supplied state. |
| medium | Schema stability | storage | Offset cache repository should not require a migration. | pass: uses existing schema. |
| low | Validation | `validate_usage_scan_file` | Negative size/offset/timestamps can corrupt watcher semantics. | pass: validates required strings and non-negative numeric fields. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-storage --test usage_scan_cache -- --nocapture` | pass |
| `cargo test -p homie-storage --test diri_storage_indexing` | pass |
| `cargo check -p homie-storage` | pass |
| `cargo clippy -p homie-storage --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |
| `make parity-lock` | pass |
