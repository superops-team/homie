# Diri Parity Child Tasks Verification Report

```yaml
change_id: diri-parity-child-tasks
report_type: verification
status: pass
source_lock: docs/research/diri-parity-lock.md
matrix: docs/verification/diri-parity-child-tasks/child-task-matrix.md
```

## Summary

The current non-implemented Diri parity lock rows have been converted into row-level child tasks with Beads ownership, OpenSpec task ids, functional case ids, required evidence, and completion rules.

## Beads

| Group | Bead | Status |
|-------|------|--------|
| G-UI | `homie-h7n.4` | created |
| G-RUNTIME | `homie-h7n.2` | created |
| G-PROTOCOL | `homie-h7n.1` | created |
| G-AUTOMATION | `homie-h7n.3` | created |
| G-REMOTE-RELEASE | `homie-h7n.5` | created |

## Verification

| Gate | Command | Result |
|------|---------|--------|
| Matrix coverage | Python matrix validation against `docs/research/diri-parity-lock.md` | pass, 36 rows after `API-002` was closed by `diri-protocol-runtime-wiring` |
| Beads group ownership | `bd list --json` group id validation | pass, 5 groups |
| Parity lock | `make parity-lock` | pass, incomplete rows still listed |
| LoopX contract | `loopx --registry .loopx/registry.json check --scan-root /Users/bytedance/workspace/github/homie` | pass |
| Format | `cargo fmt --all -- --check` | pass |

## Completion Rule

No row in `docs/research/diri-parity-lock.md` may be changed to `implemented` merely because it is present in the matrix or has a Beads owner. The row must pass the required evidence command in `child-task-matrix.md`, and the evidence path must be recorded in the parity lock.

## Gate Decision

Decision: pass

Reason:

- All 36 current non-implemented rows are represented in the child task matrix.
- Each row has group ownership, Beads id, OpenSpec task id, functional case id, required evidence, and a completion rule.
- Existing incomplete rows remain incomplete in the parity lock.

