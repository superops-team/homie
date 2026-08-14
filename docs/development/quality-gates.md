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
