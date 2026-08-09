# OpenSpec Plan: Diri MCP release_agent Owned-child Guard

```yaml
change_id: diri-mcp-release-owned-child-guard
beads: homie-al5
source_prd: prd-spec/features/diri-mcp-release-owned-child-guard/2026-08-08-diri-mcp-release-owned-child-guard-design.md
component_specs:
  - specs/mcp-automation/README.md
```

## 1. Scope

Implement the Diri MCP lineage permission rule for `release_agent`: an MCP caller may release only a direct child it spawned. The slice is limited to CLI/MCP runtime-backed behavior and tests.

## 2. Current State

- Direct child release exists.
- Self release is denied.
- Parent and ancestor release are denied.
- Sibling and unrelated release are not yet denied by an allow-list contract.

## 3. Target State

```text
release_agent(sessionId=target)
  -> relation = lineage_relation(caller, target)
  -> self: deny with existing self guard
  -> parent/ancestor: deny with existing upstream guard
  -> child: terminate target
  -> sibling/unrelated/other: deny before terminate_session
```

## 4. Module Changes

| Module | Change |
|--------|--------|
| `homie-cli` | Add owned-child allow-list to `release_agent` tool handler. |
| `homie-cli tests` | Add MCP stdio integration tests for sibling and unrelated deny without side effects. |
| `specs/mcp-automation` | Record `release_agent` permission contract. |
| parity docs | Update API-004/API-005 evidence after verification. |

## 5. Verification

- FC-DMRO-001 sibling deny.
- FC-DMRO-002 unrelated deny.
- FC-DMRO-003 existing release guards regression.
- FC-DMRO-004 quality gates.
