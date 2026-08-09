# Functional Cases: Diri MCP Worktree Tools

```yaml
change_id: diri-mcp-worktree-tools
beads: homie-4wg
```

## FC-DMWT-001: MCP create/list/remove real git worktree

- Command: `cargo test -p homie-cli --test mcp_worktree_tools_cli -- mcp_creates_lists_and_removes_real_worktree --nocapture`
- Setup:
  - Create a temporary git repo with one commit.
  - Call `create_worktree` through `homie mcp-stdio --data-dir`.
  - Call `list_worktrees` through MCP.
  - Call `remove_worktree` through MCP.
- Expected:
  - Created worktree path exists and branch is the requested branch.
  - List result contains the created worktree.
  - Remove result returns `ok=true`, and path no longer exists.

## FC-DMWT-002: Missing parameters return invalid params

- Command: `cargo test -p homie-cli --test mcp_worktree_tools_cli -- missing_parameters_return_invalid_params --nocapture`
- Expected:
  - `create_worktree` without `repo` returns JSON-RPC `-32602`.
  - `remove_worktree` without `path` returns JSON-RPC `-32602`.

## FC-DMWT-003: Existing CLI worktree regression

- Command: `cargo test -p homie-cli --test worktree_cli -- --nocapture`
- Expected: direct CLI create/list/remove remains green.

## FC-DMWT-004: Quality gates

- Commands:
  - `cargo check -p homie-client -p homie-cli`
  - `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - scoped `git diff --check`
  - `make parity-lock`
- Expected: all commands pass.
