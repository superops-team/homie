## ADDED Requirements

### Requirement: Typed durable recovery facts
Storage SHALL provide one bounded typed recovery-fact projection per session by joining existing
session output metadata with version-4 holder/checkpoint/event/runtime metadata.

#### Scenario: Recovery facts are committed
- **WHEN** runtime checkpoints a session
- **THEN** holder hints, output epoch, checkpoint path/offset/content sequence, checkpointed event sequence, runtime instance, durable status, and timestamp commit atomically

#### Scenario: Recovery facts violate bounds
- **WHEN** an offset is negative, checkpoint offset exceeds the known output tail, or a snapshot version is unknown
- **THEN** the repository rejects the write without changing the prior recovery row

#### Scenario: Recovery candidates are listed
- **WHEN** daemon startup requests candidates
- **THEN** storage returns a deterministically ordered, explicitly bounded list rather than loading unbounded session history

### Requirement: Durable facts are not live proof
Persisted PID, holder instance, runtime instance, and last-observed status MUST be treated as
recovery hints and MUST NOT independently establish that a session is running.

#### Scenario: Storage says running but holder evidence is absent
- **WHEN** daemon recovery cannot verify holder identity and process evidence
- **THEN** runtime reports detached or exited and rejects live input

#### Scenario: PID has been reused
- **WHEN** a process exists at the persisted PID but holder instance/start evidence differs
- **THEN** runtime rejects adoption and does not update the session to running

#### Scenario: Holder evidence is verified
- **WHEN** T-102 runtime validation confirms holder identity, process state, and output log
- **THEN** runtime may commit a new durable assessment and publish the authoritative live projection

### Requirement: Output and checkpoint data separation
SQLite SHALL store only output/checkpoint paths, hashes, offsets, epochs, sequences, and safe
metadata. Terminal bytes, terminal grids, and checkpoint blobs MUST stay outside SQLite.

#### Scenario: Recovery metadata is inspected
- **WHEN** schema and fixtures are scanned
- **THEN** no output byte payload, terminal grid payload, raw prompt, or complete tool payload is present

### Requirement: Recovery survives daemon replacement
The production daemon SHALL reopen recovery facts after replacement and combine them with holder,
output, checkpoint, and event-log evidence.

#### Scenario: Daemon restarts with a surviving holder
- **WHEN** a new daemon opens the same data directory
- **THEN** it reads the frozen config and recovery facts, validates the surviving holder, and preserves event/output continuity

#### Scenario: Recovery commit fails
- **WHEN** a multi-field recovery assessment cannot commit
- **THEN** the previous durable facts remain intact and no partial status/event checkpoint is reported
