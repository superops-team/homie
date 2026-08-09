## ADDED Requirements

### Requirement: Manifest-driven agent spawn
Runtime session spawn SHALL resolve and freeze an `EffectiveAgentConfig` and execute the selected manifest binary, argv, scrubbed env, injection and permission profile.

#### Scenario: First-class agent is available
- **WHEN** a user spawns an enabled agent profile whose binary resolves
- **THEN** runtime starts that manifest command under holder ownership and records the immutable effective config

#### Scenario: Agent binary is unavailable
- **WHEN** readiness cannot resolve the configured binary
- **THEN** spawn fails without creating a half-created session, context root or virtual key leak

### Requirement: Durable holder lifecycle
The holder SHALL preserve PTY and output ownership across app/runtime failure and SHALL report enough stat data for deterministic adoption.

#### Scenario: Runtime restarts
- **WHEN** a holder-owned child and output log remain live
- **THEN** the restarted runtime adopts the holder and reports the session as live only after holder verification

#### Scenario: Holder evidence is missing
- **WHEN** storage says a session was running but holder/process evidence cannot be verified
- **THEN** runtime reports detached or exited and rejects live input

### Requirement: Complete lifecycle operations
Runtime SHALL implement attach, read, send, resize, wait, kill, release, archive, unarchive, hibernate, wake, resume and migrate with stable state transitions.

#### Scenario: Session is hibernated and woken
- **WHEN** hibernate then wake succeeds
- **THEN** process/holder state, screen checkpoint and input availability match the canonical state machine

#### Scenario: Session migration fails
- **WHEN** checkpoint transfer or target restore fails before commit
- **THEN** the source session remains authoritative and usable

### Requirement: Controlled daemon shutdown
Runtime MUST support prepare-shutdown and shutdown with bounded drain and deterministic flush order.

#### Scenario: Shutdown is requested
- **WHEN** the daemon receives prepare-shutdown followed by shutdown
- **THEN** it stops accepting new work, flushes durable indexes/events and preserves or terminates sessions according to explicit policy
