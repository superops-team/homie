# GPUI Architecture Hardening Alignment Report

## 1. Scope

This alignment report covers `homie-4lu` Phase 0/1 only. Phase 2-5 code
hardening is deliberately deferred to child beads.

## 2. PRD To Task Mapping

| PRD Requirement | OpenSpec Task | Functional Case | Status |
|-----------------|---------------|-----------------|--------|
| Program-level positioning and Phase 0/1 closure boundary | T1 | FC-01 | Covered |
| Worktree shared Cargo target rule remains in `AGENTS.md` | T2 | FC-02, FC-08 | Covered |
| Required project docs exist | T3 | FC-03 | Covered |
| Durable GPUI shell/interaction/component specs exist | T4 | FC-04 | Covered |
| Review inventory schema maps P1-P10 to owner tasks and evidence | T5 | FC-05 | Covered |
| Child beads exist for code-bearing phases | T6 | FC-07 | Covered |
| OpenSpec plan/tasks/alignment exist and reference verification cases | T7 | FC-06 | Covered |
| Current change does not implement Phase 2-5 code work | T8 | FC-09 | Covered |
| Static diff quality passes | T9 | FC-10 | Covered |

## 3. Functional Case To Task Mapping

| Functional Case | Tasks | Purpose |
|-----------------|-------|---------|
| FC-01 | T1 | PRD scope and closure boundary |
| FC-02 | T2 | `AGENTS.md` worktree target rule |
| FC-03 | T3 | required docs existence |
| FC-04 | T4 | GPUI specs existence and coverage |
| FC-05 | T5 | review inventory schema and P1-P10 coverage |
| FC-06 | T7 | OpenSpec plan/tasks/alignment coverage |
| FC-07 | T6 | child beads for Phase 2-5 |
| FC-08 | T2 | active worktrees share Cargo target |
| FC-09 | T8 | no GPUI code scope creep |
| FC-10 | T9 | static diff quality |

## 4. Deferred Child Changes

The following work is required for the program-level target state but is not
part of `homie-4lu` closure:

| Child Change | Program Area | Reason Deferred |
|--------------|--------------|-----------------|
| `homie-9w2` / `gpui-lifecycle-task-ownership` | task/subscription lifecycle | requires code changes and focused tests |
| `homie-yon` / `gpui-utility-surfaces-first-slice` | UtilitySurfaces first vertical slice | requires code changes and GPUI behavior validation |
| `homie-0aj` / `gpui-ui-primitives-a11y` | semantic components and accessibility | requires component API and GPUI tests |
| `homie-4fx` / `gpui-render-path-purity` | render-path locks and derived state | requires production code changes |
| `homie-mpc` / `gpui-visual-platform-gates` | runtime visual/platform matrix | requires launch/screenshot evidence |

## 5. Alignment Verdict

No Phase 0/1 PRD requirement is unmapped. Code-bearing requirements are
explicitly deferred to child changes and must not be treated as closure criteria
for `homie-4lu`.
