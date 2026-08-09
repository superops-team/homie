# Functional Cases: Diri Automation Injection Plan

```yaml
change_id: diri-automation-injection-plan
beads: homie-8ib
```

## FC-DAIP-001: Automation injection planner

- Command: `cargo test -p homie-orchestrator --test automation_injection -- --nocapture`
- Expected:
  - Base env includes Homie session id, socket, CLI path and PATH.
  - Claude hooks/MCP injection appends expected args.
  - Codex notify/MCP injection appends expected args.
  - session id flag is included and returned as agent session id.
  - return-to-login-shell wraps argv in shell `-i -l -c`.

## FC-DAIP-002: Build

- Command: `cargo check -p homie-orchestrator`
- Expected: exit code 0.

## FC-DAIP-003: Lint

- Command: `cargo clippy -p homie-orchestrator --all-targets -- -D warnings`
- Expected: exit code 0.

## FC-DAIP-004: Hygiene and parity lock

- Commands:
  - `git diff --check -- crates/homie-orchestrator prd-spec/features/diri-automation-injection-plan openspec/changes/diri-automation-injection-plan docs/verification/diri-automation-injection-plan`
  - `make parity-lock`
- Expected:
  - diff check passes.
  - `AUTO-001` may move to partial after evidence, not implemented.
