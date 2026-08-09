# OpenSpec Plan: Diri MCP Orchestration Transcript E2E

```yaml
change_id: diri-mcp-orchestration-transcript
beads: homie-3vh
source_prd: prd-spec/features/diri-mcp-orchestration-transcript/2026-08-08-diri-mcp-orchestration-transcript-design.md
```

## 1. Scope

Add a real MCP stdio transcript E2E test for the Diri orchestration flow. This is a verification slice, not a new product surface.

## 2. Current State

- Individual MCP tools have focused tests.
- Parity lock still records full transcript E2E as pending.

## 3. Target State

```text
parent MCP identity
  -> spawn_agent child
  -> send_prompt child
  -> notify child done
  -> wait_for_agent child done
  -> read_output child
  -> get_artifacts child
  -> release_agent child
```

## 4. Verification

- FC-DMOT-001 transcript E2E.
- FC-DMOT-002 quality gates.
