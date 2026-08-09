## ADDED Requirements

### Requirement: Immutable Diri baseline
The project SHALL use the embedded `diri/` repository at commit `7ba3407` as the only completion baseline for this parity program.

#### Scenario: Baseline is evaluated
- **WHEN** a parity requirement, task, test or release claim is created
- **THEN** it cites Diri commit `7ba3407` and a source entry in `docs/research/diri-7ba3407-capability-matrix.md`

#### Scenario: Upstream Diri changes
- **WHEN** the embedded Diri repository moves beyond `7ba3407`
- **THEN** the existing parity baseline remains unchanged until a separately approved rebaseline change is created

### Requirement: Truthful capability status
Every required capability SHALL use exactly one of `implemented`, `partial`, `missing` or `blocked`, and `implemented` SHALL require real code, production-path wiring, current verification and recorded evidence.

#### Scenario: DTO or static UI exists without integration
- **WHEN** Homie contains a protocol DTO, parser, static UI, source-text test or fixture but no complete production path
- **THEN** the capability remains `partial` or `missing`

#### Scenario: Current test contradicts implemented status
- **WHEN** a required verification test fails for a capability marked `implemented`
- **THEN** the capability is downgraded before dependent implementation planning continues

### Requirement: Typed evidence vocabulary
All new verification evidence MUST use only `pass`, `blocked`, `not_run`, `partial` or `fail`.

#### Scenario: A gate cannot run
- **WHEN** required hardware, credentials or environment are unavailable
- **THEN** the gate is recorded as `blocked` or `not_run` with a reason and is not counted as pass

#### Scenario: A scoped change passes
- **WHEN** one wave passes all of its own acceptance criteria while other parity capabilities remain incomplete
- **THEN** the wave may be `pass` and overall Diri parity remains `partial`

### Requirement: Final parity decision
The project MUST NOT claim Diri parity complete while any required matrix entry or final gate is incomplete.

#### Scenario: Final gate is evaluated
- **WHEN** Wave 5B evaluates the release
- **THEN** it verifies every required capability, workspace test, E2E, security, package, screenshot and performance gate before returning pass
