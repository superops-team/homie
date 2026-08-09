## ADDED Requirements

### Requirement: Integrated context facts
Session context SHALL persist safe runtime events, lineage, artifact/task/memory references and output offsets through the shared storage boundary.

#### Scenario: Runtime session is created
- **WHEN** spawn commits successfully
- **THEN** context creates the session root and subsequent CLI/MCP/UI consumers observe the same safe summary

#### Scenario: Spawn fails
- **WHEN** runtime does not create a live session
- **THEN** no context root or successful session fact is committed

### Requirement: Source-attributed memory
Memory candidates MUST reference an existing redacted context event and retrieval MUST enforce workspace/profile/session permission.

#### Scenario: Candidate lacks a safe source
- **WHEN** content is submitted without a valid source event or fails redaction
- **THEN** the memory write is rejected

#### Scenario: Cross-workspace search is denied
- **WHEN** the caller lacks permission for the target workspace
- **THEN** matching records are not returned

### Requirement: Durable shared tasks
Tasks SHALL support create, claim, block, complete, return and cancel through an atomic repository and SHALL track inactive session owners.

#### Scenario: Two sessions claim one task
- **WHEN** concurrent claim requests target an open task
- **THEN** exactly one claim commits and the other returns a stable conflict

#### Scenario: Claimed session exits
- **WHEN** runtime reports the owner inactive
- **THEN** the task remains durable with inactive-owner state until explicit recovery

### Requirement: Deterministic orchestrator execution
The orchestrator SHALL convert UI, CLI and MCP intents into typed decisions and execute them through runtime/task/context/memory service boundaries.

#### Scenario: High-risk target is ambiguous
- **WHEN** an intent could release, delete, move remote state or access credentials without one clear target
- **THEN** the orchestrator requests explicit user choice and performs no mutation

#### Scenario: Route executes
- **WHEN** a permitted route succeeds
- **THEN** it records safe decision provenance and all consumers observe the same resulting fact

### Requirement: Separate extension status
Homie control-plane completion SHALL be reported separately from Diri parity completion.

#### Scenario: Diri parity passes but extension is incomplete
- **WHEN** all Diri-required capabilities pass and a Homie extension remains partial
- **THEN** the release reports Diri parity complete and Homie control-plane partial without conflating the two
