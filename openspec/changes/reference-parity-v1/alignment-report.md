# Homie Reference Parity V1 OpenSpec Alignment Report

```yaml
change_id: reference-parity-v1
report_type: openspec-alignment
status: pass
beads: homie-h7n
source_prd: prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md
openspec_plan: openspec/changes/reference-parity-v1/plan.md
openspec_tasks: openspec/changes/reference-parity-v1/tasks.md
```

## 1. Requirement To Task Mapping

| PRD requirement | Priority | OpenSpec task | Component spec | Verification | Status |
|-----------------|----------|---------------|----------------|--------------|--------|
| FR-1 Reference 功能覆盖矩阵 | P0 | T-001 | `specs/README.md` | coverage/spec review | draft |
| FR-2 Homie 架构边界 | P0 | T-001, T-002, T-004, T-005 | all core specs | component impact review | draft |
| FR-3 Agent catalog parity | P0 | T-003 | `specs/agent-adapter-contract/README.md` | manifest/golden tests | draft |
| FR-4 Session runtime parity | P0 | T-002, T-004, T-005 | `specs/runtime-supervisor/README.md` | runtime E2E | draft |
| FR-5 Protocol/event parity | P0 | T-002 | `specs/runtime-supervisor/README.md` | protocol contract tests | draft |
| FR-6 Terminal parity | P0 | T-006 | `specs/desktop-shell/README.md` | terminal fixtures + PTY smoke | draft |
| FR-7 UI design parity | P0 | T-007, T-008, T-009 | `specs/desktop-shell/README.md` | screenshot/fidelity gate | draft |
| FR-8 Worktree/project | P1 | T-010 | `specs/session-context-store/README.md` | git fixture + E2E | draft |
| FR-9 History/resume | P1 | T-009 | `specs/session-context-store/README.md` | transcript resume E2E | draft |
| FR-10 Artifact/port/PR/browser | P1 | T-011 | `specs/mcp-automation/README.md` | artifact/browser E2E | draft |
| FR-11 Usage/cost/cache | P1 | T-012 | `specs/llm-proxy/README.md` | usage fixture + proxy tests | draft |
| FR-12 LLM custody | P0 | T-012, T-017 | `specs/virtual-key-credentials/README.md` | no-leak + virtual key tests | draft |
| FR-13 Remote/node/handoff | P2 | T-014 | `specs/remote-node-handoff/README.md` | node/handoff harness | draft |
| FR-14 CLI/hook/MCP | P1 | T-013 | `specs/mcp-automation/README.md` | CLI/MCP E2E | draft |
| FR-15 Resource/perf strategy | P1 | T-004, T-016 | `specs/runtime-supervisor/README.md` | resource/perf report | draft |
| FR-16 Packaging/updater | P0 | T-016 | `specs/packaging-updater/README.md` | updater old-to-new | draft |
| FR-17 Packaged perf gate | P0 | T-016 | `specs/packaging-updater/README.md` | packaged perf gate | draft |
| FR-18 Storage/preferences | P0 | T-005 | `specs/storage-indexing/README.md` | migration/repository tests | draft |
| FR-19 Security/privacy | P0 | T-012, T-014, T-016, T-017 | all security specs | security gauntlet | draft |
| FR-20 Context/memory/task/orchestration | P1 | T-015 | context/memory/task/orchestrator specs | controller contract tests | draft |

## 2. Component Spec Impact

| Component spec | Impact | Evidence | Status |
|----------------|--------|----------|--------|
| `specs/desktop-shell/README.md` | yes | PRD FR-6/FR-7/FR-10 and tasks T-006/T-007/T-008/T-009 | created for Reference parity; keep updated during implementation |
| `specs/runtime-supervisor/README.md` | yes | PRD FR-4/FR-5/FR-15 and tasks T-002/T-004/T-006 | created for Reference parity; keep updated during implementation |
| `specs/agent-adapter-contract/README.md` | yes | PRD FR-3 and task T-003 | created for Reference parity; keep updated during implementation |
| `specs/llm-proxy/README.md` | yes | PRD FR-11/FR-12 and task T-012 | created for Reference parity; keep updated during implementation |
| `specs/virtual-key-credentials/README.md` | yes | PRD FR-12/FR-13/FR-19 and tasks T-012/T-014/T-017 | created for Reference parity; keep updated during implementation |
| `specs/session-context-store/README.md` | yes | PRD FR-4/FR-8/FR-9/FR-20 and tasks T-010/T-015 | created for Reference parity; keep updated during implementation |
| `specs/storage-indexing/README.md` | yes | PRD FR-18 and task T-005 | created for Reference parity; keep updated during implementation |
| `specs/observability/README.md` | yes | PRD FR-10/FR-11/FR-15/FR-19 and tasks T-011/T-012/T-017 | created for Reference parity; keep updated during implementation |
| `specs/task-controller/README.md` | yes | PRD FR-20 and task T-015 | created for Reference parity; keep updated during implementation |
| `specs/memory-controller/README.md` | yes | PRD FR-20 and task T-015 | created for Reference parity; keep updated during implementation |
| `specs/intent-orchestrator/README.md` | yes | PRD FR-20 and task T-015 | created for Reference parity; keep updated during implementation |
| `specs/packaging-updater/README.md` | yes | PRD FR-16/FR-17 and task T-016 | created for Reference parity; keep updated during implementation |
| `specs/remote-node-handoff/README.md` | yes | PRD FR-13 and task T-014 | created for Reference parity; keep updated during implementation |
| `specs/mcp-automation/README.md` | yes | PRD FR-10/FR-14 and tasks T-011/T-013 | created for Reference parity; keep updated during implementation |

## 3. Beads Alignment

| Bead | Title | Status | Spec ID | Expected state |
|------|-------|--------|---------|----------------|
| `homie-h7n` | Reference parity V1 product spec | in progress | `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md` | Stay open until spec review, component specs, implementation and evidence complete |

## 4. Coverage Checks

| Check | Result | Evidence |
|-------|--------|----------|
| Every PRD FR has at least one task | pass | Section 1 maps FR-1 through FR-20 |
| Every task has a test or verification path | pass | `openspec/changes/reference-parity-v1/tasks.md` includes RED/GREEN/Acceptance |
| Every affected component spec is updated or explicitly marked no impact | pass | Section 2 marks all affected specs and required updates |
| No unowned security/credential impact remains | pass | FR-12/FR-19 map to T-012/T-014/T-016/T-017 |
| Beads state matches delivery state | pass | `homie-h7n` is claimed/in progress for draft spec work |

## 5. Risks And Follow-Ups

| Risk | Source | Mitigation | Follow-up bead |
|------|--------|------------|----------------|
| Scope is too large for one implementation branch | Full Reference parity | Keep this PRD as umbrella; split T-001..T-017 into dependent Beads before coding | Create child Beads during T-001 |
| Reference remote/node credential assumptions differ from Homie LLM custody | FR-12/FR-13 | Component specs must define Homie virtual-key-safe remote policy before implementation | Covered by T-014 |
| UI fidelity may conflict with GPUI/macOS constraints | FR-7 | Deterministic preview fixtures and intentional-deviation signoff | Covered by T-007/T-008 |
| Existing Homie component specs are incomplete | Section 2 | Update specs before code changes; block implementation if specs are absent | Covered by T-001 |
| Packaged updater requires real signing/notarization environment | FR-16 | Mark release gate blocked until real Developer ID/notary evidence exists | Covered by T-016 |

## 6. Gate Decision

Decision: pass

Reason:

- The PRD requirements FR-1 through FR-20 all map to OpenSpec tasks.
- Component spec impact is explicit and blocks implementation work until long-lived contracts are updated.
- Security and Homie-specific credential custody are represented as first-class tasks rather than hidden follow-up work.
- The current change is a spec/design change; implementation, tests, and release gates remain draft until subsequent execution tasks run.

