# Functional Cases: Diri Agent Readiness CLI

```yaml
change_id: diri-agent-readiness-cli
beads: homie-8ua
```

## FC-DARC-001: CLI readiness fixture

- Command: `cargo test -p homie-cli --test agent_readiness_cli -- --nocapture`
- Expected: fake `codex` binary is available; fake `claude` missing binary is unavailable; `shell` without binary is omitted.

## FC-DARC-002: Quality gates

- Commands:
  - `cargo check -p homie-agents -p homie-cli`
  - `cargo clippy -p homie-agents -p homie-cli --all-targets -- -D warnings`
  - scoped `git diff --check`
  - `make parity-lock`

