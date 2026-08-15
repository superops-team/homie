# Typed Agent Driver Capability Tasks

## T1: Spec and functional cases

- Deliverables:
  - `docs/verification/typed-agent-driver-capabilities/functional-cases.md`
  - `openspec/changes/typed-agent-driver-capabilities/*`
- Acceptance:
  - OpenSpec and functional cases keep the first slice read-only and fake-driver-only.
- Verification Cases: FC-01, FC-02

## T2: Add capability wire DTOs

- Deliverables:
  - `homie/crates/homie-proto/src/methods.rs`
  - `Sources/HomieProtocol/Methods.swift`
- Acceptance:
  - Rust and Swift define the same `session.capabilities` method name;
  - capability fields use camelCase spelling;
  - default capabilities serialize as all false.
- Verification Cases: FC-05

## T3: Add Engine driver abstraction and fake driver

- Deliverables:
  - `homie/crates/homie-engine/src/driver.rs`
  - `homie/crates/homie-engine/src/lib.rs`
- Acceptance:
  - default driver operations return stable unsupported errors;
  - fake driver declares steer/cancel/model/native cursor capabilities;
  - tests prove sensitive prompt-like input is not stored verbatim.
- Verification Cases: FC-03

## T4: Add read-only session.capabilities control method

- Deliverables:
  - `homie/crates/homie-engine/src/control.rs`
- Acceptance:
  - missing session returns not_found;
  - real manifest/shell/generic sessions return all false capabilities;
  - fake-driver session returns fake capabilities in tests;
  - method is read-only and does not mutate status or records.
- Verification Cases: FC-04

## T5: Final verification and review

- Deliverables:
  - verification logs and reports under `docs/verification/typed-agent-driver-capabilities/`
- Acceptance:
  - FC-01 through FC-06 pass;
  - code review reports exist;
  - release readiness report exists.
- Verification Cases: FC-06
