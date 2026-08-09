# Release Readiness Report: Diri MCP Orchestration Transcript E2E

```yaml
change_id: diri-mcp-orchestration-transcript
beads: homie-3vh
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- Real MCP stdio transcript E2E for the Diri orchestration flow.
- Coverage for `spawn_agent`, `send_prompt`, `wait_for_agent`, `read_output`, `get_artifacts`, and `release_agent`.
- Child output and artifact/port assertions.
- Explicit cleanup of child and parent sessions.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| MCP transcript E2E | `cargo test -p homie-cli --test mcp_orchestration_transcript_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |

## Remaining Work

- Browser/test_run E2E.
- PR live stats enrichment.
- UI inspector/browser preview E2E.
