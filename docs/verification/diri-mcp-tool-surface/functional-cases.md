# Functional Cases: Diri MCP Runtime-backed Tool Surface

```yaml
change_id: diri-mcp-tool-surface
beads: homie-0pd
```

## FC-DMTS-001: Tool discovery

- Command: `cargo test -p homie-cli --test mcp_stdio_runtime_cli -- lists_diri_runtime_tool_descriptors --nocapture`
- Expected:
  - `tools/list` includes `spawn_agent`, `list_agents`, `get_status`, `send_prompt`, `read_output`, `whoami`.
  - Existing no-runtime `mcp_stdio_cli` still passes.

## FC-DMTS-002: Runtime-backed list/status/read

- Command: `cargo test -p homie-cli --test mcp_stdio_runtime_cli -- runtime_backed_mcp_tools_list_status_and_read_output --nocapture`
- Expected:
  - A real session created by CLI appears in `list_agents`.
  - `get_status` returns that session status.
  - `read_output` returns real shell prompt/output text.

## FC-DMTS-003: Runtime-backed send and spawn

- Command: `cargo test -p homie-cli --test mcp_stdio_runtime_cli -- runtime_backed_mcp_tools_send_prompt_and_spawn_agent --nocapture`
- Expected:
  - `send_prompt` writes to a live session and output becomes readable.
  - `spawn_agent` creates a second runtime-backed session.

## FC-DMTS-004: Safe unsupported tools

- Command: `cargo test -p homie-cli --test mcp_stdio_runtime_cli -- unsupported_future_tools_return_safe_errors --nocapture`
- Expected:
  - Future tools not implemented in this slice return explicit safe error.

## FC-DMTS-005: Quality and parity gates

- Commands:
  - `cargo test -p homie-cli --test mcp_stdio_runtime_cli -- --nocapture`
  - `cargo test -p homie-cli --test mcp_stdio_cli -- --nocapture`
  - `cargo check -p homie-cli`
  - `cargo clippy -p homie-cli --all-targets -- -D warnings`
  - scoped `git diff --check`
  - `make parity-lock`
- Expected: all pass; API-004 remains partial with stronger evidence.
