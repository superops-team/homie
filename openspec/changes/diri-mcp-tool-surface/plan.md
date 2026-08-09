# OpenSpec Plan: Diri MCP Runtime-backed Tool Surface

```yaml
change_id: diri-mcp-tool-surface
beads: homie-0pd
prd: prd-spec/features/diri-mcp-tool-surface/2026-08-08-diri-mcp-tool-surface-design.md
```

## Scope

Deliver a bounded Diri MCP parity slice that upgrades `homie mcp-stdio` from static minimal responses to runtime-backed tools for sessions.

## Module Boundaries

| Layer | Files | Responsibility |
|-------|-------|----------------|
| CLI | `crates/homie-cli/src/main.rs` | MCP stdio args, context, tool dispatch |
| Runtime access | `homie-client` existing APIs | list/status/output/send/spawn |
| Evidence | `docs/verification/diri-mcp-tool-surface/` | cases, verification, review |
| Contract | `specs/mcp-automation/README.md` | first-stage runtime-backed tool contract |

## Non-goals

- Full lineage permission enforcement.
- Children/wait/release/worktree/browser/test_run tools.
- UI or screenshots.

## Acceptance

Runtime-backed `mcp-stdio --data-dir` test passes and parity lock records the new API-004 evidence while status remains `partial`.
