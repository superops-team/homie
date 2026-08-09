# Functional Verification Report: Diri Automation Injection Plan

```yaml
change_id: diri-automation-injection-plan
beads: homie-8ib
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Summary

This slice advances `AUTO-001` from missing to partial:

- `homie-orchestrator` now builds a pure spawn injection plan from `AgentManifest`.
- Base env includes session id, socket path, CLI path and login PATH.
- Claude hooks/MCP and Codex notify/MCP argv injection are modeled.
- Agent session id flags are included and returned.
- `return_to_login_shell` wrapping is modeled.

Full MCP stdio, forwarding and live automation E2E remain pending.

## Results

| Case | Command | Result |
|------|---------|--------|
| FC-DAIP-001 | `cargo test -p homie-orchestrator --test automation_injection -- --nocapture` | pass |
| FC-DAIP-002 | `cargo check -p homie-orchestrator` | pass |
| FC-DAIP-003 | `cargo clippy -p homie-orchestrator --all-targets -- -D warnings` | pass |
| FC-DAIP-004 | scoped `git diff --check` | pass |
| FC-DAIP-004 | `make parity-lock` | pass_with_remaining_gaps |

