## ADDED Requirements

### Requirement: Preserve the schema-v3 baseline
The change SHALL treat ordered transactional migrations through schema version 3,
`effective_agent_configs`, session core metadata, and history/worktree/usage repositories as
existing behavior.

#### Scenario: Baseline verification runs
- **WHEN** T-103 starts its RED phase
- **THEN** the existing `homie-storage` test binaries pass with their current assertions before any v4 implementation is added

#### Scenario: A new RED case is written
- **WHEN** the case targets T-103
- **THEN** it fails on a missing v4, service ownership, recovery, or freeze contract rather than falsely asserting that a schema-v3 capability is absent

### Requirement: Ordered transactional v4 migration
Schema version 4 MUST append to the existing v1, v2, and v3 migration chain and MUST atomically
apply its DDL, data backfill, indexes, and migration-version record.

#### Scenario: Empty database migrates
- **WHEN** migration runs against an empty data directory
- **THEN** the applied versions are ordered as 1, 2, 3, and 4 and the resulting schema version is 4

#### Scenario: Existing version-3 database migrates
- **WHEN** migration runs against a real version-3 fixture
- **THEN** only version 4 is applied and all existing session/history/worktree/usage facts remain readable

#### Scenario: Version-4 migration fails
- **WHEN** a deterministic failure is injected after at least one v4 statement
- **THEN** no v4 table, column, index, backfill, or schema-migration row remains committed

### Requirement: Migration idempotency and fail-closed versioning
Migration SHALL be idempotent at version 4 and SHALL reject a database with a newer schema.

#### Scenario: Migration is repeated
- **WHEN** migrate is called again after version 4 committed
- **THEN** the applied list is empty and durable facts are unchanged

#### Scenario: Database schema is newer
- **WHEN** the highest schema migration version exceeds 4
- **THEN** storage returns `SchemaTooNew` without executing any migration or fallback
