# Functional Cases: Diri MCP wait_for_agent Runtime

```yaml
change_id: diri-mcp-wait-for-agent
beads: homie-trk
```

## FC-DMWA-001: Wait until done after notify

- Command: `cargo test -p homie-cli --test mcp_wait_for_agent_cli -- waits_for_agent_until_done --nocapture`
- Setup:
  - Create a real Homie session with `homie session create --data-dir`.
  - Mark it idle through `homie notify --data-dir` with a Codex turn-complete payload.
  - Call MCP `wait_for_agent` through `homie mcp-stdio --data-dir`.
- Expected:
  - Result has `settled=true`, `timedOut=false`.
  - Result has `status="idle"` and `waitedFor="done"`.

## FC-DMWA-002: Timeout returns current status

- Command: `cargo test -p homie-cli --test mcp_wait_for_agent_cli -- timeout_returns_current_status --nocapture`
- Setup:
  - Create a running Homie session.
  - Call MCP `wait_for_agent` with `until:"done"` and `timeout_s:0`.
- Expected:
  - Result has `settled=false`, `timedOut=true`.
  - Result includes current `status="running"`.

## FC-DMWA-003: Wait until exited

- Command: `cargo test -p homie-cli --test mcp_wait_for_agent_cli -- waits_for_exited_agent --nocapture`
- Setup:
  - Create a Homie session.
  - Terminate it through `homie session kill --data-dir`.
  - Call MCP `wait_for_agent` with `until:"exited"`.
- Expected:
  - Result has `settled=true`, `timedOut=false`.
  - Result includes `status="exited"`.

## FC-DMWA-004: Existing children wait regression

- Command: `cargo test -p homie-cli --test mcp_wait_children_cli -- --nocapture`
- Expected:
  - Direct-child `wait_for_children` behavior remains green.

## FC-DMWA-005: Quality gates

- Commands:
  - `cargo check -p homie-client -p homie-cli`
  - `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - scoped `git diff --check`
  - `make parity-lock`
- Expected: all commands pass.
