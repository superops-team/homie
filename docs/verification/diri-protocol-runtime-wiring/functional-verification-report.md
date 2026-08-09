# Diri Protocol Runtime Wiring Functional Verification Report

```yaml
change_id: diri-protocol-runtime-wiring
beads: homie-qci
status: pass
validated_at: 2026-08-07
```

## Summary

This change implements the protocol/runtime client slice required to close `API-002` in the Diri parity lock:

- `homie-client` now handles external `homie_proto::ControlMessage` requests.
- `events.subscribe` emits runtime event frames and a cursor response over an NDJSON stream.
- `events.wait` uses shared timeout semantics through `HomieClient`.
- `homie-cli` now exposes `homie control-stdio`.
- `homie-cli session create/list/snapshot/kill` use `HomieClient` and real holder-owned runtime sessions instead of storage-only session records.

`API-003` remains `partial`; this change adds session/control evidence but does not complete worktree, ports, or full MCP bridge E2E.

## Functional Cases

| Case | Command | Result | Evidence |
|------|---------|--------|----------|
| FC-DPRW-001 | `cargo test -p homie-client control_stream_subscribe_emits_event_frames_and_cursor_response -- --nocapture` | pass | Event frame plus cursor response verified against real runtime event ring |
| FC-DPRW-002 | `cargo test -p homie-client --tests -- --nocapture` | pass | 7 tests passed, including spawn/send/resize/snapshot, event resume, wait timeout, protocol dispatch, and control stream subscribe |
| FC-DPRW-003 | `cargo test -p homie-cli --test control_stdio_cli -- --nocapture` | pass | `homie control-stdio` accepts NDJSON `ControlMessage::Request(events.subscribe)` and emits event/response frames |
| FC-DPRW-004 | `cargo test -p homie-cli --test session_snapshot_cli -- --nocapture` | pass | CLI `session create` starts a live holder-backed PTY; `session snapshot` reports running holder; `session kill` cleans it up |
| FC-DPRW-005 | `cargo test -p homie-proto --tests -- --nocapture` | pass | Session runtime DTOs include `SessionKillRequest`; protocol contract still passes |
| FC-DPRW-006 | `cargo test -p homie-cli --tests -- --nocapture` | pass | 8 CLI tests passed across parser, hook/notify, control-stdio, events, and session snapshot |
| FC-DPRW-007 | `cargo test -p homie-app --tests -- --nocapture` | pass | App regression remains wired through `HomieClient` |
| FC-DPRW-008 | `cargo clippy -p homie-proto -p homie-client -p homie-cli --all-targets -- -D warnings` | pass | touched protocol/client/CLI crates pass clippy |

## Gate Decision

Decision: pass

Reason:

- `API-002`'s previously recorded remaining gap, external subscription transport, now has real code and integration tests.
- CLI no longer creates storage-only sessions for `session create`.
- `API-003` is intentionally kept `partial` because worktree/ports/MCP bridge E2E remains outstanding.

