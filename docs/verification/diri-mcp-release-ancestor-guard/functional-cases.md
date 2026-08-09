# Functional Cases: Diri MCP release_agent Ancestor Guard

```yaml
change_id: diri-mcp-release-ancestor-guard
beads: homie-4na
```

## FC-DMRG-001: Parent and ancestor release refused

- Command: `cargo test -p homie-cli --test mcp_release_ancestor_guard_cli -- --nocapture`
- Expected: child cannot release parent; grandchild cannot release root ancestor.

## FC-DMRG-002: Direct child release still works

- Command: `cargo test -p homie-cli --test mcp_release_agent_cli -- --nocapture`
- Expected: previous direct child release regression remains green.

## FC-DMRG-003: Quality gates

- Commands: check, clippy, diff, parity lock.

