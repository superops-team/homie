# GPUI UtilitySurfaces First Slice Alignment Report

## 1. Scope

This report aligns the child PRD, functional cases and OpenSpec tasks for
`gpui-utility-surfaces-first-slice`.

## 2. PRD To Task Mapping

| PRD Requirement | Task | Functional Case | Status |
|-----------------|------|-----------------|--------|
| Scope limited to History/Worktrees lifecycle | T1 | FC-01 | Covered |
| Functional cases designed before implementation | T2 | FC-02 | Covered |
| History/Worktrees task fields and generation state | T3 | FC-03 | Covered |
| Clean worktree can compile embedded homie-ui icon assets | T3a | FC-04 | Covered |
| Lifecycle tasks held instead of detached | T4 | FC-05 | Covered |
| Closing surfaces clears lifecycle tasks | T4 | FC-08 | Covered |
| Stale History result ignored | T5 | FC-06 | Covered |
| Stale Worktrees result ignored | T5 | FC-07 | Covered |
| Closed surface ignores late result | T5 | FC-08 | Covered |
| Targeted tests, fmt, diff and scope gates | T6 | FC-09 | Covered |

## 3. Out-Of-Scope Guard

The following parent-program work is explicitly excluded from this child change:

- RootView split;
- UtilitySurfaces file split;
- homie-ui primitive implementation;
- homie-ui source changes beyond tracking existing icon assets required by current code;
- render-path projection migration;
- visual platform gate implementation.

## 4. Verdict

Every in-scope PRD requirement maps to an OpenSpec task and at least one
functional verification case. No Phase 2 sibling work is included in this
slice.
