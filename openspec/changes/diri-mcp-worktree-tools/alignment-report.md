# Alignment Report: Diri MCP Worktree Tools

```yaml
change_id: diri-mcp-worktree-tools
beads: homie-4wg
status: aligned
checked_at: 2026-08-08
```

## PRD to Task Mapping

| PRD Requirement | OpenSpec Task | Verification |
|-----------------|---------------|--------------|
| FR-1 create_worktree runtime-backed | T-002, T-003 | FC-DMWT-001 |
| FR-2 list_worktrees runtime-backed | T-002, T-003 | FC-DMWT-001 |
| FR-3 remove_worktree runtime-backed | T-002, T-003 | FC-DMWT-001 |
| FR-4 parameter errors | T-004 | FC-DMWT-002 |

## Functional Case Coverage

| Case | Requirement Coverage | Status before implementation |
|------|----------------------|------------------------------|
| FC-DMWT-001 | create/list/remove E2E | designed |
| FC-DMWT-002 | invalid params | designed |
| FC-DMWT-003 | CLI regression | designed |
| FC-DMWT-004 | quality gates and parity honesty | designed |

## Component Spec Impact

- `specs/mcp-automation/README.md` owns MCP automation tool contracts.
- This change updates the MCP automation spec to mark worktree tools as runtime-backed.

## Scope Control

- No UI worktree sheet changes.
- No worktree path algorithm changes.
- No storage schema changes.
- No browser/test_run work.
