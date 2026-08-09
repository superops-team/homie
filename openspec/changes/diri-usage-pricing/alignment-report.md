# Alignment Report: Diri Usage Pricing Estimate

```yaml
change_id: diri-usage-pricing
beads: homie-t3e
status: aligned
checked_at: 2026-08-08
```

## PRD to Task Mapping

| PRD Requirement | OpenSpec Task | Verification |
|-----------------|---------------|--------------|
| FR-1 Pricing model | T-003 | FC-DUP-001 |
| FR-2 Provider model matching | T-003 | FC-DUP-001, FC-DUP-002 |
| FR-3 Cost estimate | T-003, T-004 | FC-DUP-001, FC-DUP-002, FC-DUP-003 |
| FR-4 Test alignment | T-002, T-005 | FC-DUP-001..004 |

## Functional Case Coverage

| Case | Requirement Coverage | Status before implementation |
|------|----------------------|------------------------------|
| FC-DUP-001 | Claude pricing/cache write | designed |
| FC-DUP-002 | OpenAI pricing/model matching | designed |
| FC-DUP-003 | unknown/negative safety | designed |
| FC-DUP-004 | quality and parity honesty | designed |

## Scope Control

- No transcript watcher.
- No storage writes.
- No usage UI/fleet merge.
- No billed-cost claims.
