# Alignment Report: Diri MCP Runtime-backed Tool Surface

```yaml
change_id: diri-mcp-tool-surface
beads: homie-0pd
```

## PRD to OpenSpec Alignment

| PRD | Tasks | Functional Cases | Status |
|-----|-------|------------------|--------|
| FR-1 runtime-backed MCP context | T-001, T-002 | FC-DMTS-001, FC-DMTS-002 | aligned |
| FR-2 tool descriptor | T-003, T-004 | FC-DMTS-001 | aligned |
| FR-3 tool call behavior | T-003 | FC-DMTS-002, FC-DMTS-003 | aligned |
| FR-4 safe error handling | T-002, T-004 | FC-DMTS-004 | aligned |

## Scope Guard

This slice advances API-004 but keeps API-004/API-005 partial until lineage permission, children/wait/release, worktree/browser/test_run, and full MCP transcript E2E are implemented.
