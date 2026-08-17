# Homie Quality Gates

## 1. Purpose

Quality gates define what evidence is required before a Homie change can be
considered ready. The required gate depends on the change type and risk.

## 2. Universal Gates

Every change should run:

```bash
git diff --check
git status --short
```

Meaningful code changes also need repository-native checks and evidence under
`docs/verification/<change-id>/`.

### 2.1 Gate Failure Conditions

Every gate below states its failure condition. A gate is not "run" unless its
failure condition is evaluated. A layer that prints a number and exits 0 is a
report, not a gate.

Home-grown checkers (grep gates, custom scripts, manual mutation runners) must:

- **Fail closed** — a crash, unreadable input, unexpected exit code, or a
  silently skipped item inside gate code is a hard failure of the layer, never
  a pass. No `|| true`, no `2>/dev/null`, no bare fallthrough.
- **Prove it can fail before trusting its pass** — run it once against a
  known-bad input (a negative control) and watch it fail, then record the
  control in evidence. A negative control proves one known-bad case reaches the
  failure path, not that the checker catches every violation. When a gate's
  coverage is narrower than the rule it serves, say so where the rule is
  written.

## 3. Documentation / Spec-Only Gates

For changes that only update PRD/spec/docs/OpenSpec:

```bash
git diff --check
```

Required evidence:

- spec review report when the doc changes process or architecture;
- functional case document when the doc defines future validation behavior;
- OpenSpec alignment report when the doc is tied to an implementation change;
- Beads issue status and metadata.

## 4. Rust Workspace Gates

Baseline commands:

```bash
cargo fmt --check --manifest-path homie/Cargo.toml
cargo check --manifest-path homie/Cargo.toml --workspace
cargo test --manifest-path homie/Cargo.toml --workspace
```

Targeted commands may be used during development, but release readiness must
record why any full workspace gate was skipped.

### 4.1 Changed-Line Coverage Gate (fail-under)

Coverage constrains changed/added lines, not a global percentage. Global % is
vanity — changed-line coverage is the constraint.

- If a coverage tool is available: `cargo llvm-cov --fail-under-lines <n>`,
  `cargo tarpaulin --fail-under <n>`, or equivalent, and it MUST exit nonzero
  when the threshold is missed.
- If no coverage tool is available, do not report a fake percentage; record a
  manual line-by-line check that every changed/added line is executed by a test,
  and state that tool-based coverage was skipped with reason.

### 4.2 Mutation Gate

- If `cargo-mutants` is available, run it on the changed crate and classify
  survivors as "equivalent, because <reason>" rather than adding meaningless
  tests to kill them.
- If no mutation tool is available, do manual mutation: introduce 3–5 plausible
  bugs into the new code one at a time (flip a comparison, off-by-one a bound,
  drop a condition, return early); the suite must kill every one, then restore.
  Hand-written mutants get no "equivalent" excuse — choose real bugs.

Coverage and mutation are **mandatory** for security-sensitive domains
(credential custody, virtual key issuance, LLM proxying, orchestration,
concurrency, data loss); otherwise they follow the Tier calibration in
`docs/development/standards.md` (§6.3).

## 5. GPUI App Gates

For GPUI behavior changes:

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-app
cargo test --manifest-path homie/Cargo.toml -p homie-ui
```

Additional evidence depends on risk:

| Change | Minimum Evidence |
|--------|------------------|
| Pure layout/state logic | Unit tests |
| Entity action/focus behavior | `#[gpui::test]` |
| Task/subscription lifecycle | deterministic async tests and stale-result coverage |
| Render-path performance | targeted tests plus code review evidence that render does not start tasks/I/O |
| UI primitive or control | pointer, keyboard, focus, disabled and accessibility-state tests |
| Window/material/visual behavior | real app launch, screenshot or recording, light/dark and preference notes |

`cargo check` alone is never enough for GPUI interaction, focus, window, or
visual claims.

## 6. Swift Gates

For Swift CLI/protocol/core changes:

```bash
swift test --package-path .
```

If Swift/Rust protocol boundaries change, include parity fixtures or a documented
roundtrip check.

## 7. Packaging Gates

For packaging or release scripts:

```bash
homie/scripts/package.sh
codesign --verify --deep --strict <path-to-app>
```

Record package path, target triple, helper binaries, signing/notary assumptions,
and any skipped cross-platform target.

## 8. Worktree Build Cache Gate

For any worktree setup or build-cache rule change:

```bash
git worktree list --porcelain
for wt in <active-homie-worktree-list>; do realpath "$wt/homie/target"; done | sort -u
git status --short --ignored=matching homie/target
```

Passing result:

- exactly one shared target realpath for active Homie worktrees on the current machine;
- `homie/target` ignored, not tracked.

## 9. Evidence Naming

Use stable evidence names:

```text
docs/verification/<change-id>/functional-cases.md
docs/verification/<change-id>/functional-verification-report.md
docs/verification/<change-id>/code-review-round-1.md
docs/verification/<change-id>/code-review-round-2.md
docs/verification/<change-id>/release-readiness-report.md
```

Command logs should use `fc-<number>-<short-name>.log`.

## 10. Suite Health Gate

Run the test suite in randomized order where the tool supports it (e.g.
`cargo test -- --shuffle` style if available; otherwise note the lack). Repeat
any suspected flaky test. Every evidence number rests on the suite being
deterministic — a flaky suite quietly invalidates the report. On a repo with
pre-existing failures, record the baseline (which tests already fail, verbatim)
and hold the line at zero NEW failures; surface pre-existing failures, do not
silently "improve" them.

## 11. Evidence Requirements

A release-readiness report must:

- Map each behavior to the test / gate layer that verifies it, or an explicit
  "skipped + reason" line — never silently absent.
- Report the actual command run and its actual result (pasted numbers, not
  adjectives) for every layer — e.g. "47/47 tests pass, changed-line coverage
  100% (31/31 lines), 5/5 manual mutants killed".
- Source all numbers from one final fresh run executed after the last code edit;
  mid-task results are stale and must not be reported.
- Be reproducible from the repo alone: every cited command (including any
  mutation script) exists as a persisted file in the repo; tool versions are
  pinned or recorded; one entry-point command reruns every layer; the source
  state is identified (commit SHA, or tree hash when git is absent).
- List skipped layers and why, and honestly record anything that failed and how
  it was resolved.
