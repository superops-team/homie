# T-007 GPUI Design System Report

```yaml
change_id: reference-parity-v1
openspec_task: T-007
beads: homie-bfj
status: pass
functional_cases:
  - FC-009
```

## 1. Summary

T-007 implemented the first reusable UI design contract slice in `homie-ui`.

Implemented:

- Radius tokens.
- Window/sidebar metrics.
- Motion timing constants.
- Agent kind enum for UI projection.
- Status state enum.
- Deterministic animation phase calculation.
- Status color name priority.

This is not the full GPUI workbench yet. It establishes tested design tokens and status-glyph state semantics for later shell implementation.

## 2. RED

Added UI token tests:

- `crates/homie-ui/tests/tokens.rs`

The tests require:

- design token values match the Reference parity contract.
- animation phase is deterministic from wall-clock seconds.
- needs-input/danger/done/working color priority is stable.

## 3. GREEN

Implemented:

- `crates/homie-ui/Cargo.toml`
- `crates/homie-ui/src/lib.rs`
- workspace registration in `Cargo.toml`

## 4. Verification

Focused command:

```bash
cargo test -p homie-ui
```

Result:

- Exit code: 0
- UI token tests: 3 passed
- Doc tests: 0 tests

Workspace regression command:

```bash
cargo test --workspace
```

Result:

- Exit code: 0
- Homie agents/app/CLI/context/LLM/memory/orchestrator/proto/runtime/storage/task/term/UI tests passed.

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

- Actual GPUI components.
- Sidebar/workbench rendering.
- Screenshot fidelity fixtures.
- Native notification/menu bar integration.

