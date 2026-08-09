# Diri Observability/Event/Evidence Parity Phase 1 Alignment Report

```yaml
change_id: diri-observability
beads: homie-wm7
report_type: openspec-alignment
status: pass
source_prd: prd-spec/features/diri-observability/2026-08-07-diri-observability-design.md
component_spec: specs/observability/README.md
openspec_plan: openspec/changes/diri-observability/plan.md
openspec_tasks: openspec/changes/diri-observability/tasks.md
functional_cases: docs/verification/diri-observability/functional-cases.md
```

## 1. Requirement To Task Mapping

| PRD requirement | Priority | OpenSpec task | Component spec | Verification | Status |
|-----------------|----------|---------------|----------------|--------------|--------|
| FR-1 Safe field whitelist | P0 | T-002 | `specs/observability/README.md` | FC-OBS-001 | aligned |
| FR-2 Diri EventBus parity mapping | P0 | T-003 | `specs/observability/README.md` | FC-OBS-002 | aligned |
| FR-3 Metrics write failure contract | P1 | T-004 | `specs/observability/README.md` | FC-OBS-003 | aligned |
| FR-4 Usage evidence projection | P1 | T-005 | `specs/observability/README.md` | FC-OBS-004 | aligned |
| FR-5 Evidence helper model | P1 | T-006 | `specs/observability/README.md` | FC-OBS-005 | aligned |
| FR-6 Component spec and OpenSpec traceability | P0 | T-001, T-007 | `specs/observability/README.md` | FC-OBS-006, final reports | aligned |

## 2. Task To Evidence Mapping

| Task | RED evidence | GREEN evidence | Functional case |
|------|--------------|----------------|-----------------|
| T-001 | Spec review report identifies missing contracts | Component spec updated and traceability files exist | FC-OBS-006 |
| T-002 | `safe_fields` tests fail before whitelist projector | `safe_fields` tests pass | FC-OBS-001 |
| T-003 | `events` tests fail before event model | `events` tests pass | FC-OBS-002 |
| T-004 | `metrics` tests fail before failure projection | `metrics` tests pass | FC-OBS-003 |
| T-005 | `usage` tests fail before usage projection | `usage` tests pass | FC-OBS-004 |
| T-006 | `evidence` tests fail before evidence model | `evidence` tests pass | FC-OBS-005 |
| T-007 | Verification report starts not_run | Functional verification, code review, release readiness recorded | FC-OBS-001 through FC-OBS-006 |

## 3. Component Spec Impact

| Component spec | Impact | Decision |
|----------------|--------|----------|
| `specs/observability/README.md` | direct | Update in this change |
| `specs/runtime-supervisor/README.md` | future consumer | Read-only in this change |
| `specs/llm-proxy/README.md` | future consumer | Read-only in this change |
| `specs/mcp-automation/README.md` | future consumer | Read-only in this change |
| `specs/desktop-shell/README.md` | future consumer | Read-only in this change |

## 4. Beads Alignment

| Bead | Title | Status | Spec ID | Expected state |
|------|-------|--------|---------|----------------|
| `homie-wm7` | Dev loop observability parity | open | `prd-spec/features/diri-observability/2026-08-07-diri-observability-design.md` | remain open until FC-OBS-001 through FC-OBS-006 and release readiness are recorded |

## 5. Coverage Checks

| Check | Result | Evidence |
|-------|--------|----------|
| Every PRD FR has at least one OpenSpec task | pass | Section 1 maps FR-1 through FR-6 |
| Every OpenSpec task has a functional case | pass | `tasks.md` task mapping table |
| Every P0/P1 requirement has executable verification | pass | FC-OBS-001 through FC-OBS-006 specify commands and expected outputs |
| Component spec update is scoped | pass | Only `specs/observability/README.md` is direct write target |
| Implementation scope avoids other lanes | pass | Plan non-goals exclude runtime/LLM/storage/client/CLI edits |
| Security-sensitive fields have explicit handling | pass | FR-1, FR-3, FR-4, FC-OBS-001, FC-OBS-003, FC-OBS-004 |

## 6. Risks And Follow-Ups

| Risk | Source | Mitigation | Follow-up |
|------|--------|------------|-----------|
| New crate is not in root workspace | Spec review P2 | Verify via `--manifest-path`; record as residual risk | Future observability integration PRD decides workspace registration |
| Runtime/client event bus not implemented | Non-goal | Keep this phase as contract/model only | Runtime/client lane consumes T-003 contract |
| LLM proxy usage metrics not wired | Non-goal | Keep usage projection pure | `diri-usage-accounting` lane consumes T-005 contract |
| Field whitelist may need expansion as modules integrate | Expected evolution | Unknown fields default drop; future additions require spec update and tests | Future PRDs update whitelist explicitly |

## 7. Gate Decision

Decision: pass

Reason:

- PRD, spec review, functional cases, OpenSpec plan, and tasks are aligned.
- All P0/P1 requirements have functional verification coverage.
- Implementation may start with T-001, then T-002 through T-006 under SDD/TDD.
