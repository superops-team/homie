# Functional Verification Report: Diri MCP Worktree Tools

```yaml
change_id: diri-mcp-worktree-tools
beads: homie-4wg
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DMWT-001 | `cargo test -p homie-cli --test mcp_worktree_tools_cli -- --nocapture` | failed: worktree MCP tools returned unsupported and no tool content. |
| FC-DMWT-002 | `cargo test -p homie-cli --test mcp_worktree_tools_cli -- --nocapture` | failed: missing parameter case returned unsupported `-32601` instead of invalid params `-32602`. |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DMWT-001 | `cargo test -p homie-cli --test mcp_worktree_tools_cli -- mcp_creates_lists_and_removes_real_worktree --nocapture` | pass |
| FC-DMWT-002 | `cargo test -p homie-cli --test mcp_worktree_tools_cli -- missing_parameters_return_invalid_params --nocapture` | pass |
| FC-DMWT-001..002 | `cargo test -p homie-cli --test mcp_worktree_tools_cli -- --nocapture` | pass: 2 passed |
| FC-DMWT-003 | `cargo test -p homie-cli --test worktree_cli -- --nocapture` | pass: 1 passed |
| FC-DMWT-004 | `cargo check -p homie-client -p homie-cli` | pass |
| FC-DMWT-004 | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DMWT-004 | `cargo fmt --all -- --check` | pass |
| FC-DMWT-004 | scoped `git diff --check` | pass |
| FC-DMWT-004 | `make parity-lock` | pass; remaining unrelated partial rows listed honestly |

## Scope Notes

- Implements MCP dispatch to existing Homie client/runtime worktree APIs.
- Does not change worktree UI sheet behavior.
- Does not change worktree naming/path algorithm.
