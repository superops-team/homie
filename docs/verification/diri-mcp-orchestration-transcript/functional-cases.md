# Functional Cases: Diri MCP Orchestration Transcript E2E

```yaml
change_id: diri-mcp-orchestration-transcript
beads: homie-3vh
```

## FC-DMOT-001: Spawn -> send -> wait -> read -> artifacts -> release

- Command: `cargo test -p homie-cli --test mcp_orchestration_transcript_cli -- --nocapture`
- Setup:
  - Create a parent Homie session.
  - Use real `homie mcp-stdio --data-dir --session-id <parent>` calls for every MCP tool.
- Expected:
  - `spawn_agent` creates child and stamps parent relation.
  - `send_prompt` writes to child.
  - `wait_for_agent` settles after child notify done.
  - `read_output` contains the preview URL.
  - `get_artifacts` returns listening port.
  - `release_agent` succeeds from parent to direct child.

## FC-DMOT-002: Quality gates

- Commands:
  - `cargo check -p homie-client -p homie-cli`
  - `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - scoped `git diff --check`
  - `make parity-lock`
- Expected: all commands pass.
