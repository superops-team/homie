# Functional Cases: Diri Usage Transcript Storage Import

```yaml
change_id: diri-usage-transcript-import
beads: homie-dh8
```

## FC-DUTI-001: Import parsed events into storage totals

- Command: `cargo test -p homie-storage --test usage_transcript_import -- imports_transcript_usage_events_into_storage_totals --nocapture`
- Expected:
  - Two transcript usage events are inserted.
  - `query_usage_totals` returns matching tokens/cache/estimated cost.

## FC-DUTI-002: Re-import deduplicates source events

- Command: `cargo test -p homie-storage --test usage_transcript_import -- reimport_deduplicates_source_events --nocapture`
- Expected:
  - First import inserted=1/skipped=0.
  - Second import inserted=0/skipped=1.
  - Totals count remains 1.

## FC-DUTI-003: Quality gates

- Commands:
  - `cargo test -p homie-storage --test diri_storage_indexing`
  - `cargo check -p homie-storage`
  - `cargo clippy -p homie-storage --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - scoped `git diff --check`
  - `make parity-lock`
- Expected: all commands pass.
