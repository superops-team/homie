# Diri Protocol Runtime Wiring Alignment Report

```yaml
change_id: diri-protocol-runtime-wiring
status: pass
source_prd: prd-spec/features/diri-protocol-runtime-wiring/2026-08-07-diri-protocol-runtime-wiring-design.md
beads: homie-qci
```

## Requirement Alignment

| PRD requirement | OpenSpec task | Functional case | Evidence target |
|-----------------|---------------|-----------------|-----------------|
| FR-1 ControlMessage transport dispatcher | T-001, T-002 | FC-DPRW-001 | `cargo test -p homie-client control_stream_subscribe_emits_event_frames_and_cursor_response` |
| FR-2 外部 events.subscribe | T-001, T-002 | FC-DPRW-001 | Event frames plus cursor response in client transport test |
| FR-3 events.wait timeout 语义 | T-001, T-002 | FC-DPRW-002 | Shared client dispatcher wait test |
| FR-4 CLI runtime-backed session path | T-003 | FC-DPRW-004 | CLI session create/snapshot integration test |
| FR-5 外部入口 | T-003 | FC-DPRW-003 | `homie control-stdio` CLI test |

## Scope Guard

- `API-002` is eligible for `implemented` after this change because the stated remaining gap is external subscription transport.
- `API-003` is not eligible for `implemented` in this change because worktree, ports, and full MCP bridge E2E remain outside scope.
- UI rows `UI-001` and `UI-003` receive indirect evidence only; they must remain `partial` until GPUI interaction E2E and screenshot gates pass.

## Gate Decision

Decision: pass

Reason:

- Every PRD requirement maps to at least one executable OpenSpec task and functional verification case.
- Completion rules explicitly prevent over-marking unrelated parity rows.

