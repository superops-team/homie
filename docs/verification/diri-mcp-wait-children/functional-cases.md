# Functional Cases: Diri MCP wait_for_children

```yaml
change_id: diri-mcp-wait-children
beads: homie-ne8
```

## FC-DMWC-001: Wait direct child until done

- Command: `cargo test -p homie-cli --test mcp_wait_children_cli -- waits_for_child_until_done --nocapture`
- Expected: parent spawns child, child becomes idle through notify, `wait_for_children` returns settled true.

## FC-DMWC-002: Empty children settles immediately

- Command: `cargo test -p homie-cli --test mcp_wait_children_cli -- no_children_settles_immediately --nocapture`
- Expected: no child sessions returns settled true and empty children.

## FC-DMWC-003: Quality gates

- Commands: check, clippy, diff, parity lock.

