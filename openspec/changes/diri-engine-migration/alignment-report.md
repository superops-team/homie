# diri-engine-migration Gap Closure OpenSpec Alignment Report

```yaml
change_id: diri-engine-migration
report_type: openspec-alignment
status: pass
beads: homie-cj5
source_prd: prd-spec/features/diri-engine-migration/2026-08-06-diri-engine-migration-gap-closure-design.md
openspec_plan: openspec/changes/diri-engine-migration/plan.md
openspec_tasks: openspec/changes/diri-engine-migration/tasks.md
functional_cases: docs/verification/diri-engine-migration/functional-cases.md
```

## 1. Requirement To Task Mapping

| PRD requirement | Priority | OpenSpec task | Component spec | Verification | Status |
|-----------------|----------|---------------|----------------|--------------|--------|
| FR-1 真实 PTY runtime 接线 | P0 | T-002 | `specs/runtime-supervisor/README.md`, `specs/session-context-store/README.md` | FC-DIRI-001, FC-DIRI-002, FC-DIRI-003 | aligned |
| FR-2 Diri status reducer 迁移 | P0 | T-003 | `specs/agent-adapter-contract/README.md`, `specs/observability/README.md` | FC-DIRI-004 | aligned |
| FR-3 Hook/notify parsing | P0 | T-003 | `specs/agent-adapter-contract/README.md`, `specs/observability/README.md` | FC-DIRI-005 | aligned |
| FR-4 Scrollback 真实模型 | P1 | T-004 | `specs/desktop-shell/README.md`, `specs/runtime-supervisor/README.md` | FC-DIRI-006 | aligned |
| FR-5 Design token 完整对齐 | P1 | T-005 | `specs/desktop-shell/README.md` | FC-DIRI-007 | aligned |
| FR-6 Homie app 去占位并呈现 Diri 风格工作台 | P1 | T-006 | `specs/desktop-shell/README.md` | FC-DIRI-008 | aligned |
| FR-7 状态和文档必须与实际交付一致 | P0 | T-001, T-007 | all impacted specs | FC-DIRI-009, final reports | aligned |

## 2. Component Spec Impact

| Component spec | Impact | Evidence | Status |
|----------------|--------|----------|--------|
| `specs/runtime-supervisor/README.md` | yes | live PTY registry, spawn failure, output log, status input semantics | update-required |
| `specs/agent-adapter-contract/README.md` | yes | status reducer, hook parser, stable events, redaction | update-required |
| `specs/desktop-shell/README.md` | yes | Diri preview shell, token parity, no direct runtime ownership | update-required |
| `specs/session-context-store/README.md` | yes | session status/output index/read_output semantics | update-required |
| `specs/observability/README.md` | yes | safe status/hook/runtime process logs | update-required |
| `specs/storage-indexing/README.md` | conditional | only if repository/schema changes are needed for output offsets or session status | conditional |

## 3. Beads Alignment

| Bead | Title | Status | Spec ID | Expected state |
|------|-------|--------|---------|----------------|
| `homie-cj5` | 从 diri 迁移核心引擎到 homie | open | `prd-spec/features/diri-engine-migration/2026-08-06-diri-engine-migration-design.md` | remain open/in progress until FC-DIRI-001 through FC-DIRI-018 pass and release readiness is recorded |

Note: Beads currently points at the original design doc. This gap-closure PRD is an iteration under the same topic and change id. The closeout evidence must reference both the original design and this gap-closure document.

## 4. Coverage Checks

| Check | Result | Evidence |
|-------|--------|----------|
| Every PRD FR has at least one task | pass | Section 1 maps FR-1 through FR-7 to T-001 through T-007 |
| Every task has a test or verification path | pass | `tasks.md` binds every task to FC-DIRI cases and evidence paths |
| Every P0/P1 requirement has functional case coverage | pass | `functional-cases.md` covers FR-1 through FR-7 |
| Every affected component spec is updated or explicitly marked | partial | Impact is identified; spec files still need updates during T-001 |
| No unowned security/credential impact remains | pass | Hook redaction and no raw key policy mapped to T-003 and FC-DIRI-005 |
| Beads state matches delivery state | pass | `homie-cj5` is open; implementation not yet claimed complete |

## 5. Risks And Follow-Ups

| Risk | Source | Mitigation | Follow-up bead |
|------|--------|------------|----------------|
| `homie-client` crate does not exist yet despite architecture docs naming it | Spec review report | Keep `homie-app` as preview-only until client/protocol exists; decide in T-001 whether to add client crate or defer live UI operations | TBD by T-001, not required before runtime internal tests |
| Remote node, MCP, updater Diri parity remains outside this gap closure | PRD non-goals | Record as residual risks in release readiness; do not close broader reference parity claims | TBD after local gap closure |
| GPUI version drift may block full RootView migration | PRD non-goals | Implement Diri-style preview shell and source-text regression first | none for this change |
| Minimal holder-owned PTY survival and detached child kill are implemented, but full Diri holder-manager/resource-governor crash matrix remains incomplete | FR-1 explicit boundary | Claim only verified supervisor drop/reopen adoption, exited/detached restore, and process-tree termination; keep full crash matrix in parity lock | future runtime resilience bead |

## 6. Gate Decision

Decision: pass

Reason:

- The PRD, spec review, functional cases, OpenSpec plan, and OpenSpec tasks are aligned.
- All P0/P1 requirements have task and functional case coverage.
- Implementation may start with T-001, then T-002 through T-006 under SDD/TDD.
- Component spec updates remain required before final closeout and are tracked by T-001.
