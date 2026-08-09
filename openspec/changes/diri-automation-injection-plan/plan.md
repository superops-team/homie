# OpenSpec Plan: Diri Automation Injection Plan

> Change ID: `diri-automation-injection-plan`  
> Beads: `homie-8ib`

## Scope

Add a pure, testable automation injection planner to `homie-orchestrator`. It consumes `homie-agents::AgentManifest` and emits argv/env metadata that runtime can later use.

## Modules

| Module | Change |
|--------|--------|
| `crates/homie-orchestrator/src/lib.rs` | Add automation plan structs and `build_spawn_plan` |
| `crates/homie-orchestrator/tests/automation_injection.rs` | Diri-equivalent injection tests |

## Functional Cases

| Case | Command |
|------|---------|
| FC-DAIP-001 | `cargo test -p homie-orchestrator --test automation_injection -- --nocapture` |
| FC-DAIP-002 | `cargo check -p homie-orchestrator` |
| FC-DAIP-003 | `cargo clippy -p homie-orchestrator --all-targets -- -D warnings` |
| FC-DAIP-004 | scoped `git diff --check`; `make parity-lock` |

