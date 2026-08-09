## ADDED Requirements

### Requirement: HomieClient is a pure async transport client
`homie-client` SHALL expose Tokio async connection, request, event stream, terminal stream, and close APIs. It MUST NOT depend on `homie-runtime` or `homie-storage` and MUST NOT construct a runtime, open SQLite, execute git/worktree logic, or spawn a daemon.

#### Scenario: Client crate dependencies are inspected
- **WHEN** the Wave 1A implementation is complete
- **THEN** `homie-client/Cargo.toml` contains no runtime/storage dependency and production source contains no RuntimeSupervisor construction

#### Scenario: Client connects to an unavailable endpoint
- **WHEN** caller invokes connect and no daemon is available
- **THEN** client reports/retries transport availability according to connection lifecycle without spawning a process

### Requirement: Client publishes explicit connection state
The client SHALL publish disconnected, connecting, handshaking, connected, degraded, reconnecting, and terminal shutdown states.

#### Scenario: Initial connection succeeds
- **WHEN** UDS connect and Hello complete
- **THEN** observers see connecting, handshaking, and connected with daemon instance/capabilities

#### Scenario: Heartbeat times out
- **WHEN** no traffic is available and Ping receives no Pong within 10 seconds
- **THEN** client publishes degraded then reconnecting and closes the stale connection

#### Scenario: Client is explicitly closed
- **WHEN** caller closes the client
- **THEN** it enters shutdown, fails pending work, closes streams, and performs no further reconnect attempts

### Requirement: Heartbeat and reconnect are bounded
The client SHALL send heartbeat after 25 seconds of idle connection time and SHALL reconnect with exponential backoff from 500 milliseconds to 8 seconds. A successful Hello MUST reset the backoff.

#### Scenario: Daemon is temporarily unavailable
- **WHEN** connection attempts fail repeatedly
- **THEN** attempts follow bounded exponential delays and do not busy-loop

#### Scenario: Daemon returns
- **WHEN** a reconnect attempt completes Hello
- **THEN** the client resets backoff and begins stream recovery

### Requirement: Pending requests fail deterministically
Request timeout SHALL remove only that pending message. Ordinary requests SHALL use a 10-second timeout; typed long-running methods SHALL use their server deadline plus 5 seconds. Connection loss or shutdown SHALL fail all remaining pending messages once. The client MUST NOT automatically replay any request.

#### Scenario: One request times out
- **WHEN** a request exceeds its configured timeout while connection remains live
- **THEN** that caller receives `timeout` and unrelated pending requests remain valid

#### Scenario: Connection drops with pending mutations
- **WHEN** UDS closes before responses arrive
- **THEN** every pending caller receives unavailable exactly once and no mutation is resent after reconnect

#### Scenario: Caller cancels a request
- **WHEN** the request future is dropped before its response arrives
- **THEN** the client removes the pending waiter and safely ignores the late response without closing the healthy connection

#### Scenario: Started worktree mutation exceeds caller wait
- **WHEN** the client timeout/cancellation occurs after a worktree mutation started
- **THEN** the client removes its waiter without assuming the server mutation stopped or replaying it

### Requirement: Event gap recovery replaces projection from snapshot
The client SHALL retain last confirmed event sequence. On `event_gap`, it SHALL call `state.snapshot`, replace the consumer projection through a snapshot event, and reopen event stream from the snapshot cursor.

#### Scenario: Reconnect cursor is replayable
- **WHEN** daemon reconnects and retained cursor remains in the event ring
- **THEN** client reopens events after that cursor without requesting snapshot

#### Scenario: Reconnect cursor is stale
- **WHEN** daemon resets the event stream with `event_gap`
- **THEN** client obtains one authoritative snapshot and resumes after its cursor

### Requirement: Terminal stream recovery uses confirmed output offset
The client SHALL retain each terminal stream's session ID, last confirmed output offset, and latest grid sequence. After connection loss or stream reset, it SHALL reopen from the output offset and discard old grid projection until a new full Grid arrives.

#### Scenario: Daemon restarts with terminal open
- **WHEN** daemon instance changes after reconnect
- **THEN** client reopens the terminal from last confirmed offset and publishes no live diff before the new full grid

#### Scenario: Slow consumer reset occurs
- **WHEN** server returns `slow_consumer`
- **THEN** terminal handle reports resync state and may reopen without affecting other client streams

### Requirement: Client writer preserves priority and fairness
The client SHALL enqueue control, stream lifecycle, input, resize, and heartbeat frames in the high-priority queue and SHALL preserve the 32-high-frame fairness quota.

#### Scenario: Request and terminal output coexist
- **WHEN** low-priority terminal data is active and caller sends a control request
- **THEN** the request is sent with high priority while low streams continue receiving fair service

#### Scenario: High-priority queue is full
- **WHEN** the 256-frame high queue cannot accept a frame
- **THEN** the initiating operation receives `backpressure` without creating an unbounded task/channel

### Requirement: Client errors preserve protocol distinctions
The typed client SHALL map daemon stable errors without collapsing method-not-found, timeout, unavailable, backpressure, resync-required, and internal errors.

#### Scenario: Unsupported method is requested
- **WHEN** a typed or generic request calls a method absent from capabilities
- **THEN** the caller receives `method_not_found`

#### Scenario: Terminal requires resynchronization
- **WHEN** local sequence or decoded queue state cannot continue safely
- **THEN** the caller receives `resync_required` with safe last position
