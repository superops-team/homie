# Alignment Report: Diri host.locate_repo

```yaml
change_id: diri-host-locate-repo
beads: homie-37j
```

## PRD to OpenSpec Alignment

| PRD | OpenSpec Task | Functional Case | Status |
|-----|---------------|-----------------|--------|
| FR-1 protocol fields | T-001 | FC-DHLR-001 | aligned |
| FR-2 origin discovery | T-002 | FC-DHLR-002, FC-DHLR-004 | aligned |
| FR-3 candidate matching | T-003 | FC-DHLR-002, FC-DHLR-003 | aligned |
| FR-4 storage-backed handler | T-004 | FC-DHLR-005 | aligned |
| FR-5 CLI entrypoint | T-005 | FC-DHLR-005 | aligned |

## Non-goals Preserved

- No real SSH/node locate execution in this change.
- No UI work or screenshot evidence.
- No provider credential handling beyond preserving secretless logs/evidence.

## Verification Mapping

The functional cases are defined before implementation in `docs/verification/diri-host-locate-repo/functional-cases.md`. Every implementation task has at least one executable case.
