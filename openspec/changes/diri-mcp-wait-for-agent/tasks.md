# OpenSpec Tasks: Diri MCP wait_for_agent Runtime

| Task | Description | Acceptance | Cases |
|------|-------------|------------|-------|
| T-001 | Update MCP automation spec for `wait_for_agent` runtime contract | Spec lists `wait_for_agent` as runtime-backed and documents status wait semantics | FC-DMWA-005 |
| T-002 | Add RED integration tests for `wait_for_agent` | Tests fail before implementation because tool is unsupported | FC-DMWA-001, FC-DMWA-002, FC-DMWA-003 |
| T-003 | Implement `wait_for_agent_payload` | Runtime-backed MCP calls return settled/timedOut/status/waitedFor using real runtime state | FC-DMWA-001, FC-DMWA-002, FC-DMWA-003 |
| T-004 | Run regression for `wait_for_children` | Existing direct-child wait behavior stays green | FC-DMWA-004 |
| T-005 | Run quality gates and record evidence | check/clippy/fmt/diff/parity all pass | FC-DMWA-005 |
| T-006 | Update parity lock and close Bead | API-004/API-005 evidence includes `mcp_wait_for_agent_cli`; rows remain partial for remaining E2E gaps | FC-DMWA-005 |
