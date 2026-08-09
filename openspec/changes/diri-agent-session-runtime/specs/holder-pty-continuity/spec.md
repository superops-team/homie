## ADDED Requirements

### Requirement: Startup SHALL reconcile holder evidence before rewriting session state

The runtime SHALL read persisted session facts and probe the expected holder before committing a
startup projection. It SHALL NOT bulk-mark live candidates detached before holder adoption.

#### Scenario: Running holder repairs a stale detached projection

- **WHEN** a persisted session is `created`, `starting`, `running`, or `detached` and its expected holder returns a successful running `Stat`
- **THEN** the runtime SHALL adopt the holder into the live registry
- **AND** the persisted and protocol projection SHALL be `running`
- **AND** the runtime SHALL NOT start a second holder or child

#### Scenario: Behavior status survives verified adoption

- **WHEN** a persisted session is `idle` or `needs_input` and its expected holder returns a successful running `Stat`
- **THEN** the runtime SHALL adopt the holder into the live registry
- **AND** it SHALL preserve the more specific behavior status

#### Scenario: Storage row cannot prove liveness

- **WHEN** a persisted session is marked live but holder IPC, child status, or expected holder-path evidence cannot be verified
- **THEN** the runtime SHALL project the session as `detached` or `exited`
- **AND** it SHALL reject live input
- **AND** it SHALL NOT infer running from the storage row

#### Scenario: Archived state contradicts a live holder

- **WHEN** a persisted session is archived but its expected holder reports a live child
- **THEN** startup reconciliation SHALL report a recovery contradiction
- **AND** it SHALL NOT silently project the archived session as running

### Requirement: Holder live evidence SHALL be the local liveness authority

A successful response from the expected session holder with a live child status SHALL be the
authority for local PTY/process liveness. A socket path without a successful live response SHALL
not be sufficient.

#### Scenario: Holder explicitly reports exit

- **WHEN** the expected holder or its durable status marker reports that the child exited
- **THEN** the runtime SHALL persist and project `exited`
- **AND** it SHALL omit the session from the live registry

#### Scenario: Holder disappears between probe and input

- **WHEN** a holder was adopted but disappears before a later input request
- **THEN** the input SHALL fail with a stable holder-unavailable/session-not-live error
- **AND** the runtime SHALL reconcile the session away from running

### Requirement: Holder adoption SHALL preserve the real PTY and output stream

The holder SHALL remain the sole owner of the PTY, child process tree, and output-log writer
across daemon/app/client restarts.

#### Scenario: Daemon restart continues the existing PTY

- **WHEN** the daemon exits while a holder-owned session is live and a replacement daemon opens the same absolute data directory
- **THEN** the replacement SHALL adopt the existing holder
- **AND** input, output, resize, screen, and snapshot operations SHALL continue on the same PTY
- **AND** the existing output log SHALL not be truncated

#### Scenario: Duplicate adoption attempt

- **WHEN** recovery or resume finds a verified live holder for the target session
- **THEN** the runtime SHALL adopt that holder
- **AND** it SHALL NOT launch another holder, child, PTY writer, or agent process

### Requirement: Holder stat SHALL retain geometry and offset continuity

Holder stat SHALL expose enough non-secret facts to verify process and terminal continuity,
including child PID/status, process-tree size, rows, columns, log offset, and epoch offset.

#### Scenario: Resize and output update stat

- **WHEN** a client resizes a live session and the child emits output
- **THEN** a later holder stat SHALL report the latest rows and columns
- **AND** log and epoch offsets SHALL be monotonic
- **AND** `runtime_holder_stat_tracks_resize_and_log_offsets` SHALL remain green

#### Scenario: Reopen reads prior output

- **WHEN** a replacement daemon adopts a holder with a non-zero output offset
- **THEN** attach/read SHALL continue from the requested prior offset
- **AND** new bytes SHALL append after the existing offset

### Requirement: Holder launch SHALL be structured and direct

New holder launches SHALL receive structured argv, cwd, sanitized environment, geometry, session
identity, and output path. The runtime SHALL NOT construct a shell command string for agent
launch.

#### Scenario: Direct executable launch

- **WHEN** the runtime supplies an absolute executable and argv vector
- **THEN** the holder SHALL spawn that vector directly in the PTY
- **AND** argument boundaries SHALL be preserved
- **AND** no intermediate shell command injection SHALL occur

#### Scenario: Invalid launch plan

- **WHEN** executable, cwd, geometry, or structured launch data fails validation
- **THEN** the holder SHALL reject the launch before creating a child
- **AND** the runtime SHALL roll back the uncommitted session

### Requirement: Holder operations SHALL be bounded and fixture-cleanable

Holder requests, launch readiness, process signals, and cleanup SHALL have fixed deadlines and
SHALL identify exact session resources. Real-process suites SHALL compare holder PID/start-time
sets before and after execution.

#### Scenario: Holder request times out

- **WHEN** a holder does not respond within the holder IPC deadline
- **THEN** the runtime SHALL return a stable timeout/unavailable error
- **AND** it SHALL not block the runtime actor or socket worker indefinitely

#### Scenario: Process test fails mid-assertion

- **WHEN** a real holder/PTY test exits early
- **THEN** its panic-safe guard SHALL terminate only its recorded holder, child tree, socket, and daemon
- **AND** it SHALL verify no fixture-owned process or control file remains
- **AND** it SHALL not use global process-name cleanup

#### Scenario: RED, panic, or timeout cleanup is checked against baseline

- **WHEN** a real-process suite fails an assertion, panics, times out, or completes normally
- **THEN** it SHALL resample holder PID and start-time facts after fixture cleanup
- **AND** the set of holders added after the pre-test baseline SHALL be empty
- **AND** pre-existing baseline holders SHALL remain untouched
