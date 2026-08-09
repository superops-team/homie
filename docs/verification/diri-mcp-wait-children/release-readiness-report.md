# Release Readiness Report: Diri MCP wait_for_children

```yaml
change_id: diri-mcp-wait-children
beads: homie-ne8
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- MCP `wait_for_children` direct-child polling.
- `until` modes: `settled`, `done`, `exited`.
- `timeout_s` support.
- CLI E2E for no-child immediate settle and child idle completion.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| MCP wait children | `cargo test -p homie-cli --test mcp_wait_children_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |

## Remaining Work

- Recursive descendants.
- Event-driven wait.
- release_agent lineage guard.
- Permission enforcement.
