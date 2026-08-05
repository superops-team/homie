# <Change Title> OpenSpec Plan

> Change ID: `<change-id>`
> Source PRD: `prd-spec/<type>/<topic>/YYYY-MM-DD-<description>.md`
> Beads: `<bead-id>`
> Status: draft | reviewed | in_progress | complete

## 1. Summary

Describe the implementation intent in one or two paragraphs.

## 2. Goals

| Goal | Source requirement | Acceptance |
|------|--------------------|------------|
| G-1 | FR-1 | ... |

## 3. Non-Goals

- ...

## 4. Affected Component Specs

| Component spec | Impact | Required update |
|----------------|--------|-----------------|
| `specs/<component>/README.md` | yes/no | ... |

## 5. Implementation Scope

| Area | Files/modules | Reason |
|------|---------------|--------|
| ... | ... | ... |

## 6. Data, State, and Security Impact

| Topic | Impact | Handling |
|-------|--------|----------|
| Credential / virtual key | ... | ... |
| Session context | ... | ... |
| Memory | ... | ... |
| Task state | ... | ... |
| Observability | ... | ... |

## 7. Test Strategy

| Layer | Required cases | Command or evidence |
|-------|----------------|---------------------|
| Unit | ... | ... |
| Integration | ... | ... |
| E2E/manual | ... | ... |
| Security | ... | ... |

## 8. Release Gates

- `docs/verification/<change-id>/spec-review-report.md` is pass.
- `docs/verification/<change-id>/openspec-alignment-report.md` is pass.
- Required Rust checks and tests pass.
- Security-sensitive paths are covered by regression tests.
- Beads issue state matches the actual delivery status.
