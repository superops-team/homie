# Release Readiness Report: Diri MCP release_agent

```yaml
change_id: diri-mcp-release-agent
beads: homie-3th
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- MCP `release_agent`.
- Direct child termination.
- Self-release guard.
- CLI E2E for child release and self-release refusal.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| MCP release agent | `cargo test -p homie-cli --test mcp_release_agent_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |

## Remaining Work

- Parent/ancestor guard.
- Recursive lineage.
- Full permission enforcement.
