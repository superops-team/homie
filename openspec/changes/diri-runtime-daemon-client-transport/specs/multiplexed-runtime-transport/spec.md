## ADDED Requirements

### Requirement: Connections use a fixed binary preface and frame
The transport SHALL begin with the 12-byte `HOMIEIPC` preface and SHALL encode every message as a big-endian length-prefixed frame with version, kind, flags, stream ID, message ID, sequence, and payload fields.

#### Scenario: Partial and coalesced reads decode
- **WHEN** preface and frames arrive across arbitrary read boundaries or multiple frames arrive in one read
- **THEN** the codec emits exactly the complete frames in wire order

#### Scenario: Hostile length is rejected
- **WHEN** `frame_len` is smaller than the 24-byte header or exceeds 16 MiB
- **THEN** the codec rejects the connection before allocating the declared payload

#### Scenario: Unknown wire values fail closed
- **WHEN** version, kind, flags, or payload structure is invalid for Wave 1A
- **THEN** the server closes the connection with a safe protocol error

### Requirement: Payload encoding is selected by frame kind
The transport SHALL encode Hello, Request, Response, Event, and stream metadata with bounded Serde JSON. It SHALL encode Output, Input, Resize, Grid, and Modes as raw binary payloads.

#### Scenario: Control JSON exceeds limit
- **WHEN** a JSON control payload exceeds 4 MiB
- **THEN** the codec rejects it without deserialization

#### Scenario: Terminal output is sent raw
- **WHEN** the server sends terminal bytes
- **THEN** it emits Output frames of at most 64 KiB without base64 encoding

### Requirement: Stream ID ownership is enforced
The connection SHALL reserve stream ID 0 for control, client-created odd IDs for event/terminal streams, and even IDs for server-created streams.

#### Scenario: Client opens a valid stream
- **WHEN** a client sends StreamOpen on an unused odd stream ID
- **THEN** the server validates the stream kind and replies on the same stream ID

#### Scenario: Client violates stream ownership
- **WHEN** a client opens stream 0, an even stream, or a reused active ID
- **THEN** the server rejects the protocol operation without corrupting other stream state

### Requirement: Hello negotiates version and exact capabilities
Hello SHALL be the first frame. HelloAck SHALL include selected wire version, daemon build, PID, instance ID, executable SHA-256, exact method/stream capabilities, and event oldest/latest sequence.

#### Scenario: Compatible client handshakes
- **WHEN** client and daemon have the same wire major and compatible minor
- **THEN** the daemon returns HelloAck and permits control/stream traffic

#### Scenario: Wire major differs
- **WHEN** the client wire major differs from daemon
- **THEN** the daemon returns or records `version_mismatch` and closes without compatibility fallback

#### Scenario: Proto constant has no handler
- **WHEN** a method constant exists but is absent from the daemon handler registry
- **THEN** HelloAck omits it and a direct request returns `method_not_found`

### Requirement: Control requests are correlated and bounded
Each Request SHALL use a non-zero connection-unique message ID and each Response SHALL reuse that ID. A client connection MUST cap pending requests at 1024.

#### Scenario: Responses arrive out of order
- **WHEN** concurrent requests complete in a different order
- **THEN** the client resolves each waiting caller by message ID

#### Scenario: Pending limit is reached
- **WHEN** a caller attempts request 1025 while 1024 remain pending
- **THEN** the client returns `backpressure` without enqueueing the request

#### Scenario: Unknown method is called
- **WHEN** a Request names a method outside the exact registry
- **THEN** the daemon returns a Response with `method_not_found`

### Requirement: Stable errors exclude sensitive payloads
The transport SHALL use only the stable Wave 1A error codes and SHALL not include raw payload, terminal bytes, argv, environment, credentials, cookies, or tool arguments/results in errors or logs.

#### Scenario: Handler returns an internal error
- **WHEN** a production handler fails unexpectedly
- **THEN** the client receives `internal` with safe context and the sensitive source error remains unexposed

#### Scenario: Peer is unauthorized
- **WHEN** local peer UID validation fails
- **THEN** the server records `unauthorized` without recording peer payload

### Requirement: Event streams replay or reset explicitly
An event stream SHALL accept `afterSeq`, emit Event frames with runtime event sequence, and replay from a 1024-entry ring. The server MUST reset with `event_gap` if the requested cursor is no longer available or delivery overflows.

#### Scenario: Event cursor is replayable
- **WHEN** `afterSeq` is within the event ring
- **THEN** the stream emits every later event in increasing sequence before live events

#### Scenario: Event cursor is too old
- **WHEN** `afterSeq` precedes the oldest retained event
- **THEN** the server sends `StreamReset(event_gap,latest_seq)` and emits no silent partial replay

### Requirement: Terminal streams have deterministic replay and live order
A terminal stream SHALL emit StreamOpened, ReplayBegin, zero or more offset-bearing Output frames, ReplayEnd, full Grid, Modes, then live frames. Stream sequence MUST increase monotonically. The daemon MUST share one `TerminalSource` per attached session across client streams.

#### Scenario: Terminal opens from an offset
- **WHEN** client opens `terminal.v1` with a valid session and output offset
- **THEN** server replays bytes from that offset, sends a full grid, and begins live delivery in the required order

#### Scenario: Input and resize arrive
- **WHEN** the client sends Input or Resize on an open terminal stream
- **THEN** the server routes the command to the bound session through RuntimeActor

#### Scenario: Multiple clients attach one session
- **WHEN** multiple terminal streams attach the same session
- **THEN** they subscribe to one daemon terminal source without multiplying output-log readers or actor polling

#### Scenario: Terminal stream closes
- **WHEN** either side closes or resets the stream
- **THEN** the runtime session continues running

### Requirement: Writer queues are bounded and fair
Each connection SHALL use a 256-frame high-priority queue and a 256-frame low queue per non-control stream. It SHALL permit at most 64 active streams and SHALL attempt a round-robin low frame after at most 32 consecutive high frames.

#### Scenario: High-priority input competes with output
- **WHEN** terminal output is continuously available and the client sends input
- **THEN** input is scheduled through the high-priority queue without permanently starving low output

#### Scenario: One terminal is slow
- **WHEN** one terminal low queue reaches 256 frames
- **THEN** the server resets only that stream with `slow_consumer` and continues control and other streams

#### Scenario: Server high-priority queue is full
- **WHEN** a connection's 256-frame high-priority queue cannot accept a required frame
- **THEN** the server closes that connection instead of allocating more memory or silently dropping control

#### Scenario: Decoded client queue is full
- **WHEN** a client terminal consumer does not drain its 256-frame decoded queue
- **THEN** the client closes that local stream and exposes `ResyncRequired(last_confirmed_offset)`
