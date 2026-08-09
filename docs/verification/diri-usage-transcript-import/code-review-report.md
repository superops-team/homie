# Code Review Report: Diri Usage Transcript Storage Import

```yaml
change_id: diri-usage-transcript-import
beads: homie-dh8
status: pass
reviewed_at: 2026-08-08
```

## Round 1: Explicit Defect Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| high | Missing behavior | `crates/homie-storage/src/lib.rs` | Parsed transcript usage events had no path into storage usage records. | fixed: added single and batch transcript usage import APIs. |
| high | Dedupe | importer mapping | Transcript event id must be used as `source_event_id` to reuse storage dedupe. | fixed and tested with reimport skipped. |
| medium | Foreign keys | provider mapping | Hardcoded `provider_codex/provider_claude` did not exist in seeded storage. | fixed: provider id comes from `UsageImportDefaults`. |

## Round 2: Hidden Risk Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| medium | Scope | importer implementation | Importer should not scan directories or maintain offsets. | pass: API only accepts parsed events. |
| medium | Schema stability | storage | This slice should not alter schema. | pass: no migration/schema changes. |
| low | Cost semantics | mapping | Estimated costs should remain strings in storage. | pass: importer formats cost as decimal string and leaves billed cost optional. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-storage --test usage_transcript_import -- --nocapture` | pass |
| `cargo test -p homie-storage --test diri_storage_indexing` | pass |
| `cargo check -p homie-storage` | pass |
| `cargo clippy -p homie-storage --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |
| `make parity-lock` | pass |
