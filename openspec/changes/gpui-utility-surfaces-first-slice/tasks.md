# GPUI UtilitySurfaces First Slice Tasks

## T1: Finalize child PRD and spec review

- Description: Create a self-contained PRD/spec and spec review report for the
  UtilitySurfaces lifecycle slice.
- Deliverables:
  - `prd-spec/refactors/gpui-utility-surfaces-first-slice/2026-08-14-gpui-utility-surfaces-first-slice-design.md`
  - `docs/verification/gpui-utility-surfaces-first-slice/spec-review-report.md`
- Acceptance:
  - PRD limits scope to History/Worktrees lifecycle.
  - No P0/P1 spec review blockers remain.
- Verification Cases: FC-01

## T2: Design functional verification cases

- Description: Define executable functional cases before implementation.
- Deliverables:
  - `docs/verification/gpui-utility-surfaces-first-slice/functional-cases.md`
- Acceptance:
  - Cases cover task fields, stale results, close guards, targeted tests and
    scope boundaries.
- Verification Cases: FC-02

## T3: Add task ownership fields and generation state

- Description: Add History and Worktrees task/generation fields to
  `UtilitySurfaces`.
- Deliverables:
  - `homie/crates/homie-app/src/surface_shell.rs`
- Acceptance:
  - `history_generation`, `history_task`, `worktrees_generation`,
    `worktrees_task` are present and initialized.
- Verification Cases: FC-03

## T3a: Track existing homie-ui icon assets

- Description: Fix the clean-worktree compile blocker caused by `.gitignore`
  ignoring the `homie-ui/assets/icons` directory through the broad `Icon?`
  pattern.
- Deliverables:
  - `.gitignore`
  - `homie/crates/homie-ui/assets/icons/*.svg`
- Acceptance:
  - existing SVG assets referenced by `include_bytes!` are tracked.
  - `.gitignore` no longer ignores nested `icons` directories through `Icon?`.
- Verification Cases: FC-04

## T4: Convert lifecycle operations away from detach

- Description: Store lifecycle tasks for History and Worktrees instead of
  detaching them.
- Deliverables:
  - `homie/crates/homie-app/src/surface_shell.rs`
- Acceptance:
  - `open_history`, `resume_history`, `refresh_worktrees`, `confirm_cleanup`
    store returned tasks in fields.
  - Closing History/Worktrees clears the corresponding task field.
- Verification Cases: FC-05, FC-08

## T5: Add stale result guards and tests

- Description: Ensure stale History and Worktrees async results are ignored.
- Deliverables:
  - `homie/crates/homie-app/src/surface_shell.rs`
- Acceptance:
  - stale History results cannot overwrite newer History state.
  - stale Worktrees results cannot overwrite newer Worktrees state.
  - closed surfaces ignore late results.
- Verification Cases: FC-06, FC-07, FC-08

## T6: Run final targeted gates

- Description: Execute formatting, targeted tests, scope and diff checks.
- Deliverables:
  - `docs/verification/gpui-utility-surfaces-first-slice/functional-verification-report.md`
  - command logs under `docs/verification/gpui-utility-surfaces-first-slice/`
- Acceptance:
  - all functional cases pass.
  - no out-of-scope crates are modified.
- Verification Cases: FC-09
