# OpenSpec Plan: Diri MCP get_artifacts Runtime

```yaml
change_id: diri-mcp-get-artifacts
beads: homie-pyt
source_prd: prd-spec/features/diri-mcp-get-artifacts/2026-08-08-diri-mcp-get-artifacts-design.md
component_specs:
  - specs/mcp-automation/README.md
```

## 1. Scope

Add runtime-backed MCP `get_artifacts` by dispatching to existing Homie session artifact scanning. The slice returns current scanner artifacts and listening ports only.

## 2. Current State

- Runtime scanner detects PR URLs, preview/local URLs, ordinary links, and localhost ports.
- Client exposes `scan_session_artifacts`.
- MCP descriptor lists `get_artifacts`, but payload dispatch returns unsupported.

## 3. Target State

```text
MCP get_artifacts(session_id/sessionId)
  -> HomieClient::scan_session_artifacts
  -> { sessionId, artifacts, listeningPorts }
```

## 4. Module Changes

| Module | Change |
|--------|--------|
| `homie-cli` | Add MCP payload dispatch for `get_artifacts`. |
| `homie-cli tests` | Add MCP stdio integration tests using real session output. |
| `specs/mcp-automation` | Promote `get_artifacts` from unsupported list to runtime-backed tools. |
| parity docs | Add API-004/ART-001/ART-002 evidence while preserving browser/test_run/PR live stats gaps. |

## 5. Verification

- FC-DMGA-001 get_artifacts E2E.
- FC-DMGA-002 invalid params.
- FC-DMGA-003 scanner/ports regressions.
- FC-DMGA-004 quality gates.
