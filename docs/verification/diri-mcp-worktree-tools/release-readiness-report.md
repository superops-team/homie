# Release Readiness Report: Diri MCP Worktree Tools

```yaml
change_id: diri-mcp-worktree-tools
beads: homie-4wg
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- Runtime-backed MCP `create_worktree`.
- Runtime-backed MCP `list_worktrees`.
- Runtime-backed MCP `remove_worktree`.
- Diri-style MCP parameters `repo`, `branch`, `base`, `path`, `force`.
- Invalid params errors for missing required MCP arguments.
- Real git repo MCP E2E and existing CLI worktree regression.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| MCP worktree E2E | `cargo test -p homie-cli --test mcp_worktree_tools_cli -- --nocapture` | pass |
| CLI worktree regression | `cargo test -p homie-cli --test worktree_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |
| Parity lock | `make parity-lock` | pass |

## Remaining Work

- Worktree app sheet full interaction E2E.
- Cleanup suggestion workflow E2E.
- Browser/test_run and full MCP transcript E2E.
