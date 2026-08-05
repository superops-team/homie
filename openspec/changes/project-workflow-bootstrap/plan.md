# Project Workflow Bootstrap OpenSpec Plan

> Change ID: `project-workflow-bootstrap`
> Source PRD: `prd-spec/features/project-workflow-bootstrap/2026-08-05-project-workflow-bootstrap-design.md`
> Beads: `homie-rif`
> Status: complete

## 1. Summary

Initialize Homie's development workflow scaffolding by adding PRD spec, component spec, OpenSpec, verification report, and Beads requirements-management conventions.

This change intentionally avoids Rust/GPUI source initialization. It only creates the workflow foundation needed before implementation work begins.

## 2. Goals

| Goal | Source requirement | Acceptance |
|------|--------------------|------------|
| G-1 | FR-1 | `prd-spec/README.md` defines feature/refactor/bugfix document rules |
| G-2 | FR-2 | `specs/README.md` defines component spec boundaries and Homie component plan |
| G-3 | FR-3 | `openspec/README.md` and templates define change planning flow |
| G-4 | FR-4 | `docs/verification/report-templates/` contains reusable evidence templates |
| G-5 | FR-5 | Beads is initialized and workflow docs explain issue linking |
| G-6 | FR-6 | `AGENTS.md` includes the repo workflow requirements |

## 3. Non-Goals

- Do not initialize a Rust workspace.
- Do not add GPUI application code.
- Do not implement agent adapters, LLM proxy, context, memory, task, or orchestrator code.
- Do not add CI or quality scripts before the Rust project structure exists.

## 4. Affected Component Specs

| Component spec | Impact | Required update |
|----------------|--------|-----------------|
| `specs/README.md` | yes | Initialize the component spec system and component planning table |
| `specs/<component>/README.md` | no | No concrete component contract is created in this change |

## 5. Implementation Scope

| Area | Files/modules | Reason |
|------|---------------|--------|
| PRD spec | `prd-spec/README.md`, `prd-spec/features/project-workflow-bootstrap/...` | Define demand-design workflow and source PRD |
| Component spec | `specs/README.md` | Define long-term component contract rules |
| OpenSpec | `openspec/README.md`, `openspec/templates/*`, `openspec/changes/project-workflow-bootstrap/*` | Define and prove change-planning workflow |
| Verification | `docs/verification/report-templates/*` | Provide evidence report templates |
| Workflow docs | `docs/workflows/requirements-management.md` | Document Beads + PRD/spec/OpenSpec usage |
| Repo guidance | `AGENTS.md`, `README.md` | Make workflow discoverable to future agents and developers |
| Beads | `.beads/` | Initialize issue tracking |

## 6. Data, State, and Security Impact

| Topic | Impact | Handling |
|-------|--------|----------|
| Credential / virtual key | none | No credential-handling code is introduced |
| Session context | none | No runtime state is introduced |
| Memory | none | No memory store is introduced |
| Task state | yes | Beads becomes the local issue and task state system |
| Observability | none | Verification report templates are created, but no runtime telemetry exists yet |

## 7. Test Strategy

| Layer | Required cases | Command or evidence |
|-------|----------------|---------------------|
| Beads smoke | Beads database initializes and reports zero issues before seed creation | `bd status --json` |
| Documentation existence | Required directories and README/template files exist | file inspection |
| Workflow consistency | AGENTS/README/workflow docs agree on PRD/spec/OpenSpec/Beads responsibilities | manual review |
| Rust tests | Not applicable | Rust project not initialized |

## 8. Release Gates

- `bd status --json` works.
- `prd-spec/`, `specs/`, `openspec/`, and `docs/verification/` are present.
- `AGENTS.md` documents the workflow rules.
- No real secrets or provider credentials are introduced.
