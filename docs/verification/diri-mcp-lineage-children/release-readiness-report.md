# Release Readiness Report: Diri MCP Lineage Children

```yaml
change_id: diri-mcp-lineage-children
beads: homie-s82
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- Storage parent linkage API.
- Client parent/children APIs.
- MCP `spawn_agent` parent stamping when `--session-id` is set.
- MCP `list_children` direct child output.
- CLI E2E for parent spawn and child listing.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| MCP lineage children | `cargo test -p homie-cli --test mcp_lineage_children_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-storage -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-storage -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |

## Remaining Work

- `wait_for_children`.
- `release_agent` lineage guard.
- Recursive descendants and permission enforcement.
