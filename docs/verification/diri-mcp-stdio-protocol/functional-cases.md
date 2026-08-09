# Functional Cases: Diri MCP Stdio Protocol

```yaml
change_id: diri-mcp-stdio-protocol
beads: homie-0xb
```

## FC-DMSP-001: MCP stdio line handler

- Command: `cargo test -p homie-cli --test mcp_stdio_cli -- --nocapture`
- Expected:
  - `tools/list` returns MCP tools.
  - `tools/call` `list_agents` returns ok payload.
  - `tools/call` `whoami` returns identity payload.
  - Unknown tool returns JSON-RPC error.

## FC-DMSP-002: Build

- Command: `cargo check -p homie-cli`

## FC-DMSP-003: Lint

- Command: `cargo clippy -p homie-cli --all-targets -- -D warnings`

## FC-DMSP-004: Hygiene and parity lock

- Commands:
  - `git diff --check -- crates/homie-cli prd-spec/features/diri-mcp-stdio-protocol openspec/changes/diri-mcp-stdio-protocol docs/verification/diri-mcp-stdio-protocol`
  - `make parity-lock`
