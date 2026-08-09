# Release Readiness Report: Diri Automation Injection Plan

```yaml
change_id: diri-automation-injection-plan
beads: homie-8ib
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## 1. Source

- PRD: `prd-spec/features/diri-automation-injection-plan/2026-08-08-diri-automation-injection-plan-design.md`
- OpenSpec: `openspec/changes/diri-automation-injection-plan/`
- Functional cases: `docs/verification/diri-automation-injection-plan/functional-cases.md`
- Beads: `homie-8ib`

## 2. Delivered

- Pure automation spawn injection planner in `homie-orchestrator`.
- Claude hooks/MCP and Codex notify/MCP argv projection.
- Session id flag handling.
- Return-to-login-shell wrapping.
- Parity lock updated from `missing` to `partial` for `AUTO-001`.

## 3. Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Automation tests | `cargo test -p homie-orchestrator --test automation_injection -- --nocapture` | pass |
| Build | `cargo check -p homie-orchestrator` | pass |
| Lint | `cargo clippy -p homie-orchestrator --all-targets -- -D warnings` | pass |
| Diff hygiene | scoped `git diff --check` | pass |
| Parity lock | `make parity-lock` | pass_with_remaining_gaps |

## 4. Remaining Work

- MCP stdio server and tool protocol E2E.
- Forwarding automation.
- Runtime spawn consumption of the injection plan.
