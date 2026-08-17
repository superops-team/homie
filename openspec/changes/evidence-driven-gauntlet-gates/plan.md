# Evidence-Driven Gauntlet Gates Plan

## 1. Scope

This OpenSpec change implements the `evidence-driven-gauntlet-gates` refactor.
It turns the old-coder evidence-first review findings (G1–G5) into durable
development-process constraints in Homie's standards and quality gates.

In scope:

- Expand `docs/development/standards.md` Testing Standards (RED→GREEN→REFACTOR,
  anti-gaming rules, Tier calibration, Tier 3 failure model).
- Add failure conditions and new gates (coverage fail-under, mutation, suite
  health, checker fail-closed + negative control, evidence hard fields) to
  `docs/development/quality-gates.md`.
- Small referencing updates in `AGENTS.md`.

Out of scope:

- Any new Rust/Swift dependency or toolchain (llvm-cov, tarpaulin, cargo-mutants,
  proptest, pytest-randomly). Documents use "use the tool if available, else
  manual fallback" wording.
- Any production code change, `specs/*.md` contract change, `homie/crates`,
  `Sources`, or `Tests` edits.
- Changing existing gate commands; only adding failure conditions and evidence
  requirements on top.

## 2. Requirement Map

| PRD Requirement | OpenSpec Task | Verification Case |
|-----------------|---------------|-------------------|
| R1 TDD loop expansion | T1 | FC-01 |
| R2 anti-gaming hard rules | T1 | FC-01 |
| R3 gate failure conditions + checker fail-closed/negative control | T2 | FC-03 |
| R4 coverage fail-under + mutation gates | T2 | FC-04 |
| R5 Tier calibration + Tier 3 failure model | T1 | FC-02 |
| R6 evidence hard fields (numbers, fresh run, reproducibility) | T2 | FC-05 |
| AGENTS.md referencing updates | T3 | FC-06 |
| No specs/production edits | T4 | FC-08 |
| diff/status clean | T4 | FC-07 |

## 3. Delivery Strategy

Documentation-only change. Complete when:

1. R1–R6 are expressed in `standards.md` / `quality-gates.md`.
2. `AGENTS.md` references the expanded TDD rules.
3. FC-01 through FC-08 pass with evidence under
   `docs/verification/evidence-driven-gauntlet-gates/`.
4. No `specs/*.md` or production tree changes.

## 4. Verification

Verification uses `docs/verification/evidence-driven-gauntlet-gates/` evidence:

- FC-01 through FC-08 check the document contents and scope boundary.
- `git diff --check` and `git status --short` confirm cleanliness.
