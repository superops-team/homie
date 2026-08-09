# T-015 Context Memory Task Intent Report

```yaml
change_id: reference-parity-v1
openspec_task: T-015
beads: homie-ele
status: pass
functional_cases:
  - FC-016
  - FC-018
```

## 1. Summary

T-015 implemented minimal controller contracts for Homie-owned context, task, memory and intent orchestration.

Implemented:

- `homie-context`: safe session summary with sensitive-word redaction.
- `homie-task`: task open/claim/block/return state model.
- `homie-memory`: memory candidate with required source event and unsafe-content rejection.
- `homie-orchestrator`: deterministic route decisions for New Agent, command palette and MCP prompt routing.

These are pure domain contracts. They do not yet persist to SQLite or drive runtime execution.

## 2. RED

Added focused tests:

- `crates/homie-context/tests/summary.rs`
- `crates/homie-task/tests/task_state.rs`
- `crates/homie-memory/tests/candidate.rs`
- `crates/homie-orchestrator/tests/routing.rs`

## 3. GREEN

Implemented:

- `crates/homie-context`
- `crates/homie-task`
- `crates/homie-memory`
- `crates/homie-orchestrator`
- workspace registration in `Cargo.toml`

## 4. Verification

Focused command:

```bash
cargo test -p homie-context -p homie-task -p homie-memory -p homie-orchestrator
```

Result:

- Exit code: 0
- Context tests: 1 passed
- Task tests: 2 passed
- Memory tests: 3 passed
- Orchestrator tests: 3 passed

Workspace regression command:

```bash
cargo test --workspace
```

Result:

- Exit code: 0
- Homie agents/app/CLI/context/LLM/memory/orchestrator/proto/runtime/storage/task tests passed.

Safety checks:

```bash
rg -n -i "<old-reference-name-pattern>" .
git diff --check
```

Result:

- Old reference name scan: no matches.
- Markdown/patch whitespace check: pass.

## 5. Remaining Scope

Still deferred:

- Persistence integration.
- Runtime-driven task updates.
- Memory approval UI/workflow.
- Permission-aware orchestration.

