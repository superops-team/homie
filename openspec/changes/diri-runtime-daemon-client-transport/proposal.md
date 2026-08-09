# Change: Diri Runtime Daemon and Multiplexed Async Client Transport

## Why

Homie currently executes production runtime and storage operations inside `homie-client`. App, CLI, and MCP therefore do not share a real process boundary, live registry, event cursor, or terminal attachment lifecycle. This prevents Diri `7ba3407` runtime/client parity and violates the long-lived runtime ownership contract.

## What Changes

- Add one owner-only `homie-runtime-daemon` per absolute data directory.
- Add an explicit absolute-path `RuntimeLauncher`; client connect remains side-effect free.
- Replace the in-process client with a pure Tokio async UDS client.
- Use one binary length-prefixed multiplexed connection for control, events, and terminal streams.
- Add fixed queue/frame/request/stream limits, heartbeat, reconnect, event snapshot recovery, and per-stream reset.
- Move currently executable runtime/worktree/history dispatch from client to daemon.
- Migrate app, CLI, and MCP runtime operations to the shared daemon.
- Delete production embedded/synchronous client paths and client runtime/storage dependencies.
- Package the daemon binary as part of the app dependency closure.

## Capabilities

### New Capabilities

- `runtime-daemon-lifecycle`: singleton daemon paths, actor ownership, launcher, startup, drain, shutdown, and recovery.
- `multiplexed-runtime-transport`: preface/frame codec, hello, control requests, event streams, terminal streams, bounded scheduling, and stable errors.
- `async-runtime-client`: Tokio client lifecycle, request correlation, heartbeat, reconnect, event gap recovery, and terminal reopen.
- `shared-runtime-consumers`: CLI/MCP/GPUI migration to one daemon and removal of embedded runtime shortcuts.

### Modified Capabilities

- None. This child change introduces the executable Wave 1A capabilities derived from the Wave 0 parity baseline.

## Impact

- Code:
  - `crates/homie-proto/`
  - `crates/homie-runtime/`
  - `crates/homie-client/`
  - `crates/homie-cli/`
  - `crates/homie-app/`
  - package/build scripts
- Long-lived specs:
  - `specs/runtime-client-transport/README.md`
  - `specs/runtime-supervisor/README.md`
  - `specs/desktop-shell/README.md`
  - `specs/mcp-automation/README.md`
  - `specs/observability/README.md`
  - `specs/packaging-updater/README.md`
- Tracking:
  - Bead `homie-nep`
  - parent change `diri-7ba3407-parity-rebaseline`
  - master task T-101
- Explicitly deferred:
  - T-102 agent/holder parity and current detached regression
  - T-103 complete UI/CLI storage ownership migration
  - remote/node transport
  - full packaging/signing release gate
