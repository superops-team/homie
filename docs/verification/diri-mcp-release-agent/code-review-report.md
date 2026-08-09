# Code Review Report: Diri MCP release_agent

```yaml
change_id: diri-mcp-release-agent
beads: homie-3th
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| high | Safety | `crates/homie-cli/src/main.rs` | `release_agent` must not terminate the calling session. | fixed: self-release returns safe runtime error. |
| low | Scope | parity lock | Direct child release is not full lineage permission enforcement. | accepted: API-005 remains partial. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test mcp_release_agent_cli -- --nocapture` | pass |
| `cargo check -p homie-client -p homie-cli` | pass |
| `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |

