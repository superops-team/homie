# GPUI Architecture Hardening Tasks

## T1: Finalize PRD scope after spec review

- Description: Keep the PRD as program-level scope while limiting `homie-4lu`
  closure to Phase 0/1 contract baseline.
- Deliverables:
  - `prd-spec/refactors/gpui-architecture-hardening/2026-08-14-gpui-architecture-hardening-design.md`
  - `docs/verification/gpui-architecture-hardening/spec-review-report.md`
- Acceptance:
  - PRD includes program-level positioning.
  - PRD separates Program-level completion from `homie-4lu` closure.
- Verification Cases: FC-01
- Estimate: S

## T2: Preserve worktree shared Cargo target rule

- Description: Keep the Homie project-level worktree target sharing rule in
  `AGENTS.md`, and ensure PRD treats the machine path as current-instance
  evidence, not a cross-machine product contract.
- Deliverables:
  - `AGENTS.md`
  - PRD section 3.2
- Acceptance:
  - `AGENTS.md` contains `Worktree Build Cache Rules`.
  - PRD references `AGENTS.md` as authoritative.
  - Active Homie worktrees resolve `homie/target` to one shared realpath.
- Verification Cases: FC-02, FC-08
- Estimate: S

## T3: Add required project docs

- Description: Add the four documentation files that `AGENTS.md` requires
  implementers to read before code changes.
- Deliverables:
  - `docs/architecture/project-layout.md`
  - `docs/development/standards.md`
  - `docs/development/quality-gates.md`
  - `docs/research/rust-package-selection.md`
- Acceptance:
  - All four files exist and are non-empty.
  - Each file includes GPUI architecture hardening guidance relevant to Phase
    0/1.
- Verification Cases: FC-03
- Estimate: M

## T4: Add durable GPUI component specs

- Description: Add long-lived specs for GPUI shell, interaction contracts, and
  shared UI component primitives.
- Deliverables:
  - `specs/gpui-shell.md`
  - `specs/gpui-interaction-contract.md`
  - `specs/ui-components.md`
- Acceptance:
  - Specs cover RootView, UtilitySurfaces, task/subscription ownership,
    render-path purity, stable IDs, accessibility, and semantic primitives.
- Verification Cases: FC-04
- Estimate: M

## T5: Create review inventory

- Description: Convert the P1-P10 review findings into structured owner-task
  inventory.
- Deliverables:
  - `docs/verification/gpui-architecture-hardening/review-inventory.md`
- Acceptance:
  - Inventory uses the required columns:
    `finding_id`, `source`, `current_symptom`, `target_contract`,
    `owner_task`, `verification_command`, `evidence_path`.
  - P1-P10 each have a row.
- Verification Cases: FC-05
- Estimate: S

## T6: Create child beads for code-bearing phases

- Description: Create future work items for Phase 2-5 and link them to
  `homie-4lu`, without implementing those code changes in this branch.
- Deliverables:
  - Beads issues:
    - `homie-9w2` / `gpui-lifecycle-task-ownership`
    - `homie-yon` / `gpui-utility-surfaces-first-slice`
    - `homie-0aj` / `gpui-ui-primitives-a11y`
    - `homie-4fx` / `gpui-render-path-purity`
    - `homie-mpc` / `gpui-visual-platform-gates`
- Acceptance:
  - Child beads exist.
  - `bd children homie-4lu` lists the child work, or dependency links make the
    relationship visible.
- Verification Cases: FC-07
- Estimate: S

## T7: Prove OpenSpec alignment

- Description: Map PRD requirements to OpenSpec tasks and functional cases.
- Deliverables:
  - `openspec/changes/gpui-architecture-hardening/plan.md`
  - `openspec/changes/gpui-architecture-hardening/tasks.md`
  - `openspec/changes/gpui-architecture-hardening/alignment-report.md`
- Acceptance:
  - Every Phase 0/1 PRD requirement has a task and verification case.
  - Phase 2-5 are explicitly identified as child changes.
- Verification Cases: FC-06
- Estimate: S

## T8: Guard against code scope creep

- Description: Ensure this contract-baseline branch does not modify GPUI app
  or UI runtime code.
- Deliverables:
  - `docs/verification/gpui-architecture-hardening/fc-09-no-code-scope-creep.log`
- Acceptance:
  - `git diff --name-only -- homie/crates/homie-app homie/crates/homie-ui`
    has no output.
- Verification Cases: FC-09
- Estimate: S

## T9: Run static documentation gate

- Description: Run final static diff quality check.
- Deliverables:
  - `docs/verification/gpui-architecture-hardening/fc-10-diff-check.log`
- Acceptance:
  - `git diff --check` exits 0.
- Verification Cases: FC-10
- Estimate: S
