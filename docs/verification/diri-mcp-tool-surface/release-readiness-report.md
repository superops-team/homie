# Release Readiness Report: Diri MCP Runtime-backed Tool Surface

```yaml
change_id: diri-mcp-tool-surface
beads: homie-0pd
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- `homie mcp-stdio --data-dir`.
- Runtime-backed MCP tools: `list_agents`, `whoami`, `get_status`, `read_output`, `send_prompt`, `spawn_agent`.
- Existing no-runtime MCP mode preserved.
- Safe unsupported errors for future tools.
- MCP component spec and parity lock updated.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| MCP runtime tools | `cargo test -p homie-cli --test mcp_stdio_runtime_cli -- --nocapture` | pass |
| MCP no-runtime regression | `cargo test -p homie-cli --test mcp_stdio_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-cli --all-targets -- -D warnings` | pass |

## Beads

- `homie-0pd` is complete for this bounded slice.
- Parent protocol/MCP group remains open for broader CLI/MCP/lineage parity.

## Remaining Work

- Full MCP transcript E2E.
- Lineage storage, children/wait tools, and permission enforcement.
- Worktree/browser/test_run/release/wait tool implementations.
