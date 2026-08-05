# Project Workflow Bootstrap OpenSpec Tasks

> Change ID: `project-workflow-bootstrap`
> Source PRD: `prd-spec/features/project-workflow-bootstrap/2026-08-05-project-workflow-bootstrap-design.md`
> Beads: `homie-rif`

## Task Status

| Status | Meaning |
|--------|---------|
| todo | Not started |
| red | Failing test or contract written |
| green | Implementation passes focused verification |
| refactor | Cleanup while tests stay green |
| done | Task evidence recorded and accepted |

## Tasks

### T-001: Initialize Beads

| Field | Value |
|-------|-------|
| Status | green |
| Source requirement | FR-5 |
| Component spec | `specs/README.md` |
| Beads | `homie-rif` |
| Files | `.beads/`, `.gitignore` |

Objective:

- Initialize local Beads issue tracking with the `homie` prefix.

RED:

- Before init, `bd status --json` has no Homie database context.

GREEN:

- Run `bd init --non-interactive --prefix homie --skip-agents --init-if-missing`.
- Confirm `bd status --json` returns a valid status document.

Acceptance:

- `.beads/metadata.json` identifies database `homie`.
- Workflow docs explain how to create and update issues.

### T-002: Add PRD spec workflow

| Field | Value |
|-------|-------|
| Status | green |
| Source requirement | FR-1 |
| Component spec | `specs/README.md` |
| Beads | `homie-rif` |
| Files | `prd-spec/README.md`, `prd-spec/features/project-workflow-bootstrap/...` |

Objective:

- Establish Chinese PRD/spec conventions for features, refactors, and bugfixes.

RED:

- No PRD spec directory or templates exist.

GREEN:

- Add `prd-spec/README.md`.
- Add the workflow bootstrap PRD.

Acceptance:

- PRD docs define classification, naming, templates, and Beads linkage.

### T-003: Add component spec workflow

| Field | Value |
|-------|-------|
| Status | green |
| Source requirement | FR-2 |
| Component spec | `specs/README.md` |
| Beads | `homie-rif` |
| Files | `specs/README.md` |

Objective:

- Define long-term component spec boundaries for Homie.

RED:

- No component spec entrypoint exists.

GREEN:

- Add `specs/README.md` with document layering, component plan, and component spec structure.

Acceptance:

- P0/P1/P2 Homie components are listed with responsibilities.

### T-004: Add OpenSpec workflow

| Field | Value |
|-------|-------|
| Status | green |
| Source requirement | FR-3 |
| Component spec | `specs/README.md` |
| Beads | `homie-rif` |
| Files | `openspec/README.md`, `openspec/templates/*`, `openspec/changes/project-workflow-bootstrap/*` |

Objective:

- Establish per-change implementation planning conventions.

RED:

- No OpenSpec entrypoint, templates, or change directory exist.

GREEN:

- Add OpenSpec README, plan/tasks/alignment templates, and this bootstrap change.

Acceptance:

- OpenSpec docs explain change id, plan, tasks, and alignment requirements.

### T-005: Add verification workflow

| Field | Value |
|-------|-------|
| Status | green |
| Source requirement | FR-4 |
| Component spec | `specs/README.md` |
| Beads | `homie-rif` |
| Files | `docs/verification/report-templates/*` |

Objective:

- Provide a lightweight evidence-report structure for future changes.

RED:

- No report template exists.

GREEN:

- Add report-template README and generic report template.

Acceptance:

- The template supports spec review, OpenSpec alignment, SDD/TDD, tests, E2E, security review, code review, and release readiness.

### T-006: Update repo guidance

| Field | Value |
|-------|-------|
| Status | green |
| Source requirement | FR-6 |
| Component spec | `specs/README.md` |
| Beads | `homie-rif` |
| Files | `AGENTS.md`, `README.md` |

Objective:

- Make the workflow visible from the project README and mandatory for future agents.

RED:

- Existing `AGENTS.md` only has high-level constraints.

GREEN:

- Update `AGENTS.md` with the Beads + PRD/spec + OpenSpec workflow.
- Update `README.md` with links to workflow docs.

Acceptance:

- Future agents can find the workflow without prior chat context.

Evidence:

- `AGENTS.md`
- `README.md`
- `docs/verification/project-workflow-bootstrap/release-readiness-report.md`
