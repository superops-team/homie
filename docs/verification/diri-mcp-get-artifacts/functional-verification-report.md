# Functional Verification Report: Diri MCP get_artifacts Runtime

```yaml
change_id: diri-mcp-get-artifacts
beads: homie-pyt
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DMGA-001 | `cargo test -p homie-cli --test mcp_get_artifacts_cli -- --nocapture` | failed: `get_artifacts` returned unsupported and no MCP tool content. |
| FC-DMGA-002 | `cargo test -p homie-cli --test mcp_get_artifacts_cli -- --nocapture` | failed: missing session id returned unsupported `-32601` instead of invalid params `-32602`. |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DMGA-001 | `cargo test -p homie-cli --test mcp_get_artifacts_cli -- mcp_get_artifacts_reads_real_session_output --nocapture` | pass |
| FC-DMGA-002 | `cargo test -p homie-cli --test mcp_get_artifacts_cli -- missing_session_id_returns_invalid_params --nocapture` | pass |
| FC-DMGA-001..002 | `cargo test -p homie-cli --test mcp_get_artifacts_cli -- --nocapture` | pass: 2 passed |
| FC-DMGA-003 | `cargo test -p homie-runtime --test artifact_scanner` | pass: 2 passed |
| FC-DMGA-003 | `cargo test -p homie-cli --test ports_cli -- --nocapture` | pass: 2 passed |
| FC-DMGA-004 | `cargo check -p homie-client -p homie-cli` | pass |
| FC-DMGA-004 | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DMGA-004 | `cargo fmt --all -- --check` | pass after running `cargo fmt --all` for the new test file |
| FC-DMGA-004 | scoped `git diff --check` | pass |
| FC-DMGA-004 | `make parity-lock` | pass; remaining unrelated partial rows listed honestly |

## Scope Notes

- Implements MCP dispatch to existing Homie session artifact scanner.
- Does not implement PR live stats.
- Does not implement browser/test_run.
