# OpenSpec Tasks: Diri MCP Worktree Tools

| Task | Description | Acceptance | Cases |
|------|-------------|------------|-------|
| T-001 | Update MCP automation spec for worktree tools | Spec lists `create_worktree`, `list_worktrees`, `remove_worktree` as runtime-backed tools and removes them from unsupported list | FC-DMWT-004 |
| T-002 | Add RED MCP worktree E2E tests | Tests fail before implementation because tools are unsupported | FC-DMWT-001, FC-DMWT-002 |
| T-003 | Implement MCP payload dispatch | Three tools call `HomieClient::worktree_*` and return structured results | FC-DMWT-001 |
| T-004 | Preserve invalid params behavior | Missing `repo`/`path` returns JSON-RPC `-32602` | FC-DMWT-002 |
| T-005 | Run CLI worktree regression and quality gates | CLI regression and all quality gates pass | FC-DMWT-003, FC-DMWT-004 |
| T-006 | Update parity lock and close Bead | API-003/API-004/GIT-002 evidence references MCP worktree test while rows remain partial for UI/E2E gaps | FC-DMWT-004 |
