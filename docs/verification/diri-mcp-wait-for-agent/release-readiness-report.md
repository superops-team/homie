# Release Readiness Report: Diri MCP wait_for_agent Runtime

```yaml
change_id: diri-mcp-wait-for-agent
beads: homie-trk
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- Runtime-backed MCP `wait_for_agent`.
- Diri parameter compatibility for `session_id` and `timeout_s`, plus Homie camelCase aliases.
- `done` status target mapped to `idle` or `exited`.
- Structured timeout response with current status.
- Regression coverage for existing `wait_for_children`.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| wait_for_agent runtime E2E | `cargo test -p homie-cli --test mcp_wait_for_agent_cli -- --nocapture` | pass |
| wait_for_children regression | `cargo test -p homie-cli --test mcp_wait_children_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |
| Parity lock | `make parity-lock` | pass |

## Remaining Work

- Diri-style daemon-side `events.wait` long-poll optimization.
- Browser/test_run MCP tools.
- Full transcript/artifact E2E across the MCP orchestration flow.
