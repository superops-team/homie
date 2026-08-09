# Diri Runtime Daemon and Multiplexed Async Client Transport Implementation Plan

> **Execution rule:** Use `test-driven-development` for every behavior change. Complete RED evidence before implementation, then GREEN, then refactor. Do not write production code from this plan before the OpenSpec/spec-review gate is approved.

**Goal:** Replace Homie's in-process runtime client with one owner-only daemon per data directory and one bounded binary-multiplexed async client connection shared by app, CLI, and MCP.

**Architecture:** `homie-runtime-daemon` owns `RuntimeSupervisor` behind a single actor. `homie-client` uses Tokio UDS for control, events, and terminal streams. `RuntimeLauncher` is explicit and separate. GPUI uses a two-worker Tokio bridge. Current product behavior moves behind exact daemon handlers; old embedded paths are deleted.

**Tech stack:** Rust 1.95, Edition 2024, Tokio 1, Serde/Serde JSON, SHA-256, Unix UDS/libc, rusqlite, GPUI, existing alacritty terminal model.

**Absolute repository root:** `/Users/bytedance/workspace/github/homie`. Every command in this plan runs with this exact working directory. Runtime/process tests pass freshly created absolute temporary data-directory paths; they do not use environment configuration.

**Source documents:**

- PRD: `prd-spec/features/diri-runtime-daemon-client-transport/2026-08-08-diri-runtime-daemon-client-transport-design.md`
- Design: `openspec/changes/diri-runtime-daemon-client-transport/design.md`
- Capability specs: `openspec/changes/diri-runtime-daemon-client-transport/specs/`
- Bead: `homie-nep`

## Guardrails

1. Do not add an in-process production fallback.
2. Do not use environment variables for data dir, endpoint, daemon path, queue limits, or backend selection.
3. Do not introduce a daemon `--test-mode` or fake-backend flag.
4. Do not advertise a method/stream without a production handler/opener.
5. Do not log raw payload, terminal bytes, argv/env, credentials, cookies, or tool args/results.
6. Do not change holder liveness from `detached` to `running` without T-102 evidence.
7. Do not let GPUI first frame block on daemon, storage, PTY, process, or network work.
8. Use `apply_patch` for manual edits and preserve unrelated dirty-worktree changes.

## Task 1: Freeze Baseline and Dependency Delta

**Files:**

- Modify: `Cargo.toml`
- Modify: `docs/research/rust-package-selection.md`
- Test: `Cargo.lock`

**Step 1.1 - Record RED dependency evidence**

Run:

```bash
cargo tree -p homie-client -e normal
```

Expected RED: output contains `homie-runtime` and `homie-storage`; workspace has no Tokio dependency.

**Step 1.2 - Add approved workspace dependencies**

Add:

```toml
sha2 = "0.10"
tokio = { version = "1", features = [
    "fs",
    "io-util",
    "macros",
    "net",
    "process",
    "rt-multi-thread",
    "signal",
    "sync",
    "time",
] }
```

Use existing `libc = "0.2"` through a workspace Unix dependency rather than adding another lock/peer package.

**Step 1.3 - Resolve lockfile and verify**

Run:

```bash
cargo check -p homie-proto -p homie-runtime -p homie-client
```

Expected GREEN: dependencies resolve without changing product behavior.

## Task 2: Move Transport DTO Ownership to homie-proto

**Files:**

- Create: `crates/homie-proto/src/transport.rs`
- Create: `crates/homie-proto/src/stream.rs`
- Modify: `crates/homie-proto/src/lib.rs`
- Modify: `crates/homie-proto/src/model.rs`
- Modify: `crates/homie-proto/Cargo.toml`
- Create: `crates/homie-proto/tests/runtime_transport_contract.rs`

**Step 2.1 - Write RED DTO tests**

Cover:

- Hello request/ack camelCase JSON;
- exact stable error codes;
- stream open/reset metadata;
- session/status/artifact/port/lineage request/result DTOs;
- method constants for existing client behavior;
- runtime event and session summary DTOs no longer require runtime/storage imports.

Run:

```bash
cargo test -p homie-proto --test runtime_transport_contract
```

Expected RED: missing modules/types/constants.

**Step 2.2 - Define protocol-owned types**

Add constants/types including:

```rust
pub const WIRE_MAJOR: u16 = 1;
pub const WIRE_MINOR: u16 = 0;

pub enum StreamKind {
    EventsV1,
    TerminalV1,
}

pub struct RuntimeSnapshot {
    pub cursor: u64,
    pub sessions: Vec<SessionSummary>,
}
```

Move or mirror wire DTOs currently owned by `homie-runtime`/`homie-storage` into proto. Runtime implementation may re-export aliases during the same change, but client public signatures MUST use proto types.

Add current behavior method constants:

```text
session.snapshot
session.status
session.artifacts
session.ports
session.set_parent
session.list_children
session.parent
worktree.overview
hook.report
```

**Step 2.3 - Verify DTO contract**

Run:

```bash
cargo test -p homie-proto --test runtime_transport_contract
cargo test -p homie-proto --test protocol_contract
```

Expected GREEN: exact serialized fixtures and legacy protocol tests pass.

## Task 3: Implement the Fixed Frame Codec

**Files:**

- Modify: `crates/homie-proto/src/transport.rs`
- Create: `crates/homie-proto/tests/transport_codec.rs`

**Step 3.1 - Write RED frame round-trip tests**

Test all fixed fields and exact bytes:

```rust
pub struct FrameHeader {
    pub version: u16,
    pub kind: FrameKind,
    pub flags: u8,
    pub stream_id: u32,
    pub message_id: u64,
    pub sequence: u64,
}
```

Run:

```bash
cargo test -p homie-proto --test transport_codec frame_round_trip
```

Expected RED: codec absent.

**Step 3.2 - Implement preface and frame encoding**

Required constants:

```rust
pub const PREFACE_LEN: usize = 12;
pub const FRAME_HEADER_LEN: usize = 24;
pub const MAX_FRAME_LEN: usize = 16 * 1024 * 1024;
pub const MAX_CONTROL_PAYLOAD: usize = 4 * 1024 * 1024;
pub const MAX_OUTPUT_PAYLOAD: usize = 64 * 1024;
```

Use big-endian `to_be_bytes`/`from_be_bytes`; do not add a codec package.

**Step 3.3 - Write hostile-input RED tests**

Cover:

- partial preface/header/payload;
- coalesced frames;
- length `<24`;
- length `>16 MiB`;
- unknown version/kind/flags;
- JSON `>4 MiB`;
- Output `>64 KiB`;
- invalid stream ownership.

Run:

```bash
cargo test -p homie-proto --test transport_codec
```

Expected RED before validation implementation, then GREEN after minimal bounds checks.

## Task 4: Add Runtime Paths and Explicit Launcher

**Files:**

- Create: `crates/homie-proto/src/paths.rs`
- Create: `crates/homie-client/src/launcher.rs`
- Modify: `crates/homie-client/src/lib.rs`
- Modify: `crates/homie-client/Cargo.toml`
- Create: `crates/homie-client/tests/launcher.rs`

**Step 4.1 - Write RED path tests**

Test exact absolute paths and rejection of relative data-dir/executable input.

Run:

```bash
cargo test -p homie-client --test launcher runtime_paths
```

Expected RED: no RuntimePaths/launcher.

**Step 4.2 - Implement RuntimePaths**

Expose:

```rust
pub struct RuntimePaths {
    pub runtime_dir: PathBuf,
    pub socket: PathBuf,
    pub lock: PathBuf,
    pub boot_log: PathBuf,
    pub daemon_log: PathBuf,
}
```

Do not read HOME or other environment variables inside this type.

**Step 4.3 - Write RED launcher probe/spawn tests**

Use an absolute fixture executable and fake live probe. Assert:

- live endpoint is not spawned;
- unavailable endpoint starts once;
- version/protocol/auth/hash difference returns without spawn or replacement;
- launcher returns before readiness;
- relative/missing/non-executable path fails;
- argv contains explicit `--data-dir <absolute>`.

**Step 4.4 - Implement launcher**

`RuntimeLauncher::ensure_running` MUST remain a separate public type. `HomieClient::connect` MUST not call it.

**Step 4.5 - Verify**

Run:

```bash
cargo test -p homie-client --test launcher
```

Expected GREEN.

## Task 5: Create Runtime Backend Commands and Single-Owner Actor

**Files:**

- Create: `crates/homie-runtime/src/runtime_actor.rs`
- Create: `crates/homie-runtime/src/long_running.rs`
- Create: `crates/homie-runtime/src/dispatcher.rs`
- Modify: `crates/homie-runtime/src/lib.rs`
- Modify: `crates/homie-runtime/src/history.rs`
- Modify: `crates/homie-runtime/Cargo.toml`

**Step 5.1 - Write RED actor unit tests**

Inside `runtime_actor.rs`, add tests for:

- ordered concurrent mutations;
- correlated oneshot replies;
- queue capacity 256;
- actor shutdown;
- no command accepted after prepare-shutdown.
- a long-running job does not block actor session/list/input/resize commands.

Run:

```bash
cargo test -p homie-runtime runtime_actor::tests
```

Expected RED: actor absent.

**Step 5.2 - Write RED LongRunningLane tests**

Inside `long_running.rs`, add tests for:

- one worker and queue capacity 32;
- the 33rd job returns backpressure;
- queued expired/cancelled job never executes;
- read-only hard deadline terminates and reaps child process group;
- started worktree mutation ignores caller cancellation but stops at 60s deadline;
- failed/timed-out history scan performs no storage commit;
- all git jobs execute serially.

Run:

```bash
cargo test -p homie-runtime long_running::tests
```

Expected RED: lane absent.

**Step 5.3 - Define concrete actor messages**

Start a dedicated `std::thread::Builder` thread named `homie-runtime-actor`. Use a capacity-256 synchronous command channel with non-blocking submission from Tokio tasks and Tokio oneshot replies. Use a concrete enum:

```rust
enum ActorCommand {
    Call {
        request: RuntimeCall,
        reply: oneshot::Sender<Result<Value, ServiceError>>,
    },
    OpenTerminalSource {
        session_id: SessionId,
        reply: oneshot::Sender<Result<TerminalSourceDescriptor, ServiceError>>,
    },
    PrepareShutdown { reply: oneshot::Sender<Result<(), ServiceError>> },
    Shutdown { reply: oneshot::Sender<()> },
}
```

The backend trait remains synchronous and `Send`; async waits stay outside it. RuntimeActor owns SQLite, PTY, live registry, and runtime mutation. Daemon shutdown joins the OS thread without blocking a Tokio worker.

**Step 5.4 - Implement the bounded LongRunningLane**

Use one dedicated `homie-runtime-long-running` OS worker and a 32-job bounded queue. The lane receives only owned path/DTO snapshots and cannot access Storage/RuntimeSupervisor/live registry.

Apply fixed server deadlines:

```text
output/artifact/status/snapshot  10s
git list/diff/locate             15s
history scan                     30s
worktree create/remove           60s
```

**Step 5.5 - Split handlers into actor prepare, lane execute, actor commit**

Move current client runtime/storage/worktree/history behavior behind daemon dispatcher handlers. Runtime/storage mutations remain actor-owned. Git/history/output scanning runs in the lane; actor commit receives owned results and never commits partial timed-out work.

Implement a bounded cancellable git runner with null stdin, disabled terminal prompt, capped stdout/stderr, independent process group, hard deadline, and child reaping. Add deadline/cancel checks and an overall file-count bound to history scanning.

Do not add a repo-key coordinator or second worker: one worker serializes all git jobs.

**Step 5.6 - Verify**

Run:

```bash
cargo test -p homie-runtime runtime_actor::tests
cargo test -p homie-runtime long_running::tests
cargo test -p homie-runtime --test worktree_git
cargo test -p homie-runtime --test history_scanner
```

Expected GREEN except already documented holder-specific tests, which are not run in this task.

## Task 6: Build Exact Handler and Stream Registries

**Files:**

- Modify: `crates/homie-runtime/src/dispatcher.rs`
- Modify: `crates/homie-proto/src/lib.rs`
- Create: `crates/homie-runtime/src/capabilities.rs`

**Step 6.1 - Write RED registry equality tests**

Assert:

- every advertised method has a handler;
- every handler appears in advertised methods;
- every handler is explicitly classified as Actor, LongRunning, or AsyncWait;
- exact stream openers are `events.v1` and `terminal.v1`;
- future `Method::ALL` constants are not automatically advertised.

Run:

```bash
cargo test -p homie-runtime capabilities::tests
```

Expected RED.

**Step 6.2 - Implement static registry**

Create one registry used by both Hello and dispatch. Include only the PRD capability list.

**Step 6.3 - Add current consumer behavior handlers**

Add handlers and proto DTOs for:

- artifacts/ports/status;
- session snapshot and lineage;
- hook report;
- worktree overview.

Do not add browser/test/LLM/task/memory/remote placeholders.

**Step 6.4 - Verify**

Run:

```bash
cargo test -p homie-runtime capabilities::tests
```

Expected GREEN with exact set equality.

## Task 7: Implement UDS Server Handshake and Control Requests

**Files:**

- Create: `crates/homie-runtime/src/server.rs`
- Create: `crates/homie-runtime/src/connection.rs`
- Modify: `crates/homie-runtime/src/lib.rs`

**Step 7.1 - Write RED server unit tests with fake backend**

Test over a real temporary Unix socket:

- peer accepted;
- Hello required first;
- version mismatch;
- exact capabilities;
- concurrent and out-of-order response correlation;
- unknown method;
- malformed/oversized frame;
- 65th active connection rejected before payload;
- actor backpressure.

Run:

```bash
cargo test -p homie-runtime server::tests
```

Expected RED.

**Step 7.2 - Implement peer UID validation**

Use `getpeereid` on macOS/BSD and `SO_PEERCRED` on Linux. Reject before protocol parsing.

**Step 7.3 - Implement connection reader/demux**

Reader validates header before payload allocation and routes stream 0 Request/Ping only after Hello.

**Step 7.4 - Implement correlated response writer**

Use the shared high-priority writer channel; do not write concurrently from handlers.

**Step 7.5 - Verify**

Run:

```bash
cargo test -p homie-runtime server::tests
```

Expected GREEN.

## Task 8: Implement Bounded Writer Scheduling

**Files:**

- Create: `crates/homie-runtime/src/writer.rs`
- Modify: `crates/homie-runtime/src/connection.rs`
- Add tests: `crates/homie-runtime/src/writer.rs`

**Step 8.1 - Write RED scheduler tests**

Test:

- capacities 256;
- max 32 consecutive high frames;
- round-robin low streams;
- removal of reset stream;
- server high-queue overflow closes the connection;
- one slow stream does not block another.

Run:

```bash
cargo test -p homie-runtime writer::tests
```

Expected RED.

**Step 8.2 - Implement scheduler state**

Use bounded Tokio mpsc for high ingress and explicit per-stream `VecDeque<Frame>` capped at 256 inside one writer task. Do not create one unbounded task/channel per frame.

**Step 8.3 - Implement slow-consumer reset**

Queue `StreamReset(slow_consumer,last_position)` as high priority, then drop only the affected low queue.

If the high-priority queue itself is full, close the connection instead of dropping control or allocating another queue.

**Step 8.4 - Verify**

Run:

```bash
cargo test -p homie-runtime writer::tests
```

Expected GREEN.

## Task 9: Implement Event Stream and Snapshot Recovery Contract

**Files:**

- Create: `crates/homie-runtime/src/streams.rs`
- Modify: `crates/homie-runtime/src/server.rs`
- Modify: `crates/homie-runtime/src/dispatcher.rs`

**Step 9.1 - Write RED event tests**

Test:

- replay from valid `afterSeq`;
- 1024-entry retention;
- too-old cursor reset;
- delivery overflow reset;
- `state.snapshot` cursor consistency;
- filter behavior.

Run:

```bash
cargo test -p homie-runtime streams::tests::event
```

Expected RED.

**Step 9.2 - Implement event stream opener**

The opener reads actor event snapshots but owns async delivery outside actor.

**Step 9.3 - Implement consistent state snapshot**

Snapshot session state and current event cursor as one actor command/result so events after the cursor can be resumed without ambiguity.

**Step 9.4 - Verify**

Run:

```bash
cargo test -p homie-runtime streams::tests::event
```

Expected GREEN.

## Task 10: Implement Terminal Replay, Grid, and Live Stream

**Files:**

- Modify: `crates/homie-runtime/src/streams.rs`
- Modify: `crates/homie-runtime/src/runtime_actor.rs`
- Create: `crates/homie-runtime/src/terminal_source.rs`
- Modify: `crates/homie-runtime/src/screen.rs`

**Step 10.1 - Write RED terminal stream tests**

Fake backend cases:

- exact open/replay/full-grid/modes/live ordering;
- absolute offset progression;
- Input/Resize routing;
- sequence gap/reset;
- two streams, one slow;
- multiple clients on one session share one source/log reader;
- close does not terminate session.

Run:

```bash
cargo test -p homie-runtime streams::tests::terminal
```

Expected RED.

**Step 10.2 - Implement shared per-session TerminalSource hub**

Ask the actor once for a validated daemon-internal source descriptor. Create at most one source/tailer per attached session and let multiple client streams subscribe. The source reads/replays at most 64 KiB per chunk and emits absolute offsets.

**Step 10.3 - Implement adaptive live tail**

Use 50 ms active delay and 250 ms idle delay. Keep one tail task per attached session, cancel it when its final subscriber closes, and route Input/Resize back through actor commands.

**Step 10.4 - Implement full grid conversion**

Add a conversion from existing `HeadlessScreen` visible lines to full `GridUpdate` with default styles. Do not add styled diff logic.

**Step 10.5 - Verify**

Run:

```bash
cargo test -p homie-runtime streams::tests::terminal
```

Expected GREEN.

## Task 11: Implement Daemon Lifecycle and Binary

**Files:**

- Create: `crates/homie-runtime/src/daemon.rs`
- Create: `crates/homie-runtime/src/bin/homie-runtime-daemon.rs`
- Modify: `crates/homie-runtime/Cargo.toml`
- Create: `crates/homie-runtime/tests/daemon_process.rs`

**Step 11.1 - Write RED process tests**

Build the daemon binary, derive its absolute path from the Cargo target directory, and test:

- 0700/0600 permissions;
- singleton race;
- stale socket cleanup;
- Hello/snapshot;
- SIGTERM drain;
- restart produces new instance ID;
- no session/database deletion on failed start.

Run:

```bash
cargo test -p homie-runtime --test daemon_process -- --test-threads=1
```

Expected RED.

**Step 11.2 - Implement lock and startup order**

Use existing Unix `libc` for non-blocking lock and `O_NOFOLLOW` lock-file open. Only lock owner can unlink.

**Step 11.3 - Implement executable SHA-256**

Hash the daemon's canonical executable incrementally with a bounded buffer and `sha2`; return safe failure if unreadable instead of fabricating a digest.

**Step 11.4 - Implement prepare/shutdown and signals**

Send shutdown ACK before listener/actor teardown. Preserve holder-owned processes.

**Step 11.5 - Verify**

Run:

```bash
cargo test -p homie-runtime --test daemon_process -- --test-threads=1
```

Expected GREEN.

## Task 12: Implement Async Client Core

**Files:**

- Create: `crates/homie-client/src/client.rs`
- Create: `crates/homie-client/src/connection.rs`
- Create: `crates/homie-client/src/writer.rs`
- Modify: `crates/homie-client/src/lib.rs`
- Create: `crates/homie-client/tests/support/mod.rs`
- Create: `crates/homie-client/tests/request_correlation.rs`

**Step 12.1 - Write RED mock-server tests**

Test:

- Hello lifecycle;
- out-of-order responses;
- timeout removes one pending;
- dropped request future removes its pending waiter and ignores a late response;
- disconnect fails all once;
- pending limit 1024;
- high queue limit/fairness;
- close enters terminal shutdown.

Run:

```bash
cargo test -p homie-client --test request_correlation
```

Expected RED.

**Step 12.2 - Implement ClientOptions and state watch**

Use:

```rust
pub struct ClientOptions {
    pub endpoint: RuntimeEndpoint,
    pub role: ClientRole,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
}
```

Expose a `watch::Receiver<ConnectionState>`.

**Step 12.3 - Implement pending request map**

Allocate non-zero message IDs, cap at 1024, and resolve by response ID. Use a cancellation guard so dropped request futures remove their waiter. Disconnect drains the map exactly once; late responses for removed IDs are ignored.

**Step 12.4 - Implement client writer**

Mirror server limits and 32-high fairness. Input/resize remain high priority.

**Step 12.5 - Verify**

Run:

```bash
cargo test -p homie-client --test request_correlation
```

Expected GREEN.

## Task 13: Implement Client Heartbeat, Reconnect, Events, and Terminal Handles

**Files:**

- Modify: `crates/homie-client/src/connection.rs`
- Create: `crates/homie-client/src/events.rs`
- Create: `crates/homie-client/src/terminal.rs`
- Create: `crates/homie-client/tests/recovery.rs`
- Create: `crates/homie-client/tests/streams.rs`

**Step 13.1 - Write RED lifecycle/reconnect tests**

Use paused Tokio time where possible. Assert 25s idle ping, 10s timeout, backoff 500ms to 8s, reset after Hello, and no reconnect after close.

**Step 13.2 - Implement heartbeat/reconnect**

Do not replay pending requests. Recover stream descriptors only after new Hello.

**Step 13.3 - Write RED event recovery tests**

Cover replayable cursor and `event_gap -> state.snapshot -> reopen`.

**Step 13.4 - Implement EventStream**

Decoded queue capacity is 256. Snapshot replacement is explicit to consumer.

**Step 13.5 - Write RED terminal recovery tests**

Cover last confirmed offset, daemon instance change, full-grid barrier, and local slow-consumer resync.

**Step 13.6 - Implement TerminalStream**

Discard grid diffs until a full Grid arrives after every reopen.

**Step 13.7 - Verify**

Run:

```bash
cargo test -p homie-client --test recovery
cargo test -p homie-client --test streams
```

Expected GREEN.

## Task 14: Build Typed Async Facade and Remove Client Runtime Logic

**Files:**

- Modify: `crates/homie-client/src/client.rs`
- Modify: `crates/homie-client/src/lib.rs`
- Modify: `crates/homie-client/Cargo.toml`
- Replace: `crates/homie-client/tests/runtime_client.rs`

**Step 14.1 - Write RED typed-facade tests**

For every current app/CLI/MCP client method, configure mock server response and assert exact method/DTO. Cover artifacts, ports, status, lineage, hook report, history, diff, worktree, and locate-repo.

**Step 14.2 - Implement typed async methods**

Typed methods call generic request or stream APIs only. No git, file, storage, runtime, history scan, or output parsing remains in client.

**Step 14.3 - Delete old production paths**

Delete:

- `HomieClient { runtime }`;
- `open(data_dir)`;
- `open_with_runtime`;
- `handle_request`;
- `handle_control_message`;
- sync `serve_control_stream`;
- normal dependencies on `homie-runtime`, `homie-storage`, and no-longer-used `homie-remote`.

**Step 14.4 - Verify dependency boundary**

Run:

```bash
cargo test -p homie-client
cargo tree -p homie-client -e normal | rg 'homie-(runtime|storage)' && exit 1 || true
rg -n 'RuntimeSupervisor|open_with_runtime|serve_control_stream' crates/homie-client/src && exit 1 || true
```

Expected GREEN: tests pass and negative scans return no match.

## Task 15: Migrate CLI and control-stdio

**Files:**

- Modify: `crates/homie-cli/src/main.rs`
- Modify: `crates/homie-cli/Cargo.toml`
- Modify: `crates/homie-cli/tests/control_stdio_cli.rs`
- Modify: all runtime-backed CLI tests under `crates/homie-cli/tests/`

**Step 15.1 - Write RED shared-daemon CLI tests**

Start a real daemon with absolute temp data dir. Assert CLI session list/snapshot/history/diff/worktree/events and hook/notify use that instance.

**Step 15.2 - Convert runtime-backed commands to async**

Use Tokio main and await typed client methods. Keep doctor/usage direct storage until T-103, but prevent those paths from supplying live runtime state.

**Step 15.3 - Replace control-stdio dispatcher**

Read stdin control JSON with a 4 MiB per-message limit, reject oversized input before deserialization, forward to daemon, and write responses. Preserve message correlation and method-not-found.

**Step 15.4 - Verify**

Run:

```bash
cargo build -p homie-runtime --bin homie-runtime-daemon --bin homie-runtime-holder
cargo test -p homie-cli --test control_stdio_cli
cargo test -p homie-cli --test events_cli
cargo test -p homie-cli --test session_snapshot_cli
cargo test -p homie-cli --test worktree_cli
cargo test -p homie-cli --test hook_report_runtime_cli
```

Expected GREEN.

## Task 16: Migrate MCP Runtime Context

**Files:**

- Modify: `crates/homie-cli/src/main.rs`
- Modify: `crates/homie-cli/tests/mcp_stdio_runtime_cli.rs`
- Modify: `crates/homie-cli/tests/mcp_orchestration_transcript_cli.rs`
- Modify: related MCP lineage/artifact/worktree tests

**Step 16.1 - Write RED capability-filter tests**

Assert MCP omits tools whose required daemon method/stream is absent.

**Step 16.2 - Make McpRuntimeContext async-client backed**

Remove `HomieClient::open` and sync references. Await status, lineage, artifact, and worktree methods.

**Step 16.3 - Preserve JSON-RPC errors**

Map daemon `method_not_found` to `-32601`; retain distinct execution/transport errors.

**Step 16.4 - Verify**

Run:

```bash
cargo build -p homie-runtime --bin homie-runtime-daemon --bin homie-runtime-holder
cargo test -p homie-cli --test mcp_stdio_runtime_cli
cargo test -p homie-cli --test mcp_orchestration_transcript_cli
cargo test -p homie-cli --test mcp_lineage_children_cli
cargo test -p homie-cli --test mcp_get_artifacts_cli
cargo test -p homie-cli --test mcp_worktree_tools_cli
```

Expected GREEN, including the previous `-32601` regression.

## Task 17: Migrate GPUI to a Two-Worker Runtime Bridge

**Files:**

- Create: `crates/homie-app/src/lib.rs`
- Create: `crates/homie-app/src/runtime_bridge.rs`
- Create: `crates/homie-app/src/daemon_launch.rs`
- Modify: `crates/homie-app/src/main.rs`
- Modify: `crates/homie-app/Cargo.toml`
- Create: `crates/homie-app/tests/runtime_bridge.rs`
- Replace brittle assertions in: `crates/homie-app/tests/app_shell_copy_regression.rs`

**Step 17.1 - Write RED bridge tests**

Test bridge state transitions and projection updates with a mock client/server. Expose the non-GPUI bridge through the app library for cross-entry tests. Add a first-frame guard that does not depend on rustfmt source substrings.

**Step 17.2 - Create two-worker Tokio owner**

Use:

```rust
tokio::runtime::Builder::new_multi_thread()
    .worker_threads(2)
    .thread_name("homie-async")
    .enable_all()
    .build()
```

**Step 17.3 - Move startup off GPUI frame path**

Resolve absolute bundle/sibling daemon path, call launcher, and schedule connect. Initial UI state is connecting/unavailable without blocking.

**Step 17.4 - Migrate live session data**

Replace polling/sync calls for list/spawn/send/resize/artifacts/worktree/terminal with bridge commands and snapshot/event/stream messages.

Settings storage paths remain only where assigned to T-103.

**Step 17.5 - Verify**

Run:

```bash
cargo test -p homie-app --test runtime_bridge
cargo test -p homie-app --test palette_model
cargo test -p homie-app --test app_shell_copy_regression
cargo check -p homie-app
```

Expected GREEN and no source-format-sensitive runtime assertions.

## Task 18: Add Daemon to Package Closure

**Files:**

- Modify: `scripts/package/package.sh`
- Modify: `scripts/package/tests/verify-app-binary.sh`
- Modify: `Makefile`
- Create or modify: package smoke test under `scripts/package/tests/`

**Step 18.1 - Write RED package verification**

Assert bundle contains executable:

```text
Homie.app/Contents/Resources/bin/homie-runtime-daemon
```

and launcher resolves that fixed location to an absolute path.

**Step 18.2 - Build/copy/sign daemon**

Add daemon to the existing release build, copy, chmod, and nested-code signing sequence.

**Step 18.3 - Add packaged hello/snapshot smoke**

Use one absolute temp data dir and verify the bundled daemon and bundled CLI/client complete Hello/snapshot. Do not claim this shell smoke launched the real GUI, and do not require T-501 notarization credentials. The production app bridge same-instance check remains Task 19.

**Step 18.4 - Verify**

Run:

```bash
bash scripts/package/tests/verify-app-binary.sh
make smoke
```

Expected GREEN for local package closure; full release trust remains T-501.

## Task 19: Cross-Entry Recovery and Security E2E

**Files:**

- Create: `crates/homie-cli/tests/shared_daemon_e2e.rs`
- Modify: `crates/homie-cli/Cargo.toml` dev-dependencies
- Use: `homie-app` library runtime bridge

**Step 19.1 - Write RED cross-entry test**

Start the real daemon, instantiate the production app runtime bridge library, and invoke CLI/MCP subprocesses against the same absolute data dir. Record the same daemon instance ID and cursor. The test derives already-built binaries from absolute target paths; it does not add environment configuration or production probe flags.

**Step 19.2 - Add restart phase**

Terminate daemon, restart it, assert:

- pending requests fail once;
- client reconnects to new instance;
- event cursor replays or snapshots;
- terminal handle reopens from confirmed offset in deterministic backend tests.

**Step 19.3 - Add security/limit evidence phase**

The real daemon black-box test exercises malformed/oversized frames, wrong flags/kind, the 65th connection, process cleanup, and safe introspection. Deterministic runtime/client suites remain the owning proof for the 65th stream, 1025th pending request, actor overflow, server high-queue overflow, slow stream, and cancellation cleanup; Task 19 aggregates those command results instead of trying to nondeterministically saturate a production process.

**Step 19.4 - Verify**

Run:

```bash
cargo build -p homie-runtime --bin homie-runtime-daemon -p homie-cli --bin homie
cargo test -p homie-cli --test shared_daemon_e2e -- --test-threads=1
```

Expected GREEN.

## Task 20: Full Gates, Evidence, and Tracking

**Files:**

- Create: `docs/verification/diri-runtime-daemon-client-transport/test-report.md`
- Create: `docs/verification/diri-runtime-daemon-client-transport/security-review-report.md`
- Create: `docs/verification/diri-runtime-daemon-client-transport/code-review-report.md`
- Create: `docs/verification/diri-runtime-daemon-client-transport/e2e-report.md`
- Create: `docs/verification/diri-runtime-daemon-client-transport/release-readiness-report.md`
- Modify only proven rows: `docs/research/diri-parity-lock.md`
- Update: Bead `homie-nep`

**Step 20.1 - Run formatting and focused gates**

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p homie-proto
cargo test -p homie-runtime
cargo test -p homie-client
cargo test -p homie-cli
cargo test -p homie-app
```

Record command, exit code, duration, and failure ownership.

**Step 20.2 - Run full workspace**

```bash
cargo test --workspace
```

If holder T-102 failures remain, record `blocked`/`partial` for full runtime parity, not Wave 1A transport failure, and include exact failing test names.

**Step 20.3 - Run project gates**

```bash
openspec validate diri-runtime-daemon-client-transport --strict
make parity-lock
git diff --check
```

**Step 20.4 - Perform two review rounds**

Round 1: syntax, compile, interface, handler/capability, errors.  
Round 2: races, cancellation, queue/memory bounds, security fields, process cleanup, first-frame behavior.

Fix findings through RED/GREEN tasks and rerun affected gates.

**Step 20.5 - Update parity evidence honestly**

API-002 may become implemented only when shared-daemon/reconnect/event/terminal E2E passes. RT-001/RT-006/RT-007 remain partial while T-102 holder tests fail.

**Step 20.6 - Close Bead only after evidence**

```bash
bd close homie-nep --reason "Implemented and verified. See docs/verification/diri-runtime-daemon-client-transport/release-readiness-report.md."
```

Do not close if release readiness is blocked.

## Execution Checkpoints

1. Tasks 1-3: protocol checkpoint.
2. Tasks 4-11: daemon/server checkpoint.
3. Tasks 12-14: client replacement checkpoint.
4. Tasks 15-17: consumer migration checkpoint.
5. Tasks 18-20: package/E2E/evidence checkpoint.

At each checkpoint:

- run affected crate tests;
- inspect `git diff --check`;
- confirm no unrelated files changed;
- update `tasks.md` incrementally;
- preserve Bead `IN_PROGRESS` until final evidence.

## Effort Estimate

Estimates are focused engineering days for one owner, excluding T-102 holder fixes and review wait time.

| Task | Estimate | Primary uncertainty |
|------|----------|---------------------|
| 1. Dependencies | 0.25 day | lockfile churn |
| 2. Proto DTO ownership | 1.0 day | number of runtime/storage type moves |
| 3. Frame codec | 0.75 day | hostile partial-read coverage |
| 4. Paths/launcher | 0.75 day | detached process behavior on macOS |
| 5. RuntimeActor/LongRunningLane | 2.0 days | actor/lane split and process deadline cleanup |
| 6. Capability registry | 1.0 day | preserving all current typed behavior |
| 7. UDS control server | 1.25 days | cancellation and peer credential portability |
| 8. Writer scheduler | 0.75 day | deterministic fairness tests |
| 9. Event stream | 1.0 day | snapshot cursor consistency |
| 10. Terminal source/stream | 1.5 days | shared tailer and grid reconstruction |
| 11. Daemon lifecycle | 1.25 days | singleton/signal process tests |
| 12. Client core | 1.25 days | concurrent pending cleanup |
| 13. Client recovery/streams | 1.5 days | deterministic time/reconnect tests |
| 14. Typed client replacement | 1.0 day | caller-visible DTO migration |
| 15. CLI migration | 1.0 day | breadth of runtime CLI fixtures |
| 16. MCP migration | 1.0 day | async context and error mapping |
| 17. GPUI bridge | 1.5 days | GPUI thread/update integration |
| 18. Package closure | 0.5 day | local bundle scripts |
| 19. Cross-entry/security E2E | 1.0 day | subprocess orchestration |
| 20. Review/evidence | 1.0 day | workspace regressions |

Total planned effort: approximately 21.25 engineering days. A T-102 failure discovered while running real holder tests is recorded as a blocker and is not absorbed into this estimate.
