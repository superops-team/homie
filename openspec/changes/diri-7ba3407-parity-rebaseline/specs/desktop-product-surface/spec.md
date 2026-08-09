## ADDED Requirements

### Requirement: Service-backed desktop actions
Every mutable desktop action SHALL dispatch through a typed client/service command and update UI state from an authoritative response or event.

#### Scenario: User pins or archives a session
- **WHEN** the action succeeds
- **THEN** the durable session fact changes and the UI updates from the resulting projection

#### Scenario: Mutable action fails
- **WHEN** runtime or storage owner rejects the command
- **THEN** the UI presents a safe error and does not retain a false successful state

### Requirement: Complete Diri product surfaces
The desktop SHALL provide runtime-backed workbench, sidebar, terminal, inspector, navigation, history, settings, worktrees, notifications and update surfaces.

#### Scenario: User opens a surface
- **WHEN** a surface requires session, git, artifact, usage, remote or update data
- **THEN** it reads the owning service projection rather than a fixture or hard-coded copy

#### Scenario: Runtime is disconnected
- **WHEN** the client enters disconnected or reconnecting state
- **THEN** the shell exposes that state, disables unsafe live actions and supports recovery

### Requirement: Complete terminal interaction
The terminal SHALL render live runtime grids and support cursor, selection, copy/paste, find, keyboard encoding, resize and offset-based scrollback.

#### Scenario: User scrolls beyond the resident viewport
- **WHEN** older rows are required
- **THEN** the terminal requests bounded row/offset data instead of loading the complete output log

#### Scenario: Terminal behavior is verified
- **WHEN** terminal acceptance runs
- **THEN** it uses deterministic terminal fixtures or real PTY interaction and not source-code string matching

### Requirement: Visual and interaction parity evidence
Desktop completion MUST include structural tests, real app interactions and side-by-side Diri/Homie screenshots.

#### Scenario: UI module is proposed as complete
- **WHEN** only model/unit tests or a static screenshot exist
- **THEN** the module remains partial until real interaction and visual gates pass
