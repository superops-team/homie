# Architecture Audit Hardening Tasks

## T1: Repair parent PRD after spec review

- Deliverables:
  - `prd-spec/refactors/architecture-audit-hardening/2026-08-14-architecture-audit-hardening-design.md`
- Acceptance:
  - parent PRD close scope is explicit;
  - existing PRD/spec relationships are mapped;
  - Phase 1-4 child Bead strategy is explicit;
  - protocol and UI behavior compatibility constraints are listed.
- Verification Cases: FC-01, FC-02, FC-03, FC-04

## T2: Record spec review evidence

- Deliverables:
  - `docs/verification/architecture-audit-hardening/spec-review-report.md`
- Acceptance:
  - all P1 review findings have a fixed status;
  - remaining P2 risks have a documented treatment.
- Verification Cases: FC-01, FC-07

## T3: Design functional verification cases

- Deliverables:
  - `docs/verification/architecture-audit-hardening/functional-cases.md`
- Acceptance:
  - FC-01 through FC-08 are present;
  - each case has command, expected result, and failure handling;
  - coverage matrix maps PRD requirements to cases.
- Verification Cases: FC-04, FC-07

## T4: Create OpenSpec plan and alignment

- Deliverables:
  - `openspec/changes/architecture-audit-hardening/plan.md`
  - `openspec/changes/architecture-audit-hardening/tasks.md`
  - `openspec/changes/architecture-audit-hardening/alignment-report.md`
- Acceptance:
  - OpenSpec files are non-empty;
  - Brooks findings are mapped to phases;
  - every task references verification cases.
- Verification Cases: FC-05, FC-06, FC-07

## T5: Execute Phase 0 verification

- Deliverables:
  - `docs/verification/architecture-audit-hardening/functional-verification-report.md`
- Acceptance:
  - FC-01 through FC-08 pass;
  - no production code changes are included in this Phase 0 loop.
- Verification Cases: FC-01, FC-02, FC-03, FC-04, FC-05, FC-06, FC-07, FC-08

## T6: Produce release readiness evidence and close Beads

- Deliverables:
  - `docs/verification/architecture-audit-hardening/release-readiness-report.md`
  - Beads `homie-om7` close reason
- Acceptance:
  - release readiness states this is documentation/planning only;
  - Beads close reason references readiness evidence.
- Verification Cases: FC-07, FC-08
