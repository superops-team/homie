# Diri Protocol Runtime Wiring Release Readiness

```yaml
change_id: diri-protocol-runtime-wiring
beads: homie-qci
status: ready_for_next_loopx_slice
```

## Delivered

- Added `HomieClient::handle_control_message` and `HomieClient::serve_control_stream`.
- Added external NDJSON `ControlMessage` transport coverage for `events.subscribe`.
- Moved `events.wait` timeout semantics into the shared client request path.
- Added `homie control-stdio`.
- Routed CLI session create/list/snapshot through `HomieClient`.
- Added `homie session kill` and `SessionKillRequest` for runtime-backed cleanup.
- Prevented holder subprocesses from inheriting CLI/test stdio pipes.

## Parity Impact

| Row | Decision | Reason |
|-----|----------|--------|
| API-002 | implemented | Runtime client now covers spawn/list/attach/send/resize, event resume, wait timeout, and external subscription transport. |
| API-003 | partial | CLI session/control paths improved, but worktree/ports/full MCP bridge E2E remains open. |
| UI-001/UI-003 | partial | App/client wiring evidence remains valid, but GPUI interaction and screenshot gates are still pending. |

## Verification

See `docs/verification/diri-protocol-runtime-wiring/functional-verification-report.md`.

