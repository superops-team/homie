# <Change Title> OpenSpec Alignment Report

```yaml
change_id: <change-id>
report_type: openspec-alignment
status: draft
beads: <bead-id>
source_prd: prd-spec/<type>/<topic>/YYYY-MM-DD-<description>.md
openspec_plan: openspec/changes/<change-id>/plan.md
openspec_tasks: openspec/changes/<change-id>/tasks.md
```

## 1. Requirement To Task Mapping

| PRD requirement | Priority | OpenSpec task | Component spec | Verification | Status |
|-----------------|----------|---------------|----------------|--------------|--------|
| FR-1 | P0 | T-001 | `specs/<component>/README.md` | unit/integration/e2e | draft |

## 2. Component Spec Impact

| Component spec | Impact | Evidence | Status |
|----------------|--------|----------|--------|
| `specs/<component>/README.md` | yes/no | reason or update link | draft |

## 3. Beads Alignment

| Bead | Title | Status | Spec ID | Expected state |
|------|-------|--------|---------|----------------|
| `<bead-id>` | ... | open/in_progress/closed | `prd-spec/...` | ... |

## 4. Coverage Checks

| Check | Result | Evidence |
|-------|--------|----------|
| Every PRD FR has at least one task | pass/fail | ... |
| Every task has a test or verification path | pass/fail | ... |
| Every affected component spec is updated or explicitly marked no impact | pass/fail | ... |
| No unowned security/credential impact remains | pass/fail | ... |
| Beads state matches delivery state | pass/fail | ... |

## 5. Risks And Follow-Ups

| Risk | Source | Mitigation | Follow-up bead |
|------|--------|------------|----------------|
| ... | ... | ... | ... |

## 6. Gate Decision

Decision: pass | blocked | not_run

Reason:

- ...
