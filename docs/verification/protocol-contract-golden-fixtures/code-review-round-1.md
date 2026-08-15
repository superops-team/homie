# Protocol Contract Golden Fixtures Code Review Round 1

## 1. Scope

Reviewed files:

- `protocol-fixtures/control-message/*`
- `homie/crates/homie-proto/tests/control_roundtrip.rs`
- `Tests/HomieProtocolTests/WireTests.swift`
- `scripts/check.sh`
- `openspec/changes/protocol-contract-golden-fixtures/*`
- `docs/verification/protocol-contract-golden-fixtures/*`

## 2. Findings

| Severity | Finding | Result |
|---|---|---|
| P0 | Fixtures may contain sensitive real data | pass: fixture JSON uses synthetic ids and payloads; sensitive-term scan passed |
| P0 | Swift and Rust might not read the same fixture source | pass: both tests read `protocol-fixtures/control-message/*` from repo root |
| P1 | Fixture may silently change protocol behavior | pass: tests only assert current envelope discrimination and canonical encoding; no production code changed |
| P1 | Invalid cases may be too weak | pass: invalid fixtures cover non-object, missing request id, wrong method type, missing event seq, missing error id, malformed error |
| P1 | Local gate might drift from focused tests | pass: `scripts/check.sh` calls both `shared_control_fixture` and `sharedControlFixture` filters |
| P2 | Swift fixture path could be fragile | fixed during implementation: path now walks from `#filePath` to repo root before appending `protocol-fixtures` |

## 3. Verification Reviewed

| Evidence | Result |
|---|---|
| `fc-03-fixture-contract.log` | fixture files and sensitive scan passed |
| `fc-04-rust-fixtures.log` | Rust focused fixture tests passed |
| `fc-05-swift-fixtures.log` | Swift focused fixture tests passed |
| `fc-06-local-gate.log` | check script contains both focused gates |
| `fc-07-static-gates.log` | shell syntax, rustfmt, diff whitespace and scope guard passed |

## 4. Conclusion

No P0/P1 code issues remain after round 1.
