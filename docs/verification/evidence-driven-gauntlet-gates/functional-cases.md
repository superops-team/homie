# Functional Verification Cases — evidence-driven-gauntlet-gates

Each case records the command, expected result, and failure handling.

## FC-01 — standards.md contains TDD loop (R1) and anti-gaming rules (R2)

- Command:
  ```bash
  grep -q "RED → GREEN → REFACTOR" docs/development/standards.md && \
  grep -q "Anti-Gaming Rules" docs/development/standards.md
  ```
- Expected: both grep exit 0.
- Failure handling: missing either phrase → hard fail, edit standards.md §6.1/§6.2.

## FC-02 — standards.md contains Tier calibration and failure model (R5)

- Command:
  ```bash
  grep -q "Calibration (Tier 1 / 2 / 3)" docs/development/standards.md && \
  grep -q "failure model" docs/development/standards.md
  ```
- Expected: both grep exit 0.

## FC-03 — quality-gates.md has fail-closed + negative control (R3)

- Command:
  ```bash
  grep -q "Fail closed" docs/development/quality-gates.md && \
  grep -q "negative control" docs/development/quality-gates.md
  ```
- Expected: both grep exit 0.

## FC-04 — quality-gates.md has coverage fail-under + mutation (R4)

- Command:
  ```bash
  grep -q "fail-under" docs/development/quality-gates.md && \
  grep -q "Mutation Gate" docs/development/quality-gates.md
  ```
- Expected: both grep exit 0.

## FC-05 — quality-gates.md has evidence hard fields (R6)

- Command:
  ```bash
  grep -q "Evidence Requirements" docs/development/quality-gates.md && \
  grep -q "final fresh run" docs/development/quality-gates.md
  ```
- Expected: both grep exit 0.

## FC-06 — AGENTS.md references the expanded TDD rules

- Command:
  ```bash
  grep -q "RED→GREEN→REFACTOR" AGENTS.md && \
  grep -q "Tier 3" AGENTS.md
  ```
- Expected: both grep exit 0.

## FC-07 — diff and worktree clean

- Command:
  ```bash
  git diff --check
  ```
- Expected: exit 0, no whitespace errors.

## FC-08 — scope boundary: no specs/production edits

- Command:
  ```bash
  git status --short | grep -E "specs/|homie/crates|Sources/|Tests/"
  ```
- Expected: no matching lines (grep exits 1). Modified files limited to
  `AGENTS.md`, `docs/development/standards.md`, `docs/development/quality-gates.md`,
  plus new PRD/OpenSpec/evidence files.
