# Reference Parity V1 Dev Loop Alignment Report

```yaml
change_id: reference-parity-v1
report_type: dev-loop-alignment
status: pass_for_component_spec_work
beads: homie-h7n
source_prd: prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md
functional_cases: docs/verification/reference-parity-v1/functional-cases.md
openspec_tasks: openspec/changes/reference-parity-v1/tasks.md
dev_loop_step: 4
```

## 1. Gate Decision

Decision: pass for component spec work; implementation remains blocked until child Beads and task-specific SDD/TDD are started.

Reason:

- PRD FR-1 through FR-20 are mapped to OpenSpec tasks.
- Functional cases FC-001 through FC-018 cover all P0/P1 requirements and the P2 remote/node path.
- OpenSpec tasks now include Task -> Functional Case mapping.
- Implementation remains blocked until required long-lived component specs exist and child Beads are created for executable slices.

## 2. FR To Task To Case Matrix

| PRD FR | OpenSpec tasks | Functional cases | Evidence path | Status |
|--------|----------------|------------------|---------------|--------|
| FR-1 | T-001 | FC-001, FC-002, FC-003, FC-018 | `functional-cases.md`, future case execution report | covered |
| FR-2 | T-001, T-002, T-004, T-005 | FC-004, FC-018 | component spec impact and future specs | covered |
| FR-3 | T-003 | FC-005 | future manifest tests | covered |
| FR-4 | T-002, T-004, T-005 | FC-006, FC-007 | future runtime E2E | covered |
| FR-5 | T-002 | FC-006 | future protocol tests | covered |
| FR-6 | T-006 | FC-008 | future terminal tests and real PTY smoke | covered |
| FR-7 | T-007, T-008, T-009 | FC-009 | future UI fidelity report | covered |
| FR-8 | T-010 | FC-010 | future worktree safety tests | covered |
| FR-9 | T-009 | FC-011 | future history resume report | covered |
| FR-10 | T-011, T-008 | FC-012 | future artifact/browser report | covered |
| FR-11 | T-012 | FC-013 | future usage/proxy report | covered |
| FR-12 | T-012, T-017 | FC-013, FC-015 | future security report | covered |
| FR-13 | T-014 | FC-015 | future remote-node report | covered |
| FR-14 | T-013 | FC-014 | future MCP/CLI report | covered |
| FR-15 | T-004, T-016 | FC-007, FC-017 | future runtime/perf reports | covered |
| FR-16 | T-016 | FC-017 | future release readiness report | covered |
| FR-17 | T-016 | FC-017 | future packaged perf report | covered |
| FR-18 | T-005 | FC-004, FC-007, FC-018 | future storage and full gate report | covered |
| FR-19 | T-012, T-014, T-016, T-017 | FC-001, FC-013, FC-015, FC-017, FC-018 | future security gauntlet | covered |
| FR-20 | T-015 | FC-016 | future context/memory/task report | covered |

## 3. Required Fixes From Step 1

| Step 1 issue | Required fix | Evidence | Result |
|--------------|--------------|----------|--------|
| P0 Task lacks functional Case IDs | Add functional Case list and Task -> Case mapping | `functional-cases.md`, `tasks.md` mapping table | fixed |
| P0 umbrella scope too large | Keep umbrella, require child Beads and component specs before implementation | `dev-loop-spec-review-report.md`, this report | fixed as gate |
| P0 component specs absent | Mark implementation blocked until specs exist | `component-spec-impact-report.md`, FC-004 | fixed as gate |
| P1 verification missing executable cases | Add FC-001 through FC-018 | `functional-cases.md` | fixed |

## 4. Remaining Blockers Before SDD/TDD Implementation

| Blocker | Why it blocks implementation | Required action |
|---------|------------------------------|-----------------|
| Child implementation Beads missing | Long-lived component specs now exist; implementation still needs child Beads and focused task specs | Create child Beads from T-002 through T-017 and keep component specs synchronized |
| Child Beads missing | Umbrella scope is not implementable in one safe branch | Split T-001..T-017 or priority slices into Beads with dependencies |
| Case execution not possible yet | Most cases reference future code paths | During implementation, each task must move Case status from designed to pass/fail/blocked with evidence |
| Real release environment unknown | Signing/notarization and packaged updater need external environment | Keep FC-017 blocked until real artifacts are available |

## 5. Next Dev Loop Step

Allowed next step:

- Continue within dev-loop by executing T-001: create/update component specs and child Beads.

Not allowed yet:

- Starting Rust/GPUI/runtime implementation for T-002 through T-017.
- Marking any functional Case as pass without executing it on real code.
- Closing `homie-h7n`.

