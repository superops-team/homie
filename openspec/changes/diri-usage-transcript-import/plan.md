# OpenSpec Plan: Diri Usage Transcript Storage Import

```yaml
change_id: diri-usage-transcript-import
beads: homie-dh8
source_prd: prd-spec/features/diri-usage-transcript-import/2026-08-08-diri-usage-transcript-import-design.md
```

## 1. Scope

Add a minimal importer from `homie-llm::TranscriptUsageEvent` to `homie-storage::RecordUsage`.

## 2. Current State

- Parser emits neutral usage events.
- Storage can record/query usage records.
- No API maps parser events into storage.

## 3. Target State

```text
record_transcript_usage_events(events, defaults)
  -> map each event into RecordUsage
  -> record_usage
  -> UsageImportResult { inserted, skipped }
```

## 4. Verification

- FC-DUTI-001 import totals.
- FC-DUTI-002 dedupe.
- FC-DUTI-003 quality gates.
