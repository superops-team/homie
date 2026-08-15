# GPUI Large Module Test Boundaries Tasks

## T1: Spec and functional cases

- Deliverables:
  - `docs/verification/gpui-large-module-test-boundaries/functional-cases.md`
  - `openspec/changes/gpui-large-module-test-boundaries/*`
- Acceptance:
  - first slice is one Sidebar picker helper extraction only;
  - completed GPUI child slices are marked out of scope.
- Verification Cases: FC-01, FC-02

## T2: Extract picker helper module

- Deliverables:
  - `homie/crates/homie-app/src/sidebar/picker_logic.rs`
  - `homie/crates/homie-app/src/sidebar/mod.rs`
  - `homie/crates/homie-app/src/sidebar/view.rs`
- Acceptance:
  - helper module has no GPUI context/entity/window/element dependency;
  - render structure stays in `view.rs`.
- Verification Cases: FC-03

## T3: Add focused tests

- Deliverables:
  - `homie/crates/homie-app/src/sidebar/picker_logic.rs`
- Acceptance:
  - tests cover local/remote directory target normalization;
  - tests cover repo preservation decision;
  - tests cover default-agent shortcut decision.
- Verification Cases: FC-04

## T4: Final verification and review

- Deliverables:
  - verification logs and reports under `docs/verification/gpui-large-module-test-boundaries/`
- Acceptance:
  - FC-01 through FC-05 pass;
  - code review reports exist;
  - release readiness report exists.
- Verification Cases: FC-05
