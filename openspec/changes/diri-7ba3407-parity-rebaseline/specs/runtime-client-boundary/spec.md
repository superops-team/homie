## ADDED Requirements

### Requirement: Independent runtime daemon
Live session, PTY, holder, event and attachment state SHALL be owned by an independent runtime daemon.

#### Scenario: Desktop application exits
- **WHEN** the app process exits while a session is running
- **THEN** the daemon/holder continue the session and a new client can reattach

#### Scenario: Multiple entry points connect
- **WHEN** app, CLI and MCP connect to the same endpoint
- **THEN** they observe the same session registry, event sequence and permission results

### Requirement: Endpoint-based runtime client
Production `homie-client` MUST connect through a versioned control/data transport and MUST NOT create `RuntimeSupervisor` or open storage.

#### Scenario: Client is constructed
- **WHEN** a production consumer creates `HomieClient`
- **THEN** it supplies a runtime endpoint and client role rather than a supervisor or data directory

#### Scenario: Runtime restarts
- **WHEN** the daemon connection drops and later becomes available
- **THEN** the client reconnects, resumes events from the last confirmed sequence and resynchronizes attachments

### Requirement: Attachment and backpressure
Terminal snapshots and diffs SHALL use a bounded attachment/data channel with epoch and sequence recovery.

#### Scenario: Client is slower than runtime output
- **WHEN** an attachment consumer exceeds its flow-control window
- **THEN** the runtime applies backpressure or drops/resynchronizes that attachment without terminating the session

#### Scenario: Attachment sequence gap occurs
- **WHEN** the client detects an epoch or sequence mismatch
- **THEN** it discards the incomplete projection and requests a full snapshot

### Requirement: Executable protocol catalog
Every published protocol method MUST have a production handler for the active runtime capabilities.

#### Scenario: Method is not implemented
- **WHEN** a method has no executable handler
- **THEN** it is absent from capability discovery and a direct unknown call returns method-not-found
