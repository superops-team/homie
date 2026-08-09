# Alignment Report: Diri Usage Transcript Parser

```yaml
change_id: diri-usage-transcript-parser
beads: homie-hd1
status: aligned
checked_at: 2026-08-08
```

## PRD to Task Mapping

| PRD Requirement | OpenSpec Task | Verification |
|-----------------|---------------|--------------|
| FR-1 Neutral event model | T-002, T-003, T-004 | FC-DUTP-001, FC-DUTP-002 |
| FR-2 Claude parser | T-003 | FC-DUTP-001 |
| FR-3 Codex parser | T-004 | FC-DUTP-002 |
| FR-4 Timestamp and id | T-005 | FC-DUTP-003 |
| FR-5 Scope honesty | T-001, T-006 | FC-DUTP-004 |

## Functional Case Coverage

| Case | Requirement Coverage | Status before implementation |
|------|----------------------|------------------------------|
| FC-DUTP-001 | Claude parser | designed |
| FC-DUTP-002 | Codex parser | designed |
| FC-DUTP-003 | bad input/id stability | designed |
| FC-DUTP-004 | quality and parity honesty | designed |

## Scope Control

- No watcher.
- No offset cache.
- No storage write/import.
- No usage UI/fleet merge.
