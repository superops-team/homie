# Release Readiness Report: Diri MCP send_prompt Lineage

```yaml
change_id: diri-mcp-send-prompt-lineage
beads: homie-ggi
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- MCP `send_prompt` self-send guard.
- Sibling/unrelated provenance header.
- `relation` and `attributed` tool response fields.
- CLI E2E for sibling attribution and self-send refusal.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| MCP send prompt lineage | `cargo test -p homie-cli --test mcp_send_prompt_lineage_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |

## Remaining Work

- Recursive ancestor/descendant relation.
- Full permission profile enforcement.
