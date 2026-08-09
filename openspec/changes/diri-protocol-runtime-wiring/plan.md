# Diri Protocol Runtime Wiring OpenSpec Plan

> Change ID: `diri-protocol-runtime-wiring`  
> Beads: `homie-qci`  
> Source PRD: `prd-spec/features/diri-protocol-runtime-wiring/2026-08-07-diri-protocol-runtime-wiring-design.md`  
> Status: `in_progress`

## 1. Summary

Implement the next protocol/client runtime parity slice for Diri alignment. This change adds a real external NDJSON `ControlMessage` transport on top of `HomieClient`, routes CLI session lifecycle commands through runtime-backed client calls, and records evidence for `API-002` and `API-003`.

## 2. Scope

In scope:

- `homie-client` control-message dispatcher and NDJSON stream serving.
- `events.subscribe` event frame emission and cursor response.
- `events.wait` timeout semantics in the shared client request handler.
- `homie control-stdio` CLI entrypoint.
- CLI session create/list/snapshot runtime-backed path.
- Tests and evidence updates.

Out of scope:

- Worktree, ports, and full MCP bridge parity for `API-003`.
- UI screenshot/E2E parity for `UI-001` and `UI-003`.
- Remote node or production daemonization.

## 3. Component Impact

| Component | Impact |
|-----------|--------|
| `homie-proto` | Reuse existing `ControlMessage`, `EventsSubscribeRequest`, `EventsWaitRequest`, and method catalog. |
| `homie-client` | Add transport-facing dispatcher and shared wait/subscribe behavior. |
| `homie-cli` | Add `control-stdio`; route session commands through `HomieClient`. |
| `homie-runtime` | No new ownership boundary; existing event ring and session APIs are used. |
| `docs/research/diri-parity-lock.md` | Update evidence; mark only `API-002` implemented if all gates pass. |

## 4. Verification Cases

| Case | Purpose | Command |
|------|---------|---------|
| FC-DPRW-001 | Client transport emits response and event frames for `events.subscribe` | `cargo test -p homie-client control_stream_subscribe_emits_event_frames_and_cursor_response -- --nocapture` |
| FC-DPRW-002 | Client dispatcher waits with timeout for `events.wait` | `cargo test -p homie-client client_dispatches_protocol_runtime_methods -- --nocapture` |
| FC-DPRW-003 | CLI `control-stdio` accepts NDJSON control messages | `cargo test -p homie-cli --test control_stdio_cli -- --nocapture` |
| FC-DPRW-004 | CLI session create/snapshot uses runtime-backed client | `cargo test -p homie-cli --test session_snapshot_cli -- --nocapture` |
| FC-DPRW-005 | Parity lock remains truthful | `make parity-lock` |

## 5. Release Gate

The change can close only after all FC-DPRW cases pass, clippy passes for touched crates, and LoopX receives a validated-progress writeback.

