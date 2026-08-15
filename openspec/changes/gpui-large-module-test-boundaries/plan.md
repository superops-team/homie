# GPUI Large Module Test Boundaries Plan

## 1. Scope

First slice for `gpui-large-module-test-boundaries`.

## 2. In Scope

- Extract Sidebar new-agent picker helper logic into `sidebar/picker_logic.rs`.
- Keep render structure and event handlers in `sidebar/view.rs`.
- Add ordinary Rust unit tests for picker helper behavior.
- Prove helper module has no GPUI context/entity/window/element dependency.

## 3. Out Of Scope

- Redesign system or UI primitives.
- Shortcut rank logic already covered by `gpui-render-path-purity`.
- UtilitySurfaces lifecycle already covered by `gpui-utility-surfaces-first-slice`.
- RootView task/subscription lifecycle already covered by `gpui-lifecycle-task-ownership`.
- Any terminal/inspector/root extraction.

## 4. Design

Move pure helper functions from `sidebar/view.rs` to `sidebar/picker_logic.rs`:

- `remote_picker_target`
- `normalize_remote_picker_path`
- `should_resolve_active_repo`
- `agent_picker_shortcut`

These helpers operate on strings and agent ids only. `view.rs` continues to own GPUI rendering and click handlers.

## 5. Evidence

- Spec review: `docs/verification/gpui-large-module-test-boundaries/spec-review-report.md`
- Functional cases: `docs/verification/gpui-large-module-test-boundaries/functional-cases.md`
- Functional verification: `docs/verification/gpui-large-module-test-boundaries/functional-verification-report.md`
- Code review: `docs/verification/gpui-large-module-test-boundaries/code-review-round-1.md`, `code-review-round-2.md`
- Release readiness: `docs/verification/gpui-large-module-test-boundaries/release-readiness-report.md`

## 6. Risks

| Risk | Control |
|---|---|
| Scope expands into sidebar rewrite | Only helper functions move |
| Hidden GPUI dependency remains | Static `rg` gate checks helper module |
| Behavior changes | Focused tests cover existing helper behavior |
