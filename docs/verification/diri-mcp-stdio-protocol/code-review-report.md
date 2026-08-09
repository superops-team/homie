# Code Review Report: Diri MCP Stdio Protocol

```yaml
change_id: diri-mcp-stdio-protocol
beads: homie-0xb
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| medium | Correctness | `homie-cli` | `mcp-stdio` subcommand did not exist, so MCP clients had no stdio protocol entry. | fixed: added newline JSON-RPC handler for `tools/list` and minimal `tools/call`. |
| low | Scope | parity lock | Minimal stdio tools do not complete full MCP orchestration. | accepted: `API-004` remains partial. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test mcp_stdio_cli -- --nocapture` | pass |
| `cargo check -p homie-cli` | pass |
| `cargo clippy -p homie-cli --all-targets -- -D warnings` | pass |
| scoped `git diff --check` | pass |
| `make parity-lock` | pass_with_remaining_gaps |

