# Architecture Audit Hardening Plan

## 1. Scope

This OpenSpec change implements Phase 0 of `architecture-audit-hardening`.
It turns Brooks architecture audit findings into durable planning artifacts.

In scope:

- PRD/spec remediation after review-spec findings;
- functional verification case design;
- OpenSpec task and alignment mapping;
- verification evidence for Phase 0.

Out of scope:

- Inspector code extraction;
- TerminalPane code extraction;
- ControlServer method-family extraction;
- Swift/Rust protocol fixture implementation;
- any change under `homie/crates`, `Sources`, or `Tests`.

## 2. Brooks Finding Map

| Finding | PRD Section | Phase | Child Bead Strategy |
|---------|-------------|-------|---------------------|
| GPUI feature containers carry too many reasons to change | 2.1 | Phase 1 / 2 | Create focused child Beads for Inspector and TerminalPane slices |
| RootView remains too broad as shell composition root | 2.2 | Later GPUI child phase | Create child Bead after Inspector / TerminalPane extraction creates stable seams |
| ControlServer is both dispatcher and runtime coordinator | 2.3 | Phase 3 | Create Engine child Bead for one low-risk method family first |
| Rust / Swift protocol and manifest mirrors drift | 2.4 | Phase 4 | Reuse protocol golden fixtures PRD and add quality gate integration |

## 3. Delivery Strategy

Phase 0 is documentation-only by design. It is complete when:

1. the parent PRD states that `homie-om7` closes planning only;
2. review-spec findings are fixed in the PRD;
3. FC-01 through FC-08 exist and are executable;
4. OpenSpec tasks map findings to cases;
5. verification reports prove the mapping.

## 4. Verification

Verification uses `docs/verification/architecture-audit-hardening/functional-cases.md`.

Key gates:

- FC-01 to FC-04: PRD completeness;
- FC-05 to FC-06: OpenSpec completeness and alignment;
- FC-07: formatting gate;
- FC-08: no production code changes in Phase 0.
