# Functional Cases: Diri MCP Lineage Children

```yaml
change_id: diri-mcp-lineage-children
beads: homie-s82
```

## FC-DMLC-001: MCP spawn stamps parent and list_children returns child

- Command: `cargo test -p homie-cli --test mcp_lineage_children_cli -- --nocapture`
- Expected: `spawn_agent` under `--session-id <parent>` creates child; `list_children` returns exactly that direct child.

## FC-DMLC-002: Quality gates

- Commands: check, clippy, diff, parity lock.

