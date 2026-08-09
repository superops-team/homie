# Release Readiness Report: Diri Usage Transcript Storage Import

```yaml
change_id: diri-usage-transcript-import
beads: homie-dh8
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- `UsageImportDefaults`.
- `UsageImportResult`.
- `Storage::record_transcript_usage_event`.
- `Storage::record_transcript_usage_events`.
- Mapping from `TranscriptUsageEvent` to `RecordUsage`.
- Storage dedupe through existing `(provider_id, source, source_event_id)` unique index.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Importer tests | `cargo test -p homie-storage --test usage_transcript_import -- --nocapture` | pass |
| Storage regression | `cargo test -p homie-storage --test diri_storage_indexing` | pass |
| Build | `cargo check -p homie-storage` | pass |
| Lint | `cargo clippy -p homie-storage --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |
| Parity lock | `make parity-lock` | pass |

## Remaining Work

- Directory watcher and offset cache.
- Pricing snapshot persistence.
- Usage UI/fleet merge.
- Provider-specific real provider id mapping beyond current defaults.
