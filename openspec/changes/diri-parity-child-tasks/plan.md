# Diri Parity Child Tasks OpenSpec Plan

> Change ID: `diri-parity-child-tasks`  
> Source PRD: `prd-spec/features/diri-parity-child-tasks/2026-08-07-diri-parity-child-tasks-design.md`  
> Source lock: `docs/research/diri-parity-lock.md`  
> Status: `in_progress`

## 1. Summary

This change turns every remaining non-implemented Diri parity lock row into an executable child task with Beads ownership and a verification gate. It is a governance and execution-safety change: it prevents rows from being marked `implemented` unless the row has real code evidence and a recorded functional verification path.

## 2. Execution Groups

| Group | Scope | Primary owners |
|-------|-------|----------------|
| G-UI | UI-001..UI-009, TERM-002, TERM-004, TERM-005 | `homie-app`, `homie-ui`, `homie-term`, `homie-client` |
| G-RUNTIME | RT-004, RT-005, RT-009, RT-010, AG-002..AG-004 | `homie-runtime`, `homie-agents`, `homie-storage` |
| G-PROTOCOL | API-001..API-005 | `homie-proto`, `homie-client`, `homie-cli`, `homie-orchestrator` |
| G-AUTOMATION | ART-001..ART-003, GIT-001..GIT-002, AUTO-001 | `homie-runtime`, `homie-app`, `homie-cli`, `homie-orchestrator` |
| G-REMOTE-RELEASE | REM-001..REM-003, USAGE-001, UPDATE-001, PKG-001, PERF-001 | `homie-remote`, `homie-llm`, `homie-updater`, `scripts/package` |

## 3. Gates

- `docs/verification/diri-parity-child-tasks/child-task-matrix.md` covers every lock row whose status is not `implemented`.
- Every row has a Beads id, OpenSpec task id, functional case id, and required evidence.
- `make parity-lock` remains valid.
- `loopx check` remains clean.

