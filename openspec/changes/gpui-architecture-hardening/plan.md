# GPUI Architecture Hardening Plan

## 1. Change Scope

`gpui-architecture-hardening` is a program-level PRD. The current Beads issue
`homie-4lu` delivers only Phase 0/1:

- documentation contracts;
- OpenSpec task decomposition;
- review inventory;
- child beads for code-bearing follow-up work;
- local worktree shared target rule validation.

This change must not modify GPUI application behavior. Phase 2-5 code work is
tracked by child beads and future OpenSpec changes.

## 2. Modules

### 2.1 Process And Documentation Layer

Files:

- `AGENTS.md`
- `docs/architecture/project-layout.md`
- `docs/development/standards.md`
- `docs/development/quality-gates.md`
- `docs/research/rust-package-selection.md`

Purpose:

- make the repository's required workflow executable;
- define the current project layout and build/cache rules;
- document quality gates before code work starts.

### 2.2 Durable GPUI Contract Layer

Files:

- `specs/gpui-shell.md`
- `specs/gpui-interaction-contract.md`
- `specs/ui-components.md`

Purpose:

- define long-lived GPUI shell and entity ownership boundaries;
- define interaction, focus, keyboard, stable ID, accessibility and preference
  contracts;
- define semantic UI component primitive requirements.

### 2.3 Evidence Layer

Files:

- `docs/verification/gpui-architecture-hardening/spec-review-report.md`
- `docs/verification/gpui-architecture-hardening/functional-cases.md`
- `docs/verification/gpui-architecture-hardening/review-inventory.md`
- `docs/verification/gpui-architecture-hardening/functional-verification-report.md`
- `docs/verification/gpui-architecture-hardening/code-review-round-1.md`
- `docs/verification/gpui-architecture-hardening/code-review-round-2.md`
- `docs/verification/gpui-architecture-hardening/release-readiness-report.md`

Purpose:

- preserve review findings and fixes;
- make validation cases executable and auditable;
- map every architecture risk to owner task and evidence.

### 2.4 Follow-up Work Layer

Child changes:

- `gpui-lifecycle-task-ownership`
- `gpui-utility-surfaces-first-slice`
- `gpui-ui-primitives-a11y`
- `gpui-render-path-purity`
- `gpui-visual-platform-gates`

Purpose:

- keep code-bearing architecture hardening out of the Phase 0/1 contract baseline;
- preserve reviewable vertical slices for future implementation.

## 3. Dependency Order

1. Spec review must pass before functional cases.
2. Functional cases must exist before OpenSpec task finalization.
3. Docs/specs must exist before review inventory can point to target contracts.
4. Child beads must exist before alignment can prove Phase 2-5 are not
   untracked.
5. Functional cases must pass before review and release readiness.

## 4. Non-Goals

- No GPUI runtime behavior changes.
- No RootView or UtilitySurfaces code split in this change.
- No new homie-ui primitives in this change.
- No tracked machine-local Cargo target path.
