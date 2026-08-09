# Release Readiness Report: Diri Sidebar Visible Interactions

```yaml
change_id: diri-sidebar-visible-interactions
beads: homie-5r1
status: pass
validated_at: 2026-08-07
```

## 1. Source

- PRD: `prd-spec/features/diri-sidebar-visible-interactions/2026-08-07-diri-sidebar-visible-interactions-design.md`
- OpenSpec: `openspec/changes/diri-sidebar-visible-interactions/`
- Functional cases: `docs/verification/diri-sidebar-visible-interactions/functional-cases.md`
- Beads: `homie-5r1`

## 2. Delivered

- Added app-level sidebar model state backed by `homie-ui::SidebarSessionModel`.
- Added `sync_sidebar_model` to project runtime session rows into the tested sidebar model while preserving pin/archive/multi-select state.
- Added visible `pin`, `select`, and `archive` controls to sidebar session rows.
- Added helper methods for pin/archive/multi-select updates.
- Preserved runtime session selection and avoided runtime termination on local archive.

## 3. Gate Results

| Gate | Command | Result |
|------|---------|--------|
| App regression | `cargo test -p homie-app --test app_shell_copy_regression -- --nocapture` | pass |
| Sidebar model | `cargo test -p homie-ui --test workbench_state -- --nocapture` | pass |
| Build | `cargo check -p homie-app` | pass |
| Lint | `cargo clippy -p homie-app --all-targets -- -D warnings` | pass |
| Diff hygiene | scoped `git diff --check` | pass |
| LoopX | `loopx --registry .loopx/registry.json check --scan-root /Users/bytedance/workspace/github/homie` | pass |
| Parity lock | `make parity-lock` | pass_with_remaining_gaps |

## 4. Parity Status

`UI-002` is still `partial`. This slice advances the app-visible sidebar interaction surface but does not complete:

- hover cards;
- GPUI drag reorder;
- pointer E2E;
- screenshot/manual E2E for these controls;
- runtime-backed archive lifecycle semantics.

## 5. Risk

Risk is medium-low. The change is contained to app state/rendering and test evidence. The main user-facing ambiguity is archive semantics; for this slice archive means local sidebar hiding only, not runtime session termination.
