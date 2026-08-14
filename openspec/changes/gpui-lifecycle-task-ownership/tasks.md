# GPUI Lifecycle Task Ownership Tasks

## T1: Finalize PRD and functional cases

- Deliverables:
  - `prd-spec/refactors/gpui-lifecycle-task-ownership/2026-08-14-gpui-lifecycle-task-ownership-design.md`
  - `docs/verification/gpui-lifecycle-task-ownership/functional-cases.md`
- Verification Cases: FC-01

## T2: Hold RootView child subscriptions

- Deliverables:
  - `homie/crates/homie-app/src/root.rs`
- Acceptance:
  - terminal/sidebar/launcher/navigation/inspector subscriptions are assigned
    to local `Subscription` values.
  - `_subscriptions` stores those values with activation and bounds observer.
- Verification Cases: FC-02

## T3: Run targeted tests

- Deliverables:
  - `docs/verification/gpui-lifecycle-task-ownership/fc-03-targeted-tests.log`
- Verification Cases: FC-03

## T4: Run static gates

- Deliverables:
  - `docs/verification/gpui-lifecycle-task-ownership/fc-04-static-gates.log`
- Verification Cases: FC-04
