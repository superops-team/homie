# GPUI Large Module Test Boundaries Alignment Report

## 1. Alignment Summary

- PRD: `prd-spec/refactors/gpui-large-module-test-boundaries/2026-08-13-gpui-large-module-test-boundaries-design.md`
- Spec review: `docs/verification/gpui-large-module-test-boundaries/spec-review-report.md`
- Functional cases: `docs/verification/gpui-large-module-test-boundaries/functional-cases.md`
- Plan: `openspec/changes/gpui-large-module-test-boundaries/plan.md`
- Tasks: `openspec/changes/gpui-large-module-test-boundaries/tasks.md`

Status: aligned for first-slice implementation.

## 2. PRD Requirement To Task Mapping

| PRD requirement | Task | Case | Status |
|---|---|---|---|
| Extract one high-change UI behavior | T2 | FC-03 | Covered |
| Add pure Rust tests | T3 | FC-04 | Covered |
| Keep UI behavior/visual structure unchanged | T2 | FC-03, FC-05 | Covered |
| Avoid overlap with completed GPUI slices | T1 | FC-01, FC-02 | Covered |
| Prove no GPUI dependency in logic module | T2 | FC-03 | Covered |

## 3. Case To Task Mapping

| Case | Tasks | Notes |
|---|---|---|
| FC-01 | T1 | Spec/review gate |
| FC-02 | T1 | OpenSpec coverage gate |
| FC-03 | T2 | Pure module static gate |
| FC-04 | T3 | Focused picker tests |
| FC-05 | T4 | Static final gate |

## 4. Out-Of-Scope Guard

| Out of scope | Guard |
|---|---|
| Sidebar rewrite | Only helper functions move |
| Shortcut rank | Already completed; no task mentions it |
| UtilitySurfaces lifecycle | Already completed; no task touches it |
| UI primitives | No `homie-ui` task |
| RootView lifecycle | No root task |

## 5. Verdict

No unmapped P0/P1 requirement remains for the first slice. Implementation can proceed.
