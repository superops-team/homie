## ADDED Requirements

### Requirement: Single-source direct lineage and safe audit
`sessions.parent_session_id` SHALL remain the single direct-parent fact. Version 4 SHALL add
idempotent safe lineage audit metadata rather than a second parent graph.

#### Scenario: Child session is created
- **WHEN** session, parent, and frozen effective config are valid
- **THEN** their relationship commits atomically and direct parent/children queries agree

#### Scenario: Lineage action is audited
- **WHEN** an owned service records a lineage decision
- **THEN** it stores a unique operation id, actor, subject, relation/action, decision, safe reason code, and timestamp

#### Scenario: Audit operation is replayed
- **WHEN** the same operation id is submitted again
- **THEN** the repository returns the original durable result or a stable conflict without duplicating the event

### Requirement: Remote operation metadata foundation
Existing host, node-account, and handoff storage SHALL gain typed repository and idempotency
contracts without implementing remote execution.

#### Scenario: Handoff metadata is created
- **WHEN** a remote owner records an operation
- **THEN** operation id, checkpoint id, phase, lease id, manifest hash, source session, target host, mode, safe error, and timestamps are durable

#### Scenario: Lease or checkpoint is replayed
- **WHEN** a duplicate operation/lease attempts an incompatible transition
- **THEN** compare-and-set validation fails closed and the prior phase is preserved

#### Scenario: Remote metadata is scanned
- **WHEN** stored rows and serialized snapshots are inspected
- **THEN** they contain no checkpoint blob, provider home, node token, raw provider key, Authorization, cookie, or SSH private material

### Requirement: Durable update receipt foundation
Version 4 SHALL provide idempotent update receipts and constrained phase transitions without
performing update network or install behavior.

#### Scenario: Update receipt is created
- **WHEN** the updater owner records an operation
- **THEN** operation id, source/target version, phase, feed host, archive SHA256, bundle/team identity, path references, safe error, and timestamps are durable

#### Scenario: Update phase changes
- **WHEN** the expected current phase matches a legal transition
- **THEN** the repository atomically advances the receipt

#### Scenario: Update phase regresses or races
- **WHEN** expected phase is stale or the requested transition is illegal
- **THEN** the repository returns a stable conflict and does not alter the receipt

#### Scenario: Update receipt is scanned
- **WHEN** receipt storage is inspected
- **THEN** it contains no feed Authorization/basic-auth value, cookie, response body, signing private key, or executable payload

### Requirement: Foundation does not close downstream parity
Repository and schema tests for lineage, remote, or update metadata SHALL prove foundation only.

#### Scenario: T-103 evidence passes
- **WHEN** all migration/repository/service/restart gates in this change are green
- **THEN** UI, recursive lineage permission, remote node/handoff, usage workflow, updater, packaging, and performance rows remain partial until their owning E2E evidence passes
