# Code Review Report: Diri Sidebar Visible Interactions

```yaml
change_id: diri-sidebar-visible-interactions
beads: homie-5r1
status: pass
reviewed_at: 2026-08-07
```

## 1. Scope

- `crates/homie-app/src/main.rs`
- `crates/homie-app/tests/app_shell_copy_regression.rs`
- PRD/OpenSpec/evidence for `diri-sidebar-visible-interactions`

## 2. Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| medium | Correctness | `crates/homie-app/src/main.rs` sidebar render path | Existing app rendered raw runtime rows and did not expose the tested sidebar model capabilities. Users could not pin/archive/multi-select from the visible sidebar. | fixed: app now syncs `SidebarSessionModel` from runtime sessions and renders visible controls. |
| medium | Scope | `archive_sidebar_session` | A visible archive button could be mistaken for runtime termination if wired directly to runtime kill. | fixed: archive is local sidebar state only in this slice; release evidence records runtime archive semantics as a later decision. |
| low | Test Quality | app regression test | Source regression cannot replace pointer E2E. | accepted: kept `UI-002` partial and documented GPUI click/drag E2E as remaining. |

## 3. Brooks-Lint Review

Mode: PR Review  
Scope: focused app sidebar interaction slice  
Health Score: 92/100

No critical design issue remains. The app now consumes the durable `homie-ui` sidebar model instead of duplicating interaction rules in the render loop. The main residual risk is verification depth: source regression plus compile/lint protects wiring, but real pointer interaction still needs a future E2E gate.

## 4. Test Quality Review

Mode: Test Quality Review  
Scope: sidebar model/app regression tests

Test Suite Map:

```text
Unit/model tests: homie-ui workbench_state
Integration/source regression: homie-app app_shell_copy_regression
E2E tests: pending
```

No mock abuse was found. The remaining test gap is intentional: app pointer E2E is not available in this slice, so source regression is used only as a guard and not as completion evidence.

## 5. Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-app --test app_shell_copy_regression -- --nocapture` | pass |
| `cargo test -p homie-ui --test workbench_state -- --nocapture` | pass |
| `cargo check -p homie-app` | pass |
| `cargo clippy -p homie-app --all-targets -- -D warnings` | pass |
| scoped `git diff --check` | pass |
| `make parity-lock` | pass_with_remaining_gaps |
