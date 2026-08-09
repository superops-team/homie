## 1. Dependency and Baseline

- [x] 1.1 Capture `homie-client` normal dependency RED evidence showing runtime/storage coupling
- [x] 1.2 Add approved Tokio, SHA-256, and workspace Unix libc dependencies
- [x] 1.3 Resolve `Cargo.lock` and run proto/runtime/client compile check

## 2. Protocol-Owned Runtime DTOs

- [x] 2.1 Add RED JSON contract tests for Hello, errors, streams, runtime events, and current typed-client DTOs
- [x] 2.2 Add `transport`, `stream`, and runtime response DTO modules to `homie-proto`
- [x] 2.3 Add method constants for status/artifacts/ports/lineage/hook/worktree-overview behavior
- [x] 2.4 Move client public response ownership from runtime/storage types to proto types
- [x] 2.5 Pass new and existing proto contract suites

## 3. Binary Frame Codec

- [x] 3.1 Add RED byte-exact preface/header/frame round-trip tests
- [x] 3.2 Implement fixed big-endian preface and 24-byte frame header codec
- [x] 3.3 Add RED partial/coalesced read and hostile-length tests
- [x] 3.4 Enforce 16 MiB frame, 4 MiB control, 64 KiB output, version/kind/flags/stream limits
- [x] 3.5 Pass full transport codec suite

## 4. Runtime Paths and Launcher

- [x] 4.1 Add RED absolute runtime-path and invalid-path tests
- [x] 4.2 Implement owner-path DTO without environment lookup
- [x] 4.3 Add RED live-probe, detached-spawn, incompatible-live-daemon, early-return, and invalid-executable launcher tests
- [x] 4.4 Implement explicit `RuntimeLauncher` separate from client connect
- [x] 4.5 Pass launcher suite

## 5. RuntimeActor

- [x] 5.1 Add RED ordered mutation, correlated reply, queue-full, shutdown, and long-job-isolation actor tests
- [x] 5.2 Add RED one-worker/32-job lane, deadline, cancellation, process cleanup, serialization, and no-partial-commit tests
- [x] 5.3 Implement capacity-256 concrete actor command channel and production `RuntimeSupervisor` adapter
- [x] 5.4 Implement fixed one-worker/32-job `LongRunningLane`
- [x] 5.5 Split handlers into actor prepare, lane execute, and actor commit
- [x] 5.6 Implement bounded cancellable git runner and bounded history scanner
- [x] 5.7 Pass actor, long-running, worktree, and history focused tests

## 6. Exact Capability Registries

- [x] 6.1 Add RED handler/capability, Actor/LongRunning/AsyncWait classification, and stream-opener equality tests
- [x] 6.2 Implement one request handler registry used by Hello and dispatch
- [x] 6.3 Implement exact `events.v1` and `terminal.v1` opener registry
- [x] 6.4 Add handlers for current app/CLI/MCP status/artifact/port/lineage/hook/overview behavior
- [x] 6.5 Prove future proto constants are not advertised automatically

## 7. UDS Handshake and Control

- [x] 7.1 Add RED real-UDS fake-backend Hello/control/error and 64-connection-limit tests
- [x] 7.2 Implement macOS/BSD and Linux peer UID validation
- [x] 7.3 Implement preface-first connection reader and stream-0 demux
- [x] 7.4 Implement correlated high-priority response path
- [x] 7.5 Pass server control tests

## 8. Bounded Writer

- [x] 8.1 Add RED capacity, 32-high quota, low-stream round-robin, and high-queue-overflow tests
- [x] 8.2 Implement bounded high and per-stream low writer queues
- [x] 8.3 Implement isolated `slow_consumer` stream reset
- [x] 8.4 Pass writer fairness/backpressure tests

## 9. Event Stream

- [x] 9.1 Add RED valid replay, 1024-ring gap, overflow, filter, and snapshot-cursor tests
- [x] 9.2 Implement event stream opener and replay/live producer outside actor
- [x] 9.3 Implement consistent `state.snapshot` with event cursor
- [x] 9.4 Pass event stream tests

## 10. Terminal Stream

- [x] 10.1 Add RED replay/full-grid/live ordering and offset tests
- [x] 10.2 Add RED Input/Resize routing, close-survival, sequence-gap, two-stream isolation, and shared-source tests
- [x] 10.3 Implement one shared `TerminalSource` per attached session
- [x] 10.4 Implement 64 KiB replay/absolute offsets and 50 ms active/250 ms idle tail
- [x] 10.5 Implement current HeadlessScreen full-grid conversion with default styles
- [x] 10.6 Pass terminal stream tests

## 11. Daemon Lifecycle

- [x] 11.1 Add RED owner permissions, singleton race, stale socket, Hello/snapshot, signal, and restart process tests
- [x] 11.2 Implement lock-owner-only startup and stale socket cleanup
- [x] 11.3 Implement executable SHA-256 and daemon instance identity
- [x] 11.4 Implement prepare/shutdown/signal drain while preserving holder sessions
- [x] 11.5 Pass serial real-daemon process suite

## 12. Async Client Core

- [x] 12.1 Add RED Hello state, out-of-order response, timeout, cancellation, disconnect, pending-limit, and close tests
- [x] 12.2 Implement `ClientOptions`, connection manager, and state watch
- [x] 12.3 Implement bounded pending request correlation and fail-all
- [x] 12.4 Implement bounded client writer with priority/fairness
- [x] 12.5 Pass request correlation suite

## 13. Client Recovery and Streams

- [x] 13.1 Add RED 25s heartbeat, 10s timeout, 500ms-to-8s backoff, and terminal shutdown tests
- [x] 13.2 Implement heartbeat/reconnect without request replay
- [x] 13.3 Add RED event replay and snapshot recovery tests
- [x] 13.4 Implement bounded EventStream and snapshot replacement
- [x] 13.5 Add RED terminal offset/full-grid/resync tests
- [x] 13.6 Implement bounded TerminalStream and automatic reopen
- [x] 13.7 Pass client recovery and stream suites

## 14. Typed Client Replacement

- [x] 14.1 Add RED typed request tests for every current app/CLI/MCP client operation
- [x] 14.2 Implement async typed facade over generic request/stream APIs only
- [x] 14.3 Delete embedded runtime fields, constructors, dispatcher, and synchronous stream server
- [x] 14.4 Remove normal client dependencies on runtime/storage/unused remote implementation
- [x] 14.5 Pass client suite and negative dependency/source scans

## 15. CLI Migration

- [x] 15.1 Add RED real-daemon CLI session/worktree/history/diff/events/hook tests
- [x] 15.2 Convert runtime-backed CLI commands to Tokio async client calls
- [x] 15.3 Replace `control-stdio` local dispatcher with bounded daemon bridge
- [x] 15.4 Keep doctor/usage storage paths isolated from live runtime projection
- [x] 15.5 Pass focused CLI runtime suites

## 16. MCP Migration

- [x] 16.1 Add RED daemon capability-to-tool filtering tests
- [x] 16.2 Convert `McpRuntimeContext` to async shared-daemon client
- [x] 16.3 Preserve JSON-RPC `-32601` for method-not-found
- [x] 16.4 Pass MCP runtime, orchestration, lineage, artifact, and worktree suites

## 17. GPUI Migration

- [x] 17.1 Add RED runtime bridge state/projection and non-source-text first-frame tests
- [x] 17.2 Implement exact two-worker Tokio owner and explicit launcher startup
- [x] 17.3 Move connect and daemon readiness off GPUI first-frame path
- [x] 17.4 Migrate live session list/spawn/send/resize/artifact/worktree/terminal flows
- [x] 17.5 Preserve T-103 settings storage scope without live-session reads
- [x] 17.6 Pass app bridge, palette, regression, and compile gates

## 18. Package Closure

- [x] 18.1 Add RED bundle verification for fixed runtime-daemon path
- [x] 18.2 Build, copy, chmod, and nested-sign `homie-runtime-daemon`
- [x] 18.3 Add local packaged Hello/snapshot/shared-instance smoke
- [x] 18.4 Pass package verification and `make smoke`

## 19. Cross-Entry and Security E2E

- [x] 19.1 Add RED app/CLI/MCP same-instance cross-entry test
- [x] 19.2 Add daemon restart, request failure, event recovery, and terminal reopen phase
- [x] 19.3 Run real-daemon frame/connection limits and aggregate deterministic stream/request/actor/backpressure/cancellation evidence
- [x] 19.4 Add safe log/evidence field scan
- [x] 19.5 Pass serial cross-entry E2E

## 20. Verification and Tracking

- [x] 20.1 Pass fmt, workspace check, clippy, and focused crate suites
- [x] 20.2 Run full workspace tests and classify any T-102 holder blocker honestly
- [x] 20.3 Pass OpenSpec strict validation, parity lock, and diff check
- [x] 20.4 Complete and fix two code-review rounds
- [x] 20.5 Record test, security, code-review, E2E, and release-readiness evidence
- [x] 20.6 Update only proven parity-lock rows
- [x] 20.7 Close `homie-nep` only when release-readiness evidence permits
