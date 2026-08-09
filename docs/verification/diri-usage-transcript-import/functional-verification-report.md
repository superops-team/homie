# Functional Verification Report: Diri Usage Transcript Storage Import

```yaml
change_id: diri-usage-transcript-import
beads: homie-dh8
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DUTI-001..002 | `cargo test -p homie-storage --test usage_transcript_import -- --nocapture` | failed: `UsageImportDefaults` and `record_transcript_usage_events` did not exist. |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DUTI-001 | `cargo test -p homie-storage --test usage_transcript_import -- imports_transcript_usage_events_into_storage_totals --nocapture` | pass |
| FC-DUTI-002 | `cargo test -p homie-storage --test usage_transcript_import -- reimport_deduplicates_source_events --nocapture` | pass |
| FC-DUTI-001..002 | `cargo test -p homie-storage --test usage_transcript_import -- --nocapture` | pass: 2 passed |
| FC-DUTI-003 | `cargo test -p homie-storage --test diri_storage_indexing` | pass: 6 passed |
| FC-DUTI-003 | `cargo check -p homie-storage` | pass |
| FC-DUTI-003 | `cargo clippy -p homie-storage --all-targets -- -D warnings` | pass |
| FC-DUTI-003 | `cargo fmt --all -- --check` | pass after running `cargo fmt --all` |
| FC-DUTI-003 | scoped `git diff --check` | pass |
| FC-DUTI-003 | `make parity-lock` | pass; remaining unrelated partial rows listed honestly |

## Fix Notes

- First implementation mapped provider ids as `provider_<kind>`, which failed seeded storage foreign keys.
- Final implementation takes provider metadata from `UsageImportDefaults`; `from_session` defaults to `provider_local_placeholder`.

## Scope Notes

- Implements parser event to storage import only.
- Does not implement directory watcher, offset cache, pricing snapshot persistence, usage UI, or fleet merge.
