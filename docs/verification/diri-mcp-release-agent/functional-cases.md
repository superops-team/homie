# Functional Cases: Diri MCP release_agent

```yaml
change_id: diri-mcp-release-agent
beads: homie-3th
```

## FC-DMRA-001: Release direct child

- Command: `cargo test -p homie-cli --test mcp_release_agent_cli -- releases_direct_child --nocapture`
- Expected: parent spawns child, `release_agent` kills child and returns ok.

## FC-DMRA-002: Reject self release

- Command: `cargo test -p homie-cli --test mcp_release_agent_cli -- rejects_releasing_calling_session --nocapture`
- Expected: releasing caller returns safe JSON-RPC error.

## FC-DMRA-003: Quality gates

- Commands: check, clippy, diff, parity lock.

