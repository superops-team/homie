# Architecture Audit Hardening Alignment Report

## 1. Alignment Summary

This change implements Phase 0 only. It aligns the Brooks architecture audit
parent PRD with executable documentation tasks and verification cases.

No production code changes are part of this OpenSpec change.

## 2. Requirement Mapping

| PRD Requirement | OpenSpec Task | Verification Case | Status |
|-----------------|---------------|-------------------|--------|
| Brooks findings are recorded with Symptom / Source / Consequence / Remedy | T1, T2 | FC-01 | Covered |
| `homie-om7` closes planning only and Phase 1-4 use child Beads | T1 | FC-02 | Covered |
| Existing PRD/spec relationships are mapped | T1 | FC-03 | Covered |
| FC-01 through FC-08 exist and are executable | T3 | FC-04 | Covered |
| OpenSpec files exist and are non-empty | T4 | FC-05 | Covered |
| OpenSpec maps findings to tasks and cases | T4 | FC-06 | Covered |
| Documentation formatting is clean | T2, T3, T4, T5, T6 | FC-07 | Covered |
| Phase 0 contains no production code changes | T5, T6 | FC-08 | Covered |

## 3. Brooks Finding Mapping

| Brooks Finding | PRD Section | Phase | Future Child Bead |
|----------------|-------------|-------|-------------------|
| GPUI feature containers carry too many reasons to change | 2.1 | Phase 1 / 2 | `inspector-artifacts-module-extraction`, `terminal-pane-logic-slice-extraction` |
| RootView remains too broad as shell composition root | 2.2 | Later GPUI child phase | Deferred until Inspector/Terminal seams exist |
| ControlServer is both dispatcher and runtime coordinator | 2.3 | Phase 3 | `control-server-method-family-extraction` |
| Rust / Swift protocol and manifest mirrors drift | 2.4 | Phase 4 | `protocol-parity-quality-gate` |

## 4. Phase Boundary Check

- Phase 0 deliverables are PRD, OpenSpec, functional cases, and evidence.
- Phase 1-4 implementation work is explicitly out of scope for `homie-om7`.
- Any code extraction requires a separate child Bead and dev-loop.

## 5. Conclusion

The OpenSpec tasks align with the reviewed PRD and cover all Phase 0 verification cases. No unmapped P1 review issue remains.
