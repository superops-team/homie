# Functional Cases: Diri MCP get_artifacts Runtime

```yaml
change_id: diri-mcp-get-artifacts
beads: homie-pyt
```

## FC-DMGA-001: MCP get_artifacts reads real session output

- Command: `cargo test -p homie-cli --test mcp_get_artifacts_cli -- mcp_get_artifacts_reads_real_session_output --nocapture`
- Setup:
  - Create a real Homie session.
  - Send output containing a PR URL, preview URL, and ordinary link through `homie control-stdio --data-dir`.
  - Call `get_artifacts` through `homie mcp-stdio --data-dir`.
- Expected:
  - `artifacts` contains pull request, preview, and link entries.
  - `listeningPorts` contains the preview localhost port.
  - `sessionId` matches target session.

## FC-DMGA-002: Missing session id returns invalid params

- Command: `cargo test -p homie-cli --test mcp_get_artifacts_cli -- missing_session_id_returns_invalid_params --nocapture`
- Expected:
  - Missing `session_id/sessionId` returns JSON-RPC `-32602`.

## FC-DMGA-003: Scanner and ports regressions

- Commands:
  - `cargo test -p homie-runtime --test artifact_scanner`
  - `cargo test -p homie-cli --test ports_cli -- --nocapture`
- Expected:
  - Existing scanner and CLI ports behavior remain green.

## FC-DMGA-004: Quality gates

- Commands:
  - `cargo check -p homie-client -p homie-cli`
  - `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - scoped `git diff --check`
  - `make parity-lock`
- Expected: all commands pass.
