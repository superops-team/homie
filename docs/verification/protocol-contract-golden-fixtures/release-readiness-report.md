# Protocol Contract Golden Fixtures Release Readiness Report

## 1. Conclusion

`protocol-contract-golden-fixtures` first slice is ready to land.

## 2. Delivered

- Shared fixture directory: `protocol-fixtures/control-message/`.
- Valid roundtrip fixture cases for request, event, success response, null success, error response and discrimination order.
- Invalid fixture cases for malformed envelopes.
- Rust focused tests in `homie/crates/homie-proto/tests/control_roundtrip.rs`.
- Swift focused tests in `Tests/HomieProtocolTests/WireTests.swift`.
- Local `scripts/check.sh` focused fixture gates.
- OpenSpec plan/tasks/alignment and verification evidence.

## 3. Verification

| Gate | Result | Evidence |
|---|---|---|
| Spec review | pass | `spec-review-report.md` |
| OpenSpec alignment | pass | `fc-02-openspec-alignment.log` |
| Fixture safety | pass | `fc-03-fixture-contract.log` |
| Rust focused tests | pass | `fc-04-rust-fixtures.log` |
| Swift focused tests | pass | `fc-05-swift-fixtures.log` |
| Local gate | pass | `fc-06-local-gate.log` |
| Static gates | pass | `fc-07-static-gates.log` |
| Code review round 1 | pass | `code-review-round-1.md` |
| Code review round 2 | pass | `code-review-round-2.md` |

## 4. Not Run

- Full `scripts/check.sh` was not run end-to-end in this slice. The focused Rust/Swift fixture checks, script syntax, rustfmt and diff whitespace gates passed, and the full check script now includes the focused fixture checks.

## 5. Residual Risk

- Future protocol method payload schemas still need their own typed tests. This slice only guards the shared control-message envelope.
