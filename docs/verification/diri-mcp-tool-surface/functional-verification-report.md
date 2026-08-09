# Functional Verification Report: Diri MCP Runtime-backed Tool Surface

```yaml
change_id: diri-mcp-tool-surface
beads: homie-0pd
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DMTS-001..004 | `cargo test -p homie-cli --test mcp_stdio_runtime_cli -- --nocapture` | failed: `homie mcp-stdio` did not accept `--data-dir` |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DMTS-001 | `cargo test -p homie-cli --test mcp_stdio_runtime_cli -- lists_diri_runtime_tool_descriptors --nocapture` | pass |
| FC-DMTS-002 | `cargo test -p homie-cli --test mcp_stdio_runtime_cli -- runtime_backed_mcp_tools_list_status_and_read_output --nocapture` | pass |
| FC-DMTS-003 | `cargo test -p homie-cli --test mcp_stdio_runtime_cli -- runtime_backed_mcp_tools_send_prompt_and_spawn_agent --nocapture` | pass |
| FC-DMTS-004 | `cargo test -p homie-cli --test mcp_stdio_runtime_cli -- unsupported_future_tools_return_safe_errors --nocapture` | pass |
| FC-DMTS-005 | `cargo test -p homie-cli --test mcp_stdio_runtime_cli -- --nocapture` | pass: 4 tests |
| FC-DMTS-005 | `cargo test -p homie-cli --test mcp_stdio_cli -- --nocapture` | pass |
| FC-DMTS-005 | `cargo check -p homie-cli` | pass |
| FC-DMTS-005 | `cargo clippy -p homie-cli --all-targets -- -D warnings` | pass |

## Scope Notes

- `mcp-stdio --data-dir` now opens a real `HomieClient`.
- `list_agents/get_status/read_output/send_prompt/spawn_agent` use real runtime paths.
- Existing no-runtime mode remains intact for minimal usage.

## Remaining Gaps

- Full lineage storage and permission enforcement are not implemented.
- `wait_for_agent/release_agent/worktree/browser/test_run/list_children/wait_for_children` remain unsupported.
- `API-004` and `API-005` remain `partial`.
