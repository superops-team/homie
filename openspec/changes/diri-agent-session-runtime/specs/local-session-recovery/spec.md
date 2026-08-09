## ADDED Requirements

### Requirement: Manifest resume SHALL relaunch the original conversation directly

An exited or archived session SHALL resume through its frozen manifest semantics under the same
Homie session identity. Resume SHALL NOT start a shell and inject a textual command.

#### Scenario: ID-based conversation resumes

- **WHEN** a frozen manifest declares ID-based resume and the session has a verified durable agent session ID
- **THEN** the runtime SHALL launch the manifest resume argv directly under a holder-owned PTY
- **AND** it SHALL keep the Homie session ID, title, parent, profile, permission, and output history

#### Scenario: Latest-style conversation resumes

- **WHEN** a frozen manifest explicitly declares latest-style resume
- **THEN** the runtime SHALL use that exact manifest resume argv without inventing an ID

#### Scenario: Resume prerequisites are missing

- **WHEN** required agent ID, frozen config, executable, cwd, or resumability evidence is unavailable
- **THEN** resume SHALL fail with a stable not-resumable/invalid-config error
- **AND** the existing record and output SHALL remain retryable

#### Scenario: Holder is already live

- **WHEN** resume/relaunch discovers a verified live holder for the target Homie session
- **THEN** it SHALL adopt the holder instead of launching another holder or agent process

### Requirement: Resume SHALL preserve output and checkpoint continuity

Resume SHALL append a new holder/output epoch to the existing session stream and SHALL retain the
latest valid screen/output checkpoint.

#### Scenario: New holder incarnation becomes ready

- **WHEN** resume starts a new holder and verifies its child before the readiness deadline
- **THEN** the runtime SHALL append a new epoch boundary
- **AND** old output SHALL remain readable
- **AND** new input/output SHALL use the same Homie session endpoint

#### Scenario: New holder incarnation fails readiness

- **WHEN** the resume holder or child fails before readiness commit
- **THEN** the runtime SHALL clean that failed incarnation
- **AND** it SHALL not project the session as running
- **AND** the prior record, checkpoint, and output SHALL remain intact

### Requirement: Archive and unarchive SHALL be distinct from resume

Archive SHALL terminate runtime resources while retaining a resumable record. Unarchive SHALL
change archival visibility/state but SHALL not automatically start a process.

#### Scenario: Live session is archived

- **WHEN** archive is requested for a live or hibernated session
- **THEN** the runtime SHALL terminate and reap the holder tree
- **AND** mark the retained record archived/resumable according to manifest evidence

#### Scenario: Session is unarchived

- **WHEN** unarchive is requested for an archived record
- **THEN** the record SHALL become visible as an offline session
- **AND** no holder SHALL start until an explicit resume

### Requirement: T-102 SHALL deliver only local migration substrate

T-102 SHALL provide local checkpoint and same-session stop/relaunch/resume primitives needed by a
later migration transaction. It SHALL NOT advertise or claim remote migration.

#### Scenario: Local relaunch substrate is prepared

- **WHEN** a same-host lifecycle operation requires replacement of an exited holder incarnation
- **THEN** the runtime SHALL flush screen/output checkpoint and frozen effective config before relaunch
- **AND** it SHALL withhold running projection until the target holder is ready

#### Scenario: Local relaunch fails

- **WHEN** target readiness fails
- **THEN** the source session record, effective config, checkpoint, and output SHALL remain available for retry
- **AND** the runtime SHALL not return migration success

#### Scenario: Remote migration is requested

- **WHEN** a caller attempts remote host transfer, move/fork handoff, transcript transfer, or lease behavior
- **THEN** the runtime SHALL expose no T-102 production capability or false success for it
- **AND** RT-010 SHALL remain partial for the later remote owner

### Requirement: Prepare shutdown SHALL quiesce and flush lifecycle facts

Prepare shutdown SHALL stop accepting new lifecycle mutations, stop new governor ticks, drain
accepted work to a deadline, and flush canonical runtime facts.

#### Scenario: Prepare shutdown with live sessions

- **WHEN** prepare shutdown begins while sessions are live or hibernated
- **THEN** new spawn/resume/archive/hibernate mutations SHALL be rejected as shutting down
- **AND** reducer status, needs-input, screen checkpoint, output cursor, event store, and SQLite WAL SHALL be flushed
- **AND** holders SHALL remain alive

#### Scenario: Accepted mutation exceeds drain deadline

- **WHEN** in-flight lifecycle work cannot finish before the existing shutdown deadline
- **THEN** prepare shutdown SHALL report the bounded timeout outcome
- **AND** it SHALL not wait indefinitely or kill unrelated holders

### Requirement: Shutdown SHALL preserve holder adoption continuity

The daemon SHALL preserve Wave 1A ACK ordering and SHALL leave live/hibernated holders available to
a replacement daemon.

#### Scenario: Graceful daemon shutdown

- **WHEN** shutdown is accepted after prepare
- **THEN** the server SHALL send the shutdown ACK before closing transport/actor tasks
- **AND** it SHALL not terminate live or hibernated holder trees

#### Scenario: Hard daemon crash and replacement

- **WHEN** the daemon is killed without graceful shutdown while a holder remains live
- **THEN** a replacement daemon SHALL use holder-first reconciliation to adopt it
- **AND** input/output/status SHALL recover without a second child

### Requirement: Recovery E2E SHALL use real local processes and exact cleanup

Blocking recovery tests SHALL use the packaged daemon binary, packaged holder binary, a real PTY,
and a real local fake-agent executable. Mocks alone SHALL not satisfy release evidence.

#### Scenario: Cross-entry restart E2E passes

- **WHEN** the fixture spawns an agent through one client entry, kills/restarts the daemon, and reopens through another entry
- **THEN** the same holder and PTY SHALL continue
- **AND** output before and after restart SHALL be observable
- **AND** status/storage/snapshot SHALL agree

#### Scenario: Fixture cleanup completes

- **WHEN** an E2E case passes, fails, panics, or times out
- **THEN** cleanup SHALL target only recorded fixture daemon/holder/child PIDs and control files
- **AND** verify fixture-owned process/socket/pid-file count is zero
- **AND** leave pre-existing user holders untouched

### Requirement: Current regression facts SHALL remain exact

Release evidence SHALL use the checkpoint's two-RED/one-retained-GREEN baseline and SHALL not
rewrite historical Wave 1A evidence.

#### Scenario: Focused lifecycle suite is accepted

- **WHEN** T-102 implementation reaches release verification
- **THEN** `runtime_reopen_can_adopt_holder_and_continue_session` SHALL pass unchanged
- **AND** `runtime_spawn_shell_uses_live_pty` SHALL pass unchanged
- **AND** `runtime_holder_stat_tracks_resize_and_log_offsets` SHALL remain green
- **AND** the complete 14-test suite SHALL pass at least five consecutive serial runs without leaks

#### Scenario: Parity evidence is recorded

- **WHEN** local runtime requirements pass but remote migration remains unimplemented
- **THEN** only locally proven parity rows MAY be advanced by the master owner
- **AND** RT-010 remote migration SHALL remain partial
