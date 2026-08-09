# Functional Verification Report: Diri MCP wait_for_agent Runtime

```yaml
change_id: diri-mcp-wait-for-agent
beads: homie-trk
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DMWA-001..003 | `cargo test -p homie-cli --test mcp_wait_for_agent_cli -- --nocapture` | failed: `wait_for_agent` did not return MCP tool content because it was still unsupported. |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DMWA-001 | `cargo test -p homie-cli --test mcp_wait_for_agent_cli -- waits_for_agent_until_done --nocapture` | pass |
| FC-DMWA-002 | `cargo test -p homie-cli --test mcp_wait_for_agent_cli -- timeout_returns_current_status --nocapture` | pass |
| FC-DMWA-003 | `cargo test -p homie-cli --test mcp_wait_for_agent_cli -- waits_for_exited_agent --nocapture` | pass |
| FC-DMWA-001..003 | `cargo test -p homie-cli --test mcp_wait_for_agent_cli -- --nocapture` | pass: 3 passed |
| FC-DMWA-004 | `cargo test -p homie-cli --test mcp_wait_children_cli -- --nocapture` | pass: 2 passed |
| FC-DMWA-005 | `cargo check -p homie-client -p homie-cli` | pass |
| FC-DMWA-005 | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DMWA-005 | `cargo fmt --all -- --check` | pass |
| FC-DMWA-005 | scoped `git diff --check` | pass |
| FC-DMWA-005 | `make parity-lock` | pass; remaining unrelated partial rows listed honestly |

## Scope Notes

- Implements runtime-backed status polling for one session.
- Does not implement Diri's daemon-side `events.wait` long-poll optimization.
- Does not implement browser/test_run.
