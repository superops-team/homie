# Release Readiness Report — evidence-driven-gauntlet-gates

## 1. Conclusion

`evidence-driven-gauntlet-gates` is complete. The old-coder evidence-first
review findings (G1–G5) are now expressed as durable development-process
constraints in Homie's standards and quality gates.

## 2. What Changed

- `docs/development/standards.md` §6: expanded Testing Standards with the
  RED→GREEN→REFACTOR loop (R1), six anti-gaming hard rules (R2), and Tier 1/2/3
  calibration with a Tier 3 failure model (R5).
- `docs/development/quality-gates.md`: added gate failure conditions and
  checker fail-closed + negative-control rules (R3), changed-line coverage
  fail-under and mutation gates (R4), suite health gate, and evidence hard
  fields (R6).
- `AGENTS.md`: Required Workflow step 8 and Implementation Guidance now
  reference the expanded TDD rules and Tier 3 default for security-sensitive
  domains.

## 3. Verification Evidence

- Functional cases: `functional-cases.md` (FC-01 through FC-08).
- Verification report: `functional-verification-report.md` — 8/8 pass.
- Spec review: `spec-review-report.md` — no `specs/*.md` update required.
- `git diff --check` clean; `git status --short` shows only intended files.

## 4. Known Limitations / Deferred Follow-up

- Tool-based coverage and mutation tooling (llvm-cov / cargo-mutants / proptest)
  are specified as "use if available, else manual fallback"; actual tool
  adoption is deferred to a future PRD.
- This change specifies the process; runtime enforcement happens per-change,
  not globally.

## 5. Beads

- `homie-awb` — 引入证据驱动开发链路约束到 quality-gates 与 standards (chore, P0)
- change_id: `evidence-driven-gauntlet-gates`
