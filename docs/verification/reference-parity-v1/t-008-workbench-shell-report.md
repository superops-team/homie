# T-008 Workbench Shell Surfaces Report

```yaml
change_id: reference-parity-v1
openspec_task: T-008
beads: homie-cgo
status: pass
functional_cases:
  - FC-009
  - FC-012
```

## 1. Summary

T-008 implemented reusable workbench/sidebar state contracts in `homie-ui`.

Implemented:

- `SidebarState` with visibility toggle, width clamp and reset.
- `move_before` and `move_to_end` reorder helpers.
- `WorkbenchLayout` and `PaneHeights`.

This is not the rendered GPUI workbench yet. It provides tested state primitives for sidebar and workbench surfaces.

## 2. RED

Added workbench state tests:

- `crates/homie-ui/tests/workbench_state.rs`

The tests require:

- sidebar width clamps to min/max and resets to default.
- reorder helpers move items without duplication.
- workbench layout splits primary and auxiliary panes.

## 3. GREEN

Implemented:

- workbench/sidebar state primitives in `crates/homie-ui/src/lib.rs`.

## 4. Verification

Focused command:

```bash
cargo test -p homie-ui
```

Result:

- Exit code: 0
- UI token tests: 3 passed
- Workbench state tests: 3 passed

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

- Actual GPUI sidebar rendering.
- Terminal pane composition.
- Inspector tabs.
- Artifact chips.

