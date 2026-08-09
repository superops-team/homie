# OpenSpec Tasks: Diri MCP Orchestration Transcript E2E

| Task | Description | Acceptance | Cases |
|------|-------------|------------|-------|
| T-001 | Add transcript E2E test | Test drives real `homie mcp-stdio --data-dir --session-id` through spawn/send/wait/read/artifacts/release | FC-DMOT-001 |
| T-002 | Fix only exposed tool gaps if test fails | Any implementation change must be scoped to the failing tool and documented | FC-DMOT-001 |
| T-003 | Run quality gates | check/clippy/fmt/diff/parity all pass | FC-DMOT-002 |
| T-004 | Update parity evidence and close Bead | API-004 evidence no longer says full transcript E2E pending; browser/test_run remain pending | FC-DMOT-002 |
