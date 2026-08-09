## ADDED Requirements

### Requirement: Authenticated first-party remote node
The remote node SHALL provide authenticated hello/capabilities, remote session operations, account state, usage and checkpoint/handoff services.

#### Scenario: Remote spawn succeeds
- **WHEN** a trusted host/node passes capability and token validation
- **THEN** runtime creates the remote session and exposes its events through the same client model

#### Scenario: Remote request contains provider raw key
- **WHEN** node, prefs sync, checkpoint or handoff receives a raw provider credential
- **THEN** the request is rejected and a secretless audit event is recorded

### Requirement: Transactional handoff
Move/fork handoff SHALL use preflight, checkpoint, incremental transfer, quarantine restore, target validation and lease commit.

#### Scenario: Restore fails before commit
- **WHEN** target restore or provider resume fails
- **THEN** source ownership and usability remain unchanged

#### Scenario: Commit request is replayed
- **WHEN** the same operation/lease ID is submitted again
- **THEN** the result is idempotent and does not create duplicate ownership

### Requirement: Unified usage and LLM proxy
Homie SHALL provide an OpenAI-compatible local proxy with scoped virtual keys, provider routing, streaming and a unified safe usage ledger.

#### Scenario: Managed agent streams a response
- **WHEN** a valid scoped virtual key calls the local proxy
- **THEN** the proxy injects upstream auth in short-lived memory, preserves stream semantics and records safe usage

#### Scenario: Metrics persistence fails
- **WHEN** provider response succeeds but usage/metrics storage fails
- **THEN** the response remains successful and `metrics.write_failed` is emitted without raw payload

### Requirement: Trusted package and updater
The release SHALL be a universal signed/notarized dependency-closed package with verified update install and rollback.

#### Scenario: Release credentials are unavailable
- **WHEN** Developer ID or notarization cannot run
- **THEN** the release gate is blocked and ad-hoc signing is not accepted as pass

#### Scenario: Staged update fails verification
- **WHEN** SHA256, host, bundle, team, version, codesign or spctl validation fails
- **THEN** the staged update is rejected and the current app remains intact

### Requirement: Real packaged performance gate
Packaged startup, attach, terminal, memory, event and updater performance MUST be measured against explicit budgets.

#### Scenario: Interactive release host is unavailable
- **WHEN** performance measurement cannot execute
- **THEN** the gate is `not_run` or `blocked` and the release cannot claim final parity
