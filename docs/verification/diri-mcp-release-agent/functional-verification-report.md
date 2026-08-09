# Functional Verification Report: Diri MCP release_agent

```yaml
change_id: diri-mcp-release-agent
beads: homie-3th
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DMRA-001..002 | `cargo test -p homie-cli --test mcp_release_agent_cli -- --nocapture` | failed: `release_agent` returned unsupported JSON-RPC error |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DMRA-001 | `cargo test -p homie-cli --test mcp_release_agent_cli -- releases_direct_child --nocapture` | pass |
| FC-DMRA-002 | `cargo test -p homie-cli --test mcp_release_agent_cli -- rejects_releasing_calling_session --nocapture` | pass |
| FC-DMRA-003 | `cargo check -p homie-client -p homie-cli` | pass |
| FC-DMRA-003 | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DMRA-003 | `cargo fmt --all -- --check` | pass |
| FC-DMRA-003 | scoped `git diff --check` | pass |

## Scope Notes

- Direct child release and self-release guard are implemented.
- Parent/ancestor guard and full permission enforcement remain pending.
