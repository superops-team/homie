## ADDED Requirements

### Requirement: CLI runtime commands use the shared daemon
CLI session, worktree, history, diff, event, and control operations SHALL use async `HomieClient` after an explicit launcher call. CLI MUST NOT create an embedded RuntimeSupervisor.

#### Scenario: CLI starts without a daemon
- **WHEN** a runtime-backed CLI command targets an unavailable data directory
- **THEN** CLI explicitly ensures the daemon and connects to the resulting endpoint

#### Scenario: CLI reads sessions
- **WHEN** the daemon has a session visible to app
- **THEN** CLI session list returns the same authoritative session identifier and daemon instance

### Requirement: control-stdio is a transport bridge
`control-stdio` SHALL parse stdin control JSON up to 4 MiB per message, issue async daemon requests, and write corresponding responses/events. It MUST reject larger input before JSON deserialization and MUST NOT own the runtime dispatcher.

#### Scenario: Valid stdio request arrives
- **WHEN** stdin provides a valid bounded control request
- **THEN** the command forwards it to daemon and emits the correlated response

#### Scenario: Unknown method arrives
- **WHEN** stdin names a method absent from daemon capabilities
- **THEN** output preserves the daemon `method_not_found` distinction

### Requirement: MCP uses async daemon-backed handlers
MCP runtime tools SHALL hold async `HomieClient`, filter advertised tools by exact capabilities, and map daemon `method_not_found` to JSON-RPC `-32601`.

#### Scenario: MCP lists tools
- **WHEN** daemon Hello omits an unavailable runtime capability
- **THEN** MCP does not advertise the dependent tool

#### Scenario: MCP invokes an unknown tool or method
- **WHEN** request has no registered executable handler
- **THEN** MCP returns JSON-RPC `-32601`, not generic `-32000`

#### Scenario: MCP connection closes
- **WHEN** MCP stdio server shuts down
- **THEN** it closes only its client connection and does not shut down the daemon

### Requirement: GPUI uses a two-worker async bridge
The desktop app SHALL create a Tokio multi-thread runtime with exactly two worker threads for client transport and service bridge tasks. It SHALL update GPUI projection only through GPUI-safe update/message paths.

#### Scenario: App renders first frame
- **WHEN** app starts and daemon is unavailable or slow
- **THEN** GPUI first frame renders without waiting for daemon readiness, SQLite, PTY, process, or network work

#### Scenario: Runtime state changes
- **WHEN** async client publishes connection, snapshot, event, or terminal updates
- **THEN** the bridge schedules projection updates on the GPUI context

#### Scenario: Runtime disconnects
- **WHEN** daemon connection drops
- **THEN** UI shows degraded/reconnecting/disconnected state and does not create an embedded runtime fallback

### Requirement: Live session projection has one source
App session list, spawn, send, resize, snapshot, events, and terminal data SHALL come through the daemon client. Existing settings/doctor/usage storage access MUST NOT be used as a live session source.

#### Scenario: Session is spawned through app
- **WHEN** user triggers the existing Wave 1A spawn action
- **THEN** app sends the daemon command and updates projection from response/event rather than a local storage write

#### Scenario: Terminal is attached
- **WHEN** user selects a live session
- **THEN** app opens `terminal.v1` and applies replay/full-grid/live frames in order

### Requirement: Embedded client APIs are removed
The implementation SHALL delete `HomieClient::open(data_dir)`, `open_with_runtime`, synchronous production facade, client runtime dispatcher, and client runtime/storage dependencies after all consumers migrate.

#### Scenario: Workspace source is scanned
- **WHEN** migration is complete
- **THEN** production callers contain no embedded client constructor or RuntimeSupervisor construction outside daemon/runtime ownership

#### Scenario: Legacy API is referenced
- **WHEN** an unmigrated caller still uses the deleted API
- **THEN** compilation fails until that caller is migrated; no compatibility shim is added

### Requirement: Daemon binary is included in package closure
Build/package scripts SHALL build and place `homie-runtime-daemon` at the fixed app-bundle location used by launcher absolute-path resolution.

#### Scenario: Packaged smoke runs
- **WHEN** the assembled app bundle starts in a clean temporary data directory
- **THEN** launcher finds the bundled executable, client completes Hello/snapshot, and app/CLI observe the same daemon instance ID

#### Scenario: Bundled daemon is missing
- **WHEN** launcher cannot resolve the fixed daemon path
- **THEN** startup exposes a safe launch error and does not search PATH or environment variables

### Requirement: Cross-entry shared-daemon evidence is required
Wave 1A SHALL not pass until app, CLI, and MCP have been verified against one daemon instance and reconnect behavior has been exercised.

#### Scenario: Three entries connect
- **WHEN** app, CLI, and MCP target the same absolute data directory
- **THEN** evidence records the same safe daemon instance ID and authoritative session/event cursor

#### Scenario: Holder regression remains
- **WHEN** transport gates pass but current holder adoption still returns detached
- **THEN** Wave 1A records the transport result honestly and retains T-102 as a blocker for full runtime parity
