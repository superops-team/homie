# Evidence-Driven Gauntlet Gates Tasks

## T1: Expand standards.md Testing Standards

- Deliverables:
  - `docs/development/standards.md`
- Acceptance:
  - Section 6 contains RED→GREEN→REFACTOR loop (R1);
  - Section 6 contains the six anti-gaming hard rules (R2);
  - Section 6 contains Tier 1/2/3 calibration and Tier 3 failure model (R5).
- Verification Cases: FC-01, FC-02

## T2: Add failure conditions and new gates to quality-gates.md

- Deliverables:
  - `docs/development/quality-gates.md`
- Acceptance:
  - each existing gate states its failure condition;
  - checker fail-closed + negative-control rules present (R3);
  - coverage fail-under + mutation gates present (R4);
  - evidence hard fields (numbers / fresh run / reproducibility / skip reason) present (R6).
- Verification Cases: FC-03, FC-04, FC-05

## T3: Reference TDD rules from AGENTS.md

- Deliverables:
  - `AGENTS.md`
- Acceptance:
  - Required Workflow step 8 references the expanded TDD rules in standards;
  - Implementation Guidance references anti-gaming / Tier rules where relevant.
- Verification Cases: FC-06

## T4: Record evidence and verify scope boundary

- Deliverables:
  - `docs/verification/evidence-driven-gauntlet-gates/spec-review-report.md`
  - `docs/verification/evidence-driven-gauntlet-gates/functional-cases.md`
  - `docs/verification/evidence-driven-gauntlet-gates/functional-verification-report.md`
  - `docs/verification/evidence-driven-gauntlet-gates/release-readiness-report.md`
- Acceptance:
  - FC-01 through FC-08 pass;
  - `git diff --check` and `git status --short` clean;
  - no edits to `specs/*.md`, `homie/crates`, `Sources`, `Tests`.
- Verification Cases: FC-07, FC-08
