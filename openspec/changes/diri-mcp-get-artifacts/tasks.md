# OpenSpec Tasks: Diri MCP get_artifacts Runtime

| Task | Description | Acceptance | Cases |
|------|-------------|------------|-------|
| T-001 | Update MCP automation spec for `get_artifacts` | Spec lists `get_artifacts` as runtime-backed and notes PR live stats are out of scope | FC-DMGA-004 |
| T-002 | Add RED MCP get_artifacts E2E tests | Tests fail before implementation because tool is unsupported | FC-DMGA-001, FC-DMGA-002 |
| T-003 | Implement MCP payload dispatch | `get_artifacts` calls `HomieClient::scan_session_artifacts` and returns artifacts/listeningPorts | FC-DMGA-001 |
| T-004 | Preserve invalid params behavior | Missing session id returns JSON-RPC `-32602` | FC-DMGA-002 |
| T-005 | Run scanner/ports regressions and quality gates | Existing scanner/ports tests and quality gates pass | FC-DMGA-003, FC-DMGA-004 |
| T-006 | Update parity lock and close Bead | API-004/ART-001/ART-002 evidence references MCP get_artifacts while remaining rows stay partial | FC-DMGA-004 |
