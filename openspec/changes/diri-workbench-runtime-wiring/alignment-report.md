# Diri Workbench Runtime Wiring Alignment Report

```yaml
change_id: diri-workbench-runtime-wiring
status: pass
source_prd: prd-spec/features/diri-workbench-runtime-wiring/2026-08-07-diri-workbench-runtime-wiring-design.md
beads: homie-3tz
```

## Requirement Alignment

| PRD requirement | OpenSpec task | Functional case | Evidence target |
|-----------------|---------------|-----------------|-----------------|
| FR-1 Runtime session projection | T-002 | FC-DWRW-001, FC-DWRW-002 | `cargo test -p homie-app --tests` |
| FR-2 SpawnShell 真实执行 | T-002 | FC-DWRW-001 | app regression test |
| FR-3 Sidebar selection | T-003 | FC-DWRW-002 | app regression test |
| FR-4 Runtime resize | T-003 | FC-DWRW-003 | app regression test |
| FR-5 测试门禁 | T-004 | FC-DWRW-004, FC-DWRW-005 | client tests and parity lock |

## Scope Guard

- `UI-001` and `UI-003` must remain `partial` after this slice.
- This slice advances real runtime wiring, not full visual parity, screenshots, or complete terminal interaction E2E.
- No direct `homie-runtime` or storage session lifecycle writes may be introduced in `homie-app`.

## Gate Decision

Decision: pass

Reason:

- All requirements map to concrete implementation tasks and executable gates.
- Status update rules prevent over-marking UI parity as complete.

