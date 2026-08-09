# Functional Cases: Diri MCP send_prompt Lineage

```yaml
change_id: diri-mcp-send-prompt-lineage
beads: homie-ggi
```

## FC-DMSP-001: Sibling send is attributed

- Command: `cargo test -p homie-cli --test mcp_send_prompt_lineage_cli -- sibling_send_prompt_is_attributed --nocapture`
- Expected: sibling target output contains provenance header.

## FC-DMSP-002: Self send is rejected

- Command: `cargo test -p homie-cli --test mcp_send_prompt_lineage_cli -- self_send_prompt_is_rejected --nocapture`
- Expected: JSON-RPC error mentions self target refusal.

## FC-DMSP-003: Quality gates

- Commands: check, clippy, diff, parity lock.

