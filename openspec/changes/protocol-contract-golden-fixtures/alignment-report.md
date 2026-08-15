# Protocol Contract Golden Fixtures Alignment Report

## 1. Alignment Summary

- PRD: `prd-spec/refactors/protocol-contract-golden-fixtures/2026-08-13-protocol-contract-golden-fixtures-design.md`
- Spec review: `docs/verification/protocol-contract-golden-fixtures/spec-review-report.md`
- Functional cases: `docs/verification/protocol-contract-golden-fixtures/functional-cases.md`
- Plan: `openspec/changes/protocol-contract-golden-fixtures/plan.md`
- Tasks: `openspec/changes/protocol-contract-golden-fixtures/tasks.md`

Status: aligned for first-slice implementation.

## 2. PRD Requirement To Task Mapping

| PRD requirement | Task | Case | Status |
|---|---|---|---|
| Shared Swift/Rust golden fixtures | T2, T3, T4 | FC-03, FC-04, FC-05 | Covered |
| Dual-end tests read the same fixture directory | T3, T4 | FC-04, FC-05 | Covered |
| Cover request/response/event/null/error/invalid cases | T2, T3, T4 | FC-03, FC-04, FC-05 | Covered |
| Sensitive payloads excluded | T2 | FC-03 | Covered |
| Local drift gate | T5 | FC-06 | Covered |
| OpenSpec/evidence traceability | T1, T6 | FC-01, FC-02, FC-07 | Covered |

## 3. Case To Task Mapping

| Case | Tasks | Notes |
|---|---|---|
| FC-01 | T1 | Spec/review gate |
| FC-02 | T1 | OpenSpec coverage gate |
| FC-03 | T2 | Fixture contract and safety |
| FC-04 | T3 | Rust focused test |
| FC-05 | T4 | Swift focused test |
| FC-06 | T5 | Local check gate |
| FC-07 | T6 | Static final gate |

## 4. Out-Of-Scope Guard

| Out of scope | Guard |
|---|---|
| New wire methods | No fixture case introduces a new method contract beyond envelope shape |
| Schema/codegen | No tasks mention schema generation |
| Runtime package fixture data | No package/dev script task copies `protocol-fixtures` |
| Protocol format migration | Plan states current NDJSON envelope remains unchanged |

## 5. Verdict

No unmapped P0/P1 requirement remains for the first slice. Implementation can proceed.
