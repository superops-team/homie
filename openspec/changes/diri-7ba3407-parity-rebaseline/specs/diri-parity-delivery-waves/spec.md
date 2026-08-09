## ADDED Requirements

### Requirement: Dependency-ordered vertical waves
The parity program SHALL use the Wave 1A through Wave 5B change IDs and dependency order defined in the source PRD.

#### Scenario: A product wave is planned
- **WHEN** a Wave 2 or Wave 3 implementation is prepared
- **THEN** its required Wave 1 runtime/client/storage dependencies are green or the task is explicitly blocked

#### Scenario: Remote work begins
- **WHEN** Wave 4A or Wave 4B implementation starts
- **THEN** the shared runtime/client boundary is already accepted and the wave does not introduce an alternative transport owner

### Requirement: Independent change artifacts
Each implementation wave MUST have its own Bead, Chinese PRD, affected component spec review, OpenSpec artifacts and verification directory before code changes.

#### Scenario: Wave implementation is requested
- **WHEN** an engineer starts a wave
- **THEN** the stable change ID resolves to matching Bead metadata, PRD path, OpenSpec directory and evidence directory

#### Scenario: Required artifact is absent
- **WHEN** proposal, design, capability spec, plan, task mapping, alignment or spec review is missing
- **THEN** implementation is blocked until the artifact is completed

### Requirement: End-to-end wave acceptance
A wave SHALL close only after its user-visible or cross-process vertical path passes, not merely its leaf libraries.

#### Scenario: Library tests pass without product wiring
- **WHEN** parser, DTO, repository or UI model tests pass but the owning user workflow is not connected
- **THEN** the wave remains partial

#### Scenario: Wave evidence is complete
- **WHEN** RED/GREEN tests, integration, E2E, security and required manual evidence match the wave PRD
- **THEN** its Bead may close with a release-readiness evidence link

### Requirement: No compatibility shortcut
When a wave replaces an incorrect internal boundary, it MUST remove that production shortcut instead of retaining a compatibility fallback.

#### Scenario: Endpoint client replaces embedded runtime
- **WHEN** Wave 1A switches consumers to the daemon client
- **THEN** production constructors that embed `RuntimeSupervisor` and direct consumer storage access are deleted
