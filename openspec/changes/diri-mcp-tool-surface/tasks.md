# OpenSpec Tasks: Diri MCP Runtime-backed Tool Surface

| Task | Description | Cases |
|------|-------------|-------|
| T-001 | Add `mcp-stdio --data-dir/--session-id/--parent-session-id` args | FC-DMTS-001 |
| T-002 | Add MCP runtime context and keep no-runtime fallback | FC-DMTS-001, FC-DMTS-002 |
| T-003 | Implement runtime-backed `list_agents`, `whoami`, `get_status`, `read_output`, `send_prompt`, `spawn_agent` | FC-DMTS-002, FC-DMTS-003 |
| T-004 | Keep unsupported future tools explicit and safe | FC-DMTS-004 |
| T-005 | Update MCP spec, parity lock, and verification reports | FC-DMTS-005 |

## Task to Requirement Mapping

| PRD Requirement | Tasks |
|-----------------|-------|
| FR-1 | T-001, T-002 |
| FR-2 | T-003, T-004 |
| FR-3 | T-003 |
| FR-4 | T-002, T-004 |
