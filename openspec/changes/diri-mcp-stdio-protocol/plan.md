# OpenSpec Plan: Diri MCP Stdio Protocol

> Change ID: `diri-mcp-stdio-protocol`  
> Beads: `homie-0xb`

## Scope

Add a minimal MCP stdio JSON-RPC protocol entry to `homie-cli`.

## Functional Cases

| Case | Command |
|------|---------|
| FC-DMSP-001 | `cargo test -p homie-cli --test mcp_stdio_cli -- --nocapture` |
| FC-DMSP-002 | `cargo check -p homie-cli` |
| FC-DMSP-003 | `cargo clippy -p homie-cli --all-targets -- -D warnings` |
| FC-DMSP-004 | scoped `git diff --check`; `make parity-lock` |

