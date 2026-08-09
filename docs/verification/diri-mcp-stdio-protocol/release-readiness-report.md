# Release Readiness Report: Diri MCP Stdio Protocol

```yaml
change_id: diri-mcp-stdio-protocol
beads: homie-0xb
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- `homie mcp-stdio` subcommand.
- JSON-RPC `tools/list`.
- JSON-RPC `tools/call` for `list_agents` and `whoami`.
- Unknown tool error handling.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| MCP stdio test | `cargo test -p homie-cli --test mcp_stdio_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-cli --all-targets -- -D warnings` | pass |
| Diff hygiene | scoped `git diff --check` | pass |
| Parity lock | `make parity-lock` | pass_with_remaining_gaps |

## Remaining Work

- Runtime-backed MCP tools.
- Full MCP stdio transcript E2E.
- Lineage/permission enforcement in tool execution.
