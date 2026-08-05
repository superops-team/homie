# Project Workflow Bootstrap OpenSpec Alignment Report

```yaml
change_id: project-workflow-bootstrap
report_type: openspec-alignment
status: pass
beads: homie-rif
source_prd: prd-spec/features/project-workflow-bootstrap/2026-08-05-project-workflow-bootstrap-design.md
openspec_plan: openspec/changes/project-workflow-bootstrap/plan.md
openspec_tasks: openspec/changes/project-workflow-bootstrap/tasks.md
```

## 1. Requirement To Task Mapping

| PRD requirement | Priority | OpenSpec task | Component spec | Verification | Status |
|-----------------|----------|---------------|----------------|--------------|--------|
| FR-1 PRD spec 目录 | P0 | T-002 | `specs/README.md` | file inspection | draft |
| FR-2 组件 spec 目录 | P0 | T-003 | `specs/README.md` | file inspection | draft |
| FR-3 OpenSpec 目录 | P0 | T-004 | `specs/README.md` | file inspection | draft |
| FR-4 验证报告模板 | P1 | T-005 | `specs/README.md` | file inspection | draft |
| FR-5 Beads 需求管理 | P0 | T-001 | `specs/README.md` | `bd status --json` | draft |
| FR-6 Agent 工作规则 | P0 | T-006 | `specs/README.md` | AGENTS/README review | draft |

## 2. Component Spec Impact

| Component spec | Impact | Evidence | Status |
|----------------|--------|----------|--------|
| `specs/README.md` | yes | Initializes component spec system and Homie component planning table | draft |
| Concrete component specs | no | This change creates workflow scaffolding only | draft |

## 3. Beads Alignment

| Bead | Title | Status | Spec ID | Expected state |
|------|-------|--------|---------|----------------|
| homie-rif | Bootstrap Homie PRD/spec/OpenSpec workflow | open | `prd-spec/features/project-workflow-bootstrap/2026-08-05-project-workflow-bootstrap-design.md` | open or closed after scaffold verification |

## 4. Coverage Checks

| Check | Result | Evidence |
|-------|--------|----------|
| Every PRD FR has at least one task | pass | `openspec/changes/project-workflow-bootstrap/tasks.md` |
| Every task has a test or verification path | pass | `openspec/changes/project-workflow-bootstrap/tasks.md` |
| Every affected component spec is updated or explicitly marked no impact | pass | `specs/README.md` |
| No unowned security/credential impact remains | pass | No runtime credential code introduced |
| Beads state matches delivery state | pass | `homie-rif` points to the bootstrap PRD |

## 5. Risks And Follow-Ups

| Risk | Source | Mitigation | Follow-up bead |
|------|--------|------------|----------------|
| Git hook installation failed during `bd init` because `.git/config` could not be locked | `bd init` output | Beads DB works; document rerun command `bd hooks install --beads` after permissions are fixed | none |
| Rust project gates are not defined yet | project is not initialized | Document `not_run` for Rust tests until Cargo workspace exists | future Rust bootstrap bead |

## 6. Gate Decision

Decision: pass

Reason:

- PRD, component spec, OpenSpec, Beads, workflow docs, and verification templates are aligned.
- Rust runtime verification is intentionally not run because this change only initializes workflow documentation and Beads.
- Release evidence is recorded in `docs/verification/project-workflow-bootstrap/release-readiness-report.md`.
