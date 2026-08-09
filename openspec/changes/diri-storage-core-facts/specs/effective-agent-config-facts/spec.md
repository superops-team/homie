## ADDED Requirements

### Requirement: Immutable effective config snapshot
Each managed session SHALL bind to one versioned, deterministic, safe
`EffectiveAgentConfigSnapshot` that remains unchanged after freeze.

#### Scenario: Session configuration is frozen
- **WHEN** T-102 supplies a resolved agent profile, runtime descriptor, managed LLM route, permission profile, skills, MCP servers, and workspace scope
- **THEN** storage persists their safe snapshots, a deterministic config hash, references, and freeze timestamp

#### Scenario: Source profile changes later
- **WHEN** agent, runtime, LLM, permission, skill, or MCP profile data is edited after freeze
- **THEN** readback for the running session returns the original frozen values and hash

#### Scenario: A second freeze is attempted
- **WHEN** another effective config is bound to the same session
- **THEN** the repository returns a stable conflict and preserves the first immutable snapshot

### Requirement: Atomic session and config binding
Session creation/parent binding and effective-config freeze MUST commit with single-transaction
semantics.

#### Scenario: Freeze succeeds
- **WHEN** all referenced profile/config rows and the parent session are valid
- **THEN** the session and exactly one effective config are durably linked

#### Scenario: Freeze fails
- **WHEN** serialization, validation, hashing, a foreign key, or session binding fails
- **THEN** neither an unbound session nor an orphan effective-config row remains

### Requirement: Safe config readback
The repository and `session.effective_config` service SHALL return a safe snapshot without secret
material.

#### Scenario: Effective config is read by session id
- **WHEN** the session has a frozen config
- **THEN** the response includes version, safe runtime/LLM/permission snapshots, ids, config hash, and frozen time

#### Scenario: Effective config contains credential references
- **WHEN** the snapshot is serialized
- **THEN** it may include `virtualKeyId` and provider/profile ids but contains no provider raw key, virtual-key material, Authorization, cookie, or secret-bearing environment value

#### Scenario: Snapshot JSON is hostile
- **WHEN** a snapshot is oversized, malformed, or has an unknown version
- **THEN** freeze/readback fails closed with a stable safe error
