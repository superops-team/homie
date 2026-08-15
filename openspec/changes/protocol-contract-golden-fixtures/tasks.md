# Protocol Contract Golden Fixtures Tasks

## T1: Spec and functional cases

- Deliverables:
  - `docs/verification/protocol-contract-golden-fixtures/functional-cases.md`
  - `openspec/changes/protocol-contract-golden-fixtures/*`
- Acceptance:
  - OpenSpec and functional cases cover fixture safety, Swift, Rust, gate and static checks.
- Verification Cases: FC-01, FC-02

## T2: Add shared fixture contract

- Deliverables:
  - `protocol-fixtures/control-message/README.md`
  - `protocol-fixtures/control-message/roundtrip-cases.json`
  - `protocol-fixtures/control-message/invalid-cases.json`
- Acceptance:
  - fixtures cover request/event/ok/null/error and invalid envelope cases;
  - fixtures are synthetic and pass sensitive-term scan.
- Verification Cases: FC-03

## T3: Add Rust shared fixture tests

- Deliverables:
  - `homie/crates/homie-proto/tests/control_roundtrip.rs`
- Acceptance:
  - Rust tests read `protocol-fixtures/control-message/*`;
  - valid cases decode and re-encode to canonical JSON;
  - invalid cases fail to decode;
  - `WIRE_VERSION` and `MAX_CONTROL_LINE_BYTES` are asserted.
- Verification Cases: FC-04

## T4: Add Swift shared fixture tests

- Deliverables:
  - `Tests/HomieProtocolTests/WireTests.swift`
- Acceptance:
  - Swift tests read `protocol-fixtures/control-message/*`;
  - valid cases decode and re-encode to canonical JSON;
  - invalid cases fail to decode;
  - `WireVersion.current` and `NDJSONBuffer.maxLineBytes` are asserted.
- Verification Cases: FC-05

## T5: Add local gate

- Deliverables:
  - `scripts/check.sh`
- Acceptance:
  - check script runs focused Swift and Rust shared fixture tests before broader suites or as part of existing test stage;
  - shell syntax remains valid.
- Verification Cases: FC-06

## T6: Final verification and review

- Deliverables:
  - verification logs and reports under `docs/verification/protocol-contract-golden-fixtures/`
- Acceptance:
  - FC-01 through FC-07 pass;
  - code review reports exist;
  - release readiness report exists.
- Verification Cases: FC-07
