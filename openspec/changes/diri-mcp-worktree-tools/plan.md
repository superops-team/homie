# OpenSpec Plan: Diri MCP Worktree Tools

```yaml
change_id: diri-mcp-worktree-tools
beads: homie-4wg
source_prd: prd-spec/features/diri-mcp-worktree-tools/2026-08-08-diri-mcp-worktree-tools-design.md
component_specs:
  - specs/mcp-automation/README.md
```

## 1. Scope

Add runtime-backed MCP `create_worktree`, `list_worktrees`, and `remove_worktree` by dispatching to existing Homie client/runtime worktree APIs.

## 2. Current State

- CLI `homie worktree create/list/remove` is implemented and tested.
- Runtime/client real git worktree APIs are implemented and tested.
- MCP descriptors list the worktree tools, but payload dispatch returns unsupported.

## 3. Target State

```text
MCP create_worktree(repo, branch?, base?)
  -> HomieClient::worktree_create

MCP list_worktrees(repo)
  -> HomieClient::worktree_list
  -> { worktrees: [...] }

MCP remove_worktree(repo, path, force?)
  -> HomieClient::worktree_remove
  -> { ok: true, path }
```

## 4. Module Changes

| Module | Change |
|--------|--------|
| `homie-cli` | Add MCP payload dispatch branches for the three worktree tools. |
| `homie-cli tests` | Add MCP stdio integration tests against a temporary real git repo. |
| `specs/mcp-automation` | Promote worktree tools from unsupported list to runtime-backed tools. |
| parity docs | Add API-003/API-004/GIT-002 evidence while preserving remaining partial rows. |

## 5. Verification

- FC-DMWT-001 create/list/remove MCP E2E.
- FC-DMWT-002 invalid params.
- FC-DMWT-003 CLI worktree regression.
- FC-DMWT-004 quality gates.
