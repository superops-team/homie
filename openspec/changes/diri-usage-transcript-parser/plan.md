# OpenSpec Plan: Diri Usage Transcript Parser

```yaml
change_id: diri-usage-transcript-parser
beads: homie-hd1
source_prd: prd-spec/features/diri-usage-transcript-parser/2026-08-08-diri-usage-transcript-parser-design.md
component_specs:
  - specs/llm-proxy/README.md
```

## 1. Scope

Add pure Claude/Codex transcript usage parsing to `homie-llm`. The parser outputs neutral events and does not scan directories, cache offsets, or write storage.

## 2. Current State

- `homie-llm` has Diri-compatible pricing helpers.
- `homie-storage` can record usage, but there is no parser-generated event model yet.
- Diri node parser provides a clear file -> usage events contract.

## 3. Target State

```text
parse_transcript_usage_events(path, provider, profile_id)
  -> Vec<TranscriptUsageEvent>
  -> caller can later map into RecordUsage
```

## 4. Module Changes

| Module | Change |
|--------|--------|
| `homie-llm` | Add provider/source/value enums, `TranscriptUsageEvent`, and parser functions. |
| `homie-llm tests` | Add Claude/Codex parser fixtures and safety tests. |
| `specs/llm-proxy` | Document transcript parser as storage import predecessor. |
| parity docs | Add USAGE-001 evidence while keeping watcher/import/UI/fleet gaps partial. |

## 5. Verification

- FC-DUTP-001 Claude parser.
- FC-DUTP-002 Codex parser.
- FC-DUTP-003 safety/id stability.
- FC-DUTP-004 quality gates.
