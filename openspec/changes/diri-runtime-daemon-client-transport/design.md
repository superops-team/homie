## Context

Homie currently models `HomieClient` as an in-process facade over `RuntimeSupervisor`. The client crate depends on runtime and storage, owns the control dispatcher, and performs worktree/history/storage work directly. App and CLI can therefore open independent runtime/storage instances for the same data directory.

Diri `7ba3407` proves the required product behavior with a long-lived daemon, UDS request correlation, heartbeat/reconnect, event resume, and terminal attachments. Its wire format uses NDJSON control messages and binary attachment frames. Homie does not need wire compatibility and has selected a single binary multiplexed connection so control, events, and multiple terminals share one lifecycle and one bounded scheduler.

Constraints:

- Rust 1.95 and Edition 2024.
- macOS/Unix local UDS for Wave 1A.
- no environment-variable configuration.
- absolute paths for data directory and daemon executable.
- `RuntimeSupervisor` embeds a rusqlite connection and must remain single-owner.
- no compatibility layer for the current embedded client.
- app first frame cannot wait on runtime/storage/process/network work.
- current holder adoption regression remains owned by T-102.

Stakeholders:

- app and GPUI state projection;
- CLI and MCP runtime tools;
- runtime/session/PTY owner;
- packaging and updater closure;
- observability and evidence gates.

## Goals / Non-Goals

**Goals:**

- One daemon and live runtime owner per absolute data directory.
- Pure Tokio async client with side-effect-free connect.
- One UDS carrying bounded control, event, and terminal streams.
- Exact capability discovery and stable errors.
- Deterministic request, event, reconnect, stream-reset, and shutdown behavior.
- Shared daemon use by app, CLI, and MCP.
- TDD seams that do not add a production fake mode.

**Non-Goals:**

- Agent manifest spawn and full holder/resource parity.
- Remote TCP/node transport.
- Complete MCP/browser/test tool catalog.
- Removal of all app/CLI durable storage reads.
- Styled terminal diff parity.
- Universal signing, notarization, and full updater release.

## Decisions

### Decision 1: Vertical replacement, not dual production paths

The change replaces `HomieClient::open`, `open_with_runtime`, sync facade, and client dispatcher after consumers migrate.

Rationale:

- A dual path would let tests pass while products still execute in-process.
- Runtime ownership cannot be proven while two owners are valid.
- Repository rules explicitly reject compatibility layers unless requested.

Alternatives:

1. Keep embedded client for tests and fallback. Rejected because production/test behavior diverges and app could silently bypass daemon.
2. Add daemon while preserving sync facade over a hidden runtime. Rejected because it retains process-side effects and hides async failure states.

### Decision 2: Explicit launcher separate from client connect

`RuntimeLauncher::ensure_running` receives absolute data-dir and daemon paths. `HomieClient::connect` only connects.

Rationale:

- CLI, app bundle, tests, and future service managers need different launch policies.
- A transport library must not spawn processes as a hidden side effect.
- Launcher races converge through daemon singleton locking.

The launcher probes hello, starts a detached process group only when the endpoint is missing or connection-refused, redirects output to a fixed boot log, and returns without waiting for daemon readiness. Version/protocol/auth errors or hash differences never trigger live daemon replacement. The client reconnect loop waits for ready.

### Decision 3: One UDS with binary multiplexing

One connection carries:

- control on stream 0;
- client-created odd event/terminal streams;
- server-reserved even stream IDs.

Rationale:

- One heartbeat/reconnect identity covers all streams.
- Per-stream reset can isolate slow terminals without extra sockets.
- Input/control are schedulable ahead of output/grid traffic.
- Cross-entry endpoint discovery remains simple.

Alternatives:

1. Diri-style NDJSON control plus separate attachment sockets. Rejected by approved design because it duplicates connection/auth/recovery and cannot prioritize across channels.
2. JSON-only frames with base64 terminal data. Rejected due allocation/size overhead and loss of raw byte semantics.
3. A full third-party RPC protocol. Rejected because current requirements need a small local protocol and no existing dependency matches terminal offset/grid semantics without extra layers.

### Decision 4: Fixed frame header plus JSON/raw payload

Connection preface:

```text
magic[8] = HOMIEIPC
major u16 BE
minor u16 BE
```

Frame:

```text
frame_len u32 BE
version u16 BE
kind u8
flags u8
stream_id u32 BE
message_id u64 BE
sequence u64 BE
payload
```

`frame_len` includes the 24-byte frame header and excludes its own four bytes. Total frame is limited to 16 MiB, JSON control to 4 MiB, and Output payload to 64 KiB.

JSON is used for hello, requests, responses, errors, events, and stream metadata. Raw binary is used for terminal output/input/resize/grid/modes. Wave 1A flags must be zero.

Rationale:

- Fixed-width routing fields can be validated before deserializing payload.
- JSON keeps control evolution inspectable.
- Raw terminal bytes avoid base64 and preserve PTY data.

### Decision 5: Exact frame kinds and invariants

Frame kinds:

- 1 Hello
- 2 HelloAck
- 3 Request
- 4 Response
- 5 Event
- 6 StreamOpen
- 7 StreamOpened
- 8 StreamReset
- 9 StreamClose
- 16 Output
- 17 Input
- 18 Resize
- 19 Grid
- 20 Modes
- 21 ReplayBegin
- 22 ReplayEnd
- 23 Ping
- 24 Pong

Control requests use non-zero message IDs and response correlation. Event and terminal stream sequence values are monotonic. Output includes an absolute log offset.

Malformed lengths, versions, flags, kinds, stream ownership, sequence regression, or payloads close the connection. Per-stream sequence gaps after a valid connection reset only that stream.

### Decision 6: Single-owner RuntimeActor

The production adapter moves `RuntimeSupervisor` into one dedicated OS thread named `homie-runtime-actor`. UDS tasks use non-blocking bounded command submission and receive Tokio oneshot replies.

Rationale:

- rusqlite connection ownership remains explicit.
- socket tasks never perform blocking SQLite, PTY, git, or file work.
- actor ordering provides deterministic mutation serialization.
- Tokio async workers are not occupied by blocking runtime calls.

Actor command capacity is 256. A full queue returns `backpressure`. Event wait, heartbeat, and writer scheduling remain outside the actor.

An internal `RuntimeBackend` service seam supports deterministic server tests. It is constructor-injected by Rust tests only; the daemon binary always constructs the production adapter and exposes no fake/test flag.

Git, history, and bounded output scans do not run on RuntimeActor. They use one dedicated `LongRunningLane` OS worker with a 32-job queue. Actor-owned state crosses this boundary only as owned path/DTO snapshots:

```text
actor prepare
  -> LongRunningLane execute
  -> actor commit
```

The lane never owns `Storage`, `RuntimeSupervisor`, or live registry. Read/output work has a 10s deadline, git list/diff/locate 15s, history scan 30s, and worktree create/remove 60s. A queued job whose deadline or waiter has expired is skipped. Git runs in its own process group and is killed/reaped at hard deadline. Started worktree mutation ignores caller cancellation and completes or reaches the hard deadline, preventing cancellation from deliberately creating a half-finished worktree.

One lane worker intentionally serializes all git jobs. A two-worker repo-key coordinator was considered and rejected for Wave 1A because current requirements do not need concurrent git mutation and the extra scheduler would not improve runtime/session responsiveness.

### Decision 7: Bounded two-level writer scheduling

Each connection has:

- one 256-frame high-priority queue;
- one 256-frame low queue per event/terminal stream;
- at most 64 active client connections per daemon;
- at most 64 active non-control streams;
- at most 1024 pending client requests.

High priority includes hello, request/response, stream lifecycle, input, resize, and ping/pong. The writer sends at most 32 consecutive high frames before attempting one low frame. Low streams use round-robin.

When a low stream fills, the server enqueues high-priority `StreamReset(slow_consumer,last_position)` and removes that low queue. Other streams continue. When a server high-priority queue is full, it closes that connection; when the client local high queue is full, the initiating operation receives `backpressure`.

Rationale:

- Bounded memory and explicit failure are mandatory for terminal output.
- A global low queue would let one terminal starve all others.
- Strict high priority without a quota could starve output indefinitely.

### Decision 8: Event and terminal recovery use authoritative cursors

The runtime event ring retains 1024 entries. Event subscription carries `afterSeq`. If the cursor predates the ring or delivery overflows, the server resets with `event_gap`; the client requests `state.snapshot`, replaces projection, then reopens from the snapshot cursor.

Terminal stream order:

```text
StreamOpened
ReplayBegin
Output*
ReplayEnd
full Grid
Modes
live Output/Grid
```

Reconnect reopens terminal streams from last confirmed output offset and always obtains a new full grid.

Rationale:

- Event sequence alone cannot reconstruct state after a gap.
- Output offsets are durable across daemon restart.
- Full grid prevents application of diffs to an unknown base.

### Decision 9: Adaptive holder-log terminal source

Wave 1A uses the holder output log as the authoritative byte source. The daemon terminal stream hub creates at most one shared `TerminalSource` per attached session. Multiple client streams subscribe to that source instead of polling the actor or reading the full log independently.

The actor validates the session and returns a daemon-internal source descriptor. The source tails at up to 20 Hz while active and backs off to 250 ms idle. Reads are capped at 64 KiB. Output replay/tail and `HeadlessScreen` updates run in the stream hub outside the actor; input and resize remain actor commands.

The current `HeadlessScreen` generates the full visible text grid with default styles for missing metadata. The wire contract already carries `GridUpdate`, allowing T-202 to add styled diffs without replacing transport.

Rationale:

- Current holder/runtime does not expose a live broadcast channel.
- Bounded adaptive tailing is complete and durable, not a production test shortcut.
- A shared per-session source prevents attachment count from multiplying file reads or actor load.
- The wire contract remains the long-term contract.

### Decision 10: Capability truth is generated from handler registries

Hello returns exact request methods and stream openers from the daemon registries. Proto constants do not imply support.

The initial request registry is:

- `state.snapshot`
- `events.wait`
- `daemon.prepare_shutdown`
- `daemon.shutdown`
- `session.spawn`
- `session.list`
- `session.snapshot`
- `session.status`
- `session.artifacts`
- `session.ports`
- `session.set_parent`
- `session.list_children`
- `session.parent`
- `session.history`
- `session.resume_from_history`
- `session.read_diff`
- `session.send_text`
- `session.resize`
- `session.kill`
- `host.locate_repo`
- `worktree.list`
- `worktree.create`
- `worktree.remove`
- `worktree.overview`
- `hook.report`

The stream registry is `events.v1` and `terminal.v1`.

Any item without a working production handler at implementation time is removed from both registry and capability, then recorded as blocked evidence. No unsupported placeholder handler is permitted.

The added session/status/artifact/port/lineage method names transport existing typed client behavior used by app/CLI/MCP. They do not expand product scope. Their DTOs move into `homie-proto` so the client can remove runtime/storage type dependencies.

### Decision 11: Client lifecycle and retry semantics

Connection states:

```text
disconnected -> connecting -> handshaking -> connected
connected -> degraded -> reconnecting -> connected
* -> shutdown
```

Heartbeat idle interval is 25 seconds and timeout is 10 seconds. Reconnect backoff starts at 500 ms and caps at 8 seconds.

Disconnect fails all pending requests. Dropping a request future removes its pending waiter; a late response for that message ID is ignored without closing the connection. The connection manager never replays a request; callers may retry only under explicit method semantics. Event and terminal stream handles retain cursor/offset recovery state.

### Decision 12: Consumer migration

- CLI command functions become async under a Tokio main runtime.
- `control-stdio` bridges bounded stdin/stdout frames to the daemon rather than dispatching locally.
- MCP runtime tools hold the same async client and preserve JSON-RPC error distinctions.
- GPUI owns an `Arc<tokio::runtime::Runtime>` with exactly two worker threads and bridges async updates back through GPUI update/message paths.
- app startup invokes launcher and schedules connect without waiting for daemon readiness or storage work on the GPUI thread.

T-103 still owns unrelated settings/doctor/usage storage reads. Those reads cannot supply live session state.

### Decision 13: Owner-only local security boundary

Runtime directory is mode 0700; socket and lock are 0600. Daemon removes a stale socket only after taking the singleton lock.

The daemon validates that UDS peer UID equals daemon UID:

- macOS/BSD uses `getpeereid`;
- Linux uses `SO_PEERCRED` when that target is enabled.

Logs contain only safe method/stream/sequence/offset/error summaries. Raw payload, terminal bytes, argv/env, credentials, cookies, and tool args/results are prohibited.

### Decision 14: Dependencies

Add workspace Tokio 1 with:

```toml
features = [
  "fs",
  "io-util",
  "macros",
  "net",
  "process",
  "rt-multi-thread",
  "signal",
  "sync",
  "time",
]
```

Use existing `libc` for lock/peer credential platform calls and add it as a workspace Unix dependency where needed. Add mature `sha2 = "0.10"` for the daemon executable SHA-256 returned by hello.

No codec, RPC, async-trait, or channel package is added. The fixed header codec uses Tokio I/O and standard byte conversions; runtime service dispatch uses concrete enums/traits that remain object-safe without async trait methods.

### Decision 15: Two-layer integration testing

Layer 1 runs the real UDS server library with an internal deterministic backend. It tests every protocol, concurrency, stream, gap, and backpressure branch.

Layer 2 spawns the actual daemon binary against an absolute temporary data directory. It tests paths, permissions, singleton, stale socket, hello/snapshot, restart, signals, and cross-entry instance identity.

No environment variable selects a backend. Real process tests receive absolute command arguments.

## Risks / Trade-offs

- [Risk] One connection failure resets all streams.  
  → Mitigation: stream recovery state is explicit; reconnect performs one hello then restores event and terminal streams.

- [Risk] A two-level writer scheduler is more code than independent sockets.  
  → Mitigation: fixed capacities, quotas, and deterministic fake-backend tests keep behavior testable.

- [Risk] A git/history/output job can block runtime mutation if executed on RuntimeActor.  
  → Mitigation: a separate one-worker/32-job LongRunningLane enforces hard deadlines while RuntimeActor remains responsive.

- [Risk] One LongRunningLane serializes unrelated repositories.  
  → Mitigation: Wave 1A prioritizes runtime responsiveness and deterministic mutation safety; queue/deadline metrics prove whether later concurrency is justified.

- [Risk] Adaptive log tailing can add up to 250 ms idle latency.  
  → Mitigation: active streams run at 20 Hz and immediately remain active after input/output; budgets are tested.

- [Risk] Default-style full grid is below final Diri visual parity.  
  → Mitigation: Grid wire format is final; T-202 adds style/diff production without transport migration.

- [Risk] Consumer migration causes a large compile break.  
  → Mitigation: migrate proto/server/client first, then CLI/MCP, then app; delete old API only after all callers compile in the same change.

- [Risk] Existing holder tests may fail real terminal process E2E.  
  → Mitigation: transport tests use deterministic backend, real daemon control tests still run, and T-102 blocker remains explicit. No false pass is recorded.

- [Risk] Stale socket cleanup can race a live daemon.  
  → Mitigation: only singleton lock owner may unlink.

## Migration Plan

1. Add protocol codec and hostile-input RED tests.
2. Add daemon server library, runtime actor, lifecycle, and fake-backend integration tests.
3. Add daemon binary and real subprocess tests.
4. Add pure async client, writer, connection manager, events, terminal, and launcher.
5. Migrate CLI and MCP.
6. Migrate GPUI startup/session bridge.
7. Delete embedded/sync client and runtime/storage client dependencies.
8. Add daemon to package closure and run cross-entry smoke.
9. Record evidence and update parity lock only for proven Wave 1A rows.

Rollback does not preserve a mixed wire/runtime path. Before release, rollback is a git revert of the complete change. Packaged deployment must update app, client, CLI, and daemon atomically as one versioned dependency closure.

## Open Questions

None. Implementation-time discoveries that change frame layout, launcher ownership, queue limits, capability scope, or T-102/T-103 boundaries require PRD/spec revision and review before code changes.
