# Alignment Report: Diri MCP wait_for_agent Runtime

```yaml
change_id: diri-mcp-wait-for-agent
beads: homie-trk
status: aligned
checked_at: 2026-08-08
```

## PRD to Task Mapping

| PRD Requirement | OpenSpec Task | Verification |
|-----------------|---------------|--------------|
| FR-1 Runtime-backed wait_for_agent | T-002, T-003 | FC-DMWA-001, FC-DMWA-002, FC-DMWA-003 |
| FR-2 Diri parameter compatibility | T-002, T-003 | FC-DMWA-001 uses `session_id` and `timeout_s` |
| FR-3 Wait semantics | T-003 | FC-DMWA-001 done, FC-DMWA-003 exited |
| FR-4 Structured timeout | T-003 | FC-DMWA-002 |

## Functional Case Coverage

| Case | Requirement Coverage | Status before implementation |
|------|----------------------|------------------------------|
| FC-DMWA-001 | done/idle wait path | designed |
| FC-DMWA-002 | timeout path | designed |
| FC-DMWA-003 | exited path | designed |
| FC-DMWA-004 | wait_for_children regression | designed |
| FC-DMWA-005 | quality gates and parity honesty | designed |

## Component Spec Impact

- `specs/mcp-automation/README.md` owns MCP stdio tool contracts and lineage/permission behavior.
- This change updates that spec to state the first-stage `wait_for_agent` status wait contract.

## Scope Control

- No browser/test_run implementation.
- No UI work.
- No event-bus long-poll rewrite.
- No changes to session status reducer semantics.
