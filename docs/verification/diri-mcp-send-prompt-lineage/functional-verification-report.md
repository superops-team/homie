# Functional Verification Report: Diri MCP send_prompt Lineage

```yaml
change_id: diri-mcp-send-prompt-lineage
beads: homie-ggi
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DMSP-001..002 | `cargo test -p homie-cli --test mcp_send_prompt_lineage_cli -- --nocapture` | failed: no relation/attribution and self-send was not rejected |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DMSP-001 | `cargo test -p homie-cli --test mcp_send_prompt_lineage_cli -- sibling_send_prompt_is_attributed --nocapture` | pass |
| FC-DMSP-002 | `cargo test -p homie-cli --test mcp_send_prompt_lineage_cli -- self_send_prompt_is_rejected --nocapture` | pass |
| FC-DMSP-003 | `cargo check -p homie-client -p homie-cli` | pass |
| FC-DMSP-003 | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DMSP-003 | `cargo fmt --all -- --check` | pass |
| FC-DMSP-003 | scoped `git diff --check` | pass |

## Scope Notes

- Implements self guard and sibling/unrelated attribution.
- Parent/direct child verbatim path remains supported.
- Recursive lineage and full permission profile enforcement remain pending.
