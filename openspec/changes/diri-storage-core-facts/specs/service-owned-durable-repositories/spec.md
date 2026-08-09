## ADDED Requirements

### Requirement: Daemon-owned production storage
The runtime daemon's actor/domain services SHALL be the only production owner of the SQLite
connection. App and CLI MUST access durable facts through typed client methods.

#### Scenario: App dependencies are inspected
- **WHEN** T-103 reaches GREEN
- **THEN** `homie-app` has no normal dependency on `homie-storage` and contains no production `open_or_create`, `StorageConfig`, or `open_ready_storage` path

#### Scenario: CLI dependencies are inspected
- **WHEN** T-103 reaches GREEN
- **THEN** `homie-cli` has no normal dependency on `homie-storage` and doctor/usage do not open SQLite directly

#### Scenario: Service code persists a fact
- **WHEN** runtime handles a settings, health, usage, config, recovery, lineage, remote-metadata, or update-metadata operation
- **THEN** the owning service calls a typed storage method and never exposes SQL, a transaction, `Storage`, or `Connection` to the client

### Requirement: Revisioned settings service
Settings reads and writes MUST use a revisioned typed service contract.

#### Scenario: Settings are loaded
- **WHEN** app calls `settings.get`
- **THEN** the service returns preferences plus the current revision from the durable repository

#### Scenario: Settings are updated from a stale revision
- **WHEN** `settings.update` carries an `expectedRevision` that does not match storage
- **THEN** the service returns a stable conflict and does not overwrite the newer preferences

#### Scenario: Settings persistence fails
- **WHEN** storage rejects or cannot commit an update
- **THEN** app receives a safe error and retains no false successful projection or direct-storage fallback

### Requirement: Service-backed health and usage queries
Storage health and usage summary SHALL be exposed as safe typed queries through the runtime
service/client boundary.

#### Scenario: CLI doctor requests health
- **WHEN** CLI calls `storage.health`
- **THEN** it receives schema version, foreign-key state, journal mode, and safe database identity without any live-session claim

#### Scenario: CLI requests usage summary
- **WHEN** CLI calls `usage.summary` with bounded filters
- **THEN** the service reuses the existing usage query semantics and returns safe aggregates without raw transcript or request content

### Requirement: Executable capability discovery
A frozen T-103 method MUST be advertised only after its handler and typed client path are
executable.

#### Scenario: A method constant exists without integration
- **WHEN** capability discovery is produced
- **THEN** that method is absent until its runtime handler and verification pass

#### Scenario: A direct unknown call is made
- **WHEN** the active daemon does not implement the requested method
- **THEN** it returns stable `method_not_found` rather than opening storage in the caller
