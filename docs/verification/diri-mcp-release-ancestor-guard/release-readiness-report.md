# Release Readiness Report: Diri MCP release_agent Ancestor Guard

```yaml
change_id: diri-mcp-release-ancestor-guard
beads: homie-4na
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- MCP `release_agent` now refuses to terminate the caller's direct parent.
- MCP `release_agent` now refuses to terminate any ancestor above the caller.
- Direct child release and self-release guard behavior remain intact.
- `API-005` parity evidence is updated without marking full lineage parity complete.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Ancestor guard functional test | `cargo test -p homie-cli --test mcp_release_ancestor_guard_cli -- --nocapture` | pass |
| Direct release regression | `cargo test -p homie-cli --test mcp_release_agent_cli -- --nocapture` | pass |
| Lint | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |

## Remaining Work

- Full recursive MCP permission matrix.
- Event-backed lineage waiting/release E2E beyond direct release and ancestor refusal.
- UI-visible lineage controls and audit trail.
