# Typed Agent Driver Capability Alignment Report

## 1. Alignment Summary

- PRD: `prd-spec/features/typed-agent-driver-capabilities/2026-08-13-typed-agent-driver-capabilities-design.md`
- Spec review: `docs/verification/typed-agent-driver-capabilities/spec-review-report.md`
- Functional cases: `docs/verification/typed-agent-driver-capabilities/functional-cases.md`
- Plan: `openspec/changes/typed-agent-driver-capabilities/plan.md`
- Tasks: `openspec/changes/typed-agent-driver-capabilities/tasks.md`

Status: aligned for first-slice implementation.

## 2. PRD Requirement To Task Mapping

| PRD requirement | Task | Case | Status |
|---|---|---|---|
| Define typed capability layer | T2, T3 | FC-03, FC-05 | Covered |
| Preserve manifest/PTY/holder/status reducer architecture | T3, T4 | FC-03, FC-04 | Covered |
| Let callers query session capabilities | T2, T4 | FC-04, FC-05 | Covered |
| Keep real provider integration out of first slice | T1, T3 | FC-01, FC-03 | Covered |
| Security boundary for driver payloads | T3 | FC-03 | Covered |
| Avoid MCP/UI/control action expansion | T1, T4 | FC-01, FC-04 | Covered |

## 3. Case To Task Mapping

| Case | Tasks | Notes |
|---|---|---|
| FC-01 | T1 | Spec/review gate |
| FC-02 | T1 | OpenSpec coverage gate |
| FC-03 | T3 | Driver abstraction and fake driver |
| FC-04 | T4 | Read-only session capability query |
| FC-05 | T2 | Swift/Rust wire compatibility |
| FC-06 | T5 | Static final gate |

## 4. Out-Of-Scope Guard

| Out of scope | Guard |
|---|---|
| Real Codex/Claude/OpenCode driver | No tasks mention provider adapters |
| steer/cancel/model action methods | Only `session.capabilities` is in scope |
| MCP tool changes | No MCP tasks |
| UI changes | No app UI tasks |
| Session status authority changes | T4 verifies query is read-only |

## 5. Verdict

No unmapped P0/P1 requirement remains for the first slice. Implementation can proceed.
