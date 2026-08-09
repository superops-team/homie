# Release Readiness Report: Diri MCP get_artifacts Runtime

```yaml
change_id: diri-mcp-get-artifacts
beads: homie-pyt
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- Runtime-backed MCP `get_artifacts`.
- Diri-compatible `session_id` input plus Homie `sessionId` alias.
- Diri-named response fields: `artifacts` and `listeningPorts`.
- Real session-output E2E for PR URL, preview URL, ordinary link, and localhost port.
- Invalid params behavior for missing session id.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| MCP get_artifacts E2E | `cargo test -p homie-cli --test mcp_get_artifacts_cli -- --nocapture` | pass |
| Runtime scanner regression | `cargo test -p homie-runtime --test artifact_scanner` | pass |
| Ports CLI regression | `cargo test -p homie-cli --test ports_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |
| Parity lock | `make parity-lock` | pass |

## Remaining Work

- PR live stats enrichment under PR monitor lane.
- Browser/test_run tools.
- Full browser preview E2E and UI inspector interaction E2E.
