# Diri Workbench Runtime Wiring Release Readiness

```yaml
change_id: diri-workbench-runtime-wiring
beads: homie-3tz
status: ready_for_next_loopx_slice
```

## Delivered

- Runtime session projection in `homie-app`.
- Real command palette `SpawnShell` runtime action.
- Clickable sidebar session selection.
- Selected-session terminal snapshot refresh.
- Runtime terminal geometry resize path.
- Desktop shell spec update for live-connected default mode.

## Parity Impact

| Row | Decision | Reason |
|-----|----------|--------|
| UI-001 | partial | Runtime-backed workbench actions improved, but full Diri workbench E2E/screenshot parity remains open. |
| UI-003 | partial | Terminal snapshot/send/resize paths are wired through `HomieClient`, but find/selection/scrollback and GPUI interaction E2E remain open. |

## Verification

See `docs/verification/diri-workbench-runtime-wiring/functional-verification-report.md`.

