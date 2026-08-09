## ADDED Requirements

### Requirement: Every live session SHALL own one stateful status reducer

The runtime SHALL create one reducer per live session using its frozen manifest authority. Status
reads SHALL project that reducer state and SHALL NOT construct a new reducer on each request.

#### Scenario: Session is spawned

- **WHEN** a manifest-driven holder becomes ready
- **THEN** the runtime SHALL register one reducer with the manifest's authority and timing policy
- **AND** process readiness SHALL enter that reducer as a signal

#### Scenario: Status is queried repeatedly

- **WHEN** clients request status or snapshots without new runtime signals
- **THEN** all reads SHALL return the same canonical projection
- **AND** reads SHALL not replay the complete output or mutate reducer state

#### Scenario: Daemon adopts a live session

- **WHEN** startup reconciliation adopts a live holder
- **THEN** the runtime SHALL rebuild a reducer from frozen authority, persisted behavior facts, checkpoint, and bounded output replay
- **AND** it SHALL not infer liveness from persisted status alone

### Requirement: Runtime signals SHALL converge through the reducer

The runtime SHALL apply holder process, PTY output, manifest screen, hook, notify, user input, and
periodic tick signals to the same session reducer.

#### Scenario: User submits input

- **WHEN** accepted input is sent to a live session
- **THEN** the runtime SHALL emit the reducer user-input signal
- **AND** it SHALL clear stale needs-input according to reducer policy

#### Scenario: Process exits

- **WHEN** the holder reports child exit
- **THEN** process-exit SHALL enter the reducer
- **AND** the runtime SHALL persist exited before publishing the exit/status event

#### Scenario: Screen and hook disagree

- **WHEN** screen observation and a hook/notify event produce competing states
- **THEN** the frozen manifest authority and reducer timing SHALL determine the canonical state
- **AND** callers SHALL not bypass that arbitration with direct storage writes

### Requirement: Hook and notify ingestion SHALL be structured, redacted, and scoped

The CLI/parser boundary SHALL map supported hook/notify payloads to structured runtime signals.
Raw payloads SHALL not be persisted or published.

#### Scenario: Parent agent requests input

- **WHEN** a verified parent-session hook maps to needs-input
- **THEN** the reducer SHALL produce the canonical needs-input projection
- **AND** storage SHALL commit before the needs-input/status event is published

#### Scenario: Subagent hook is received

- **WHEN** a hook is identified as a subagent event
- **THEN** it MAY update subagent bookkeeping
- **AND** it SHALL not overwrite parent status, title, or needs-input

#### Scenario: Unsupported or sensitive hook payload is received

- **WHEN** a payload cannot be strictly decoded or contains sensitive free-form data
- **THEN** ingestion SHALL reject or reduce it to an allowlisted signal
- **AND** logs/events SHALL not contain the raw payload

### Requirement: Screen detection SHALL use the selected manifest engine

Screen observations SHALL be interpreted through the selected manifest engine and checkpointed
incrementally. An agent-agnostic phrase classifier SHALL not be the production authority.

#### Scenario: Manifest screen region matches

- **WHEN** bounded new PTY output produces a manifest-defined screen state
- **THEN** the runtime SHALL feed that observation to the reducer
- **AND** advance the persisted screen/output cursor only after successful processing

#### Scenario: Screen checkpoint is stale or missing

- **WHEN** the daemon restarts without a valid screen checkpoint
- **THEN** it SHALL rebuild from a bounded output window
- **AND** the checkpoint absence SHALL not prove the holder exited or running

### Requirement: Process-tree operations SHALL verify identity and remain bounded

The holder SHALL enumerate and signal the actual child tree using PID start-time checks. Runtime
workers SHALL not block socket tasks while doing process inspection.

#### Scenario: Tree is hibernated

- **WHEN** the runtime requests STOP for a verified live process tree
- **THEN** the holder SHALL signal the current tree and verify stopped state before success
- **AND** PID start-time mismatch SHALL fail without signaling the reused process

#### Scenario: Tree is woken

- **WHEN** a verified stopped tree is continued
- **THEN** descendants SHALL be resumed before the root where required
- **AND** the holder SHALL verify the original tree remains live

#### Scenario: Tree is terminated

- **WHEN** kill/archive requests termination
- **THEN** the holder SHALL send TERM and CONT, wait a bounded grace period, then send KILL and CONT to survivors
- **AND** it SHALL include detached descendants discovered from the original tree

### Requirement: Resource sampling SHALL expose safe session facts

Resource sampling SHALL report at least process-tree size and memory footprint without exposing
command lines, environment, prompts, or tool arguments.

#### Scenario: Sample succeeds

- **WHEN** the holder can inspect the live child tree
- **THEN** the runtime SHALL receive bounded tree-size and memory-footprint facts
- **AND** it MAY publish the sanitized session-resources event

#### Scenario: Sample fails

- **WHEN** process inspection is unavailable, races with exit, or times out
- **THEN** the sample SHALL be unknown
- **AND** the runtime SHALL not classify the session as exited or terminate it

### Requirement: Resource governor SHALL be conservative and daemon-scoped

One bounded daemon-level governor SHALL evaluate sessions at a fixed reviewed interval. It SHALL
not create an unbounded worker per session.

#### Scenario: Idle unattached session is eligible

- **WHEN** a session is idle, unpinned, unattached, and above the configured idle or eligible memory threshold
- **THEN** the governor MAY request hibernate through the runtime actor
- **AND** the decision SHALL be recorded with sanitized reason facts

#### Scenario: Active or needs-input session is protected

- **WHEN** a session is starting, running, needs-input, attached, or pinned
- **THEN** the governor SHALL not automatically hibernate or kill it

#### Scenario: Governor sample is backpressured

- **WHEN** the bounded actor/lane queue cannot accept resource work
- **THEN** the governor SHALL skip/defer the sample with an explicit bounded result
- **AND** it SHALL not block the connection hub

### Requirement: Hibernate and wake SHALL preserve the same PTY

Hibernate SHALL stop the verified process tree without terminating the holder. Wake SHALL continue
that tree without creating a new holder or shell.

#### Scenario: Hibernate then wake

- **WHEN** a live eligible session is hibernated and later woken
- **THEN** holder identity, child identity, PTY, output log, epoch/log offsets, and Homie session ID SHALL remain continuous
- **AND** post-wake input/output SHALL continue on the same process tree

#### Scenario: Input arrives while hibernated

- **WHEN** input is submitted to a hibernated session
- **THEN** the runtime SHALL return a stable session-hibernated error
- **AND** it SHALL not silently drop or queue ambiguous input

#### Scenario: Archive is requested

- **WHEN** an active or hibernated session is archived
- **THEN** the runtime SHALL terminate the holder/process tree
- **AND** preserve a resumable archived record
- **AND** it SHALL not treat archive as hibernate
