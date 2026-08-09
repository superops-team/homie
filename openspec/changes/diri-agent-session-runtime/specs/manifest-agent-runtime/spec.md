## ADDED Requirements

### Requirement: Session spawn SHALL select an explicit agent runtime

Every production session spawn SHALL select an enabled agent profile/manifest or the explicit
shell kind. Unknown, invalid, disabled, or unavailable agents SHALL fail closed.

#### Scenario: Enabled manifest agent is selected

- **WHEN** a spawn request selects an enabled profile whose runtime maps to a bundled manifest
- **THEN** the runtime SHALL load that manifest and continue readiness evaluation
- **AND** it SHALL preserve the selected profile/runtime/permission identifiers on the session

#### Scenario: Agent selection is invalid

- **WHEN** the profile is missing, disabled, maps to no strict manifest, or the requested agent is unknown
- **THEN** spawn SHALL fail with a stable agent/profile error
- **AND** it SHALL not start `/bin/sh` or any other fallback

#### Scenario: Explicit shell session

- **WHEN** the caller explicitly selects the shell kind
- **THEN** the runtime MAY launch the reviewed `/bin/sh -i` shell manifest
- **AND** that behavior SHALL not be used for an unavailable non-shell agent

### Requirement: Readiness SHALL resolve without executing the agent

Manifest readiness SHALL resolve exactly one absolute executable under a bounded deadline without
starting the agent itself.

#### Scenario: Binary resolves through the local login environment

- **WHEN** a manifest binary is installed through the user's normal executable resolution path
- **THEN** readiness SHALL return its absolute executable path
- **AND** the agent process SHALL not be started during readiness

#### Scenario: Binary is unavailable or ambiguous

- **WHEN** readiness cannot produce one existing executable regular file
- **THEN** spawn SHALL fail as unavailable
- **AND** no session, effective config, holder, or child SHALL remain

### Requirement: Effective agent configuration SHALL be immutable per session

Before holder launch, the runtime SHALL freeze the profile/runtime/LLM/permission references,
manifest identity/version, absolute executable, argv, sanitized environment, injection decisions,
resume semantics, cwd, parent, and geometry for the session.

#### Scenario: Profile changes after spawn

- **WHEN** an administrator changes the source profile or descriptor after a session is running
- **THEN** the running session SHALL continue with its frozen effective configuration
- **AND** resume SHALL not silently adopt the changed profile

#### Scenario: Effective configuration cannot be committed

- **WHEN** effective-config creation, session linkage, or sanitized launch-record persistence fails
- **THEN** the runtime SHALL not launch or publish a running session
- **AND** it SHALL roll back any uncommitted session/config facts

#### Scenario: Frozen launch record is missing on resume

- **WHEN** an exited session requires resume but its frozen launch record is absent or invalid
- **THEN** resume SHALL fail closed as not resumable/invalid configuration
- **AND** it SHALL not reconstruct argv from a mutable profile

### Requirement: Agent child environment SHALL be sanitized and deterministic

The child environment SHALL be built from a reviewed baseline plus manifest additions and explicit
managed inputs. It SHALL apply manifest scrub rules before launch.

#### Scenario: Sensitive parent environment is present

- **WHEN** the daemon parent environment contains provider keys, Authorization values, cookies, or agent-session variables
- **THEN** those values SHALL not be inherited by default into the holder child
- **AND** raw values SHALL not appear in holder argv, launch metadata, logs, events, or evidence

#### Scenario: Manifest declares safe environment

- **WHEN** the manifest declares non-secret environment values
- **THEN** the launch plan SHALL include those values after scrub/baseline construction
- **AND** a real fake-agent executable SHALL observe exactly the expected safe values

#### Scenario: Managed LLM proxy data is absent

- **WHEN** no scoped Homie proxy configuration has been issued by its owning component
- **THEN** T-102 SHALL not inject a real provider credential or invent a production key

### Requirement: Manifest SHALL own argv, injection, authority, and resume semantics

Agent-specific conditionals SHALL be derived from the manifest catalog rather than duplicated in
runtime/app/CLI callers.

#### Scenario: Fresh manifest spawn

- **WHEN** a selected manifest declares spawn args and injection flags
- **THEN** the final argv SHALL preserve manifest argument order and reviewed injection
- **AND** the holder SHALL execute the resulting vector directly

#### Scenario: Manifest status authority is selected

- **WHEN** the session becomes live
- **THEN** its reducer SHALL be initialized with the frozen manifest status authority
- **AND** runtime status reads SHALL not force every agent to `ScreenPrimary`

#### Scenario: Manifest declares ID-based resume

- **WHEN** the manifest resume style requires an agent session ID and the ID is durably known
- **THEN** resume SHALL build the manifest-defined argv with that ID
- **AND** it SHALL use the same injection rules that are valid for resume

#### Scenario: Required resume ID is unavailable

- **WHEN** an ID-based resume is requested but no verified agent session ID exists
- **THEN** resume SHALL fail as not resumable
- **AND** it SHALL not substitute latest or a shell command

### Requirement: Spawn SHALL commit only after real holder readiness

The runtime SHALL not publish a session as running until the real holder accepts the structured
plan and reports a live child.

#### Scenario: Holder becomes ready

- **WHEN** holder launch succeeds and stat verifies a live child before the readiness deadline
- **THEN** the runtime SHALL register the live session and reducer
- **AND** it SHALL commit the running projection before publishing spawned/status events

#### Scenario: Holder launch partially fails

- **WHEN** the holder starts but the child exits or readiness times out before commit
- **THEN** the runtime SHALL terminate and reap the fixture/session holder tree
- **AND** it SHALL remove uncommitted runtime/storage/config state
- **AND** it SHALL return a stable launch error

### Requirement: Tests SHALL inject catalogs without production configuration

Tests SHALL provide deterministic fake manifests and real fake executables only through Rust
constructors/fixtures. Production SHALL have no fake manifest flag, test mode, or environment
override.

#### Scenario: Manifest process E2E

- **WHEN** an integration fixture supplies a fake manifest through an internal test constructor
- **THEN** the production spawn path SHALL start its real executable under a real holder and PTY
- **AND** argv/env/output assertions SHALL be based on process evidence

#### Scenario: Production process starts

- **WHEN** the packaged runtime daemon starts normally
- **THEN** it SHALL load only the fixed bundled descriptor source
- **AND** no environment variable SHALL redirect it to test manifests or binaries
