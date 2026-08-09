# Diri Parity Child Tasks

> Change ID: `diri-parity-child-tasks`

## Task Mapping

| Task | Scope | Functional cases | Beads |
|------|-------|------------------|-------|
| T-001 | Generate row-level child task matrix | FC-DPCT-001 | `homie-h7n.1`..`homie-h7n.5` |
| T-002 | Create/verify group Beads for all incomplete rows | FC-DPCT-002 | `homie-h7n.1`..`homie-h7n.5` |
| T-003 | Validate no incomplete row lacks owner/case/evidence | FC-DPCT-003 | `homie-h7n.1`..`homie-h7n.5` |
| T-004 | Validate parity lock remains truthful | FC-DPCT-004 | `homie-h7n.1`..`homie-h7n.5` |

## Tasks

### T-001: Row-level child task matrix

Objective:

- Produce `docs/verification/diri-parity-child-tasks/child-task-matrix.md`.
- Include every `partial`, `missing`, or `blocked` row from `docs/research/diri-parity-lock.md`.

Acceptance:

- FC-DPCT-001 passes.

### T-002: Beads group tracking

Objective:

- Create or reuse one Beads issue per execution group.
- Record Beads ids in the matrix.

Acceptance:

- FC-DPCT-002 passes.

### T-003: No unowned incomplete rows

Objective:

- Every incomplete row has owner, OpenSpec task, functional case, evidence gate, and completion rule.

Acceptance:

- FC-DPCT-003 passes.

### T-004: Truthful parity lock

Objective:

- `make parity-lock` remains valid.
- No row is marked implemented by matrix ownership alone.

Acceptance:

- FC-DPCT-004 passes.

