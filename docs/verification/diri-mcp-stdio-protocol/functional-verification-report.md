# Functional Verification Report: Diri MCP Stdio Protocol

```yaml
change_id: diri-mcp-stdio-protocol
beads: homie-0xb
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Summary

This slice advances `API-004` with a minimal MCP stdio JSON-RPC entry:

- `homie mcp-stdio` reads newline-delimited JSON-RPC from stdin.
- `tools/list` returns tool descriptors.
- `tools/call` supports `list_agents` and `whoami`.
- Unknown tools return JSON-RPC error `-32601`.

`API-004` remains partial until the full MCP tool set and runtime-backed tool execution E2E are implemented.

## Results

| Case | Command | Result |
|------|---------|--------|
| FC-DMSP-001 | `cargo test -p homie-cli --test mcp_stdio_cli -- --nocapture` | pass |
| FC-DMSP-002 | `cargo check -p homie-cli` | pass |
| FC-DMSP-003 | `cargo clippy -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DMSP-004 | scoped `git diff --check` | pass |
| FC-DMSP-004 | `make parity-lock` | pass_with_remaining_gaps |

