# OpenSpec Plan: Diri MCP wait_for_agent Runtime

```yaml
change_id: diri-mcp-wait-for-agent
beads: homie-trk
source_prd: prd-spec/features/diri-mcp-wait-for-agent/2026-08-08-diri-mcp-wait-for-agent-design.md
component_specs:
  - specs/mcp-automation/README.md
```

## 1. Scope

Add runtime-backed MCP `wait_for_agent` support for single session status waiting. This closes the non-screenshot orchestration gap in the Diri MCP flow without implementing browser/test_run or full event-bus long polling.

## 2. Current State

- `wait_for_agent` is listed as an MCP tool descriptor.
- Runtime-backed payload dispatch falls through to unsupported.
- `wait_for_children` already has bounded polling and Diri-style `done` status aliasing.

## 3. Target State

```text
wait_for_agent(session_id, until="done", timeout_s=600)
  -> read real Homie runtime status
  -> if status satisfies target, return settled=true
  -> if deadline passes, return settled=false/timedOut=true
  -> never panic or return unsupported for runtime-backed calls
```

## 4. Module Changes

| Module | Change |
|--------|--------|
| `homie-cli` | Add `wait_for_agent_payload` and dispatch it from `mcp_tool_payload`. |
| `homie-cli tests` | Add MCP stdio integration tests for done, timeout, and exited states. |
| `specs/mcp-automation` | Promote `wait_for_agent` from unsupported future tool to first-stage runtime-backed tool. |
| parity docs | Add API-004/API-005 evidence while keeping remaining full parity gaps partial. |

## 5. Verification

- FC-DMWA-001 wait until done.
- FC-DMWA-002 timeout returns current status.
- FC-DMWA-003 wait until exited.
- FC-DMWA-004 wait_for_children regression.
- FC-DMWA-005 quality gates.
