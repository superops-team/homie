# Typed Agent Driver Capability Code Review Round 1

## 1. Scope

Reviewed files:

- `homie/crates/homie-engine/src/driver.rs`
- `homie/crates/homie-engine/src/control.rs`
- `homie/crates/homie-engine/src/lib.rs`
- `homie/crates/homie-proto/src/methods.rs`
- `homie/crates/homie-client/src/client.rs`
- `Sources/HomieProtocol/Methods.swift`
- `Tests/HomieProtocolTests/WireTests.swift`
- `homie/crates/homie-proto/tests/control_roundtrip.rs`

## 2. Findings

| Severity | Finding | Result |
|---|---|---|
| P0 | Typed driver could replace PTY/session authority | pass: implementation only adds read-only `session.capabilities`; no lifecycle/status code path changes |
| P0 | Real provider behavior could leak into first slice | pass: only `UnsupportedDriver` and `FakeDriver` exist; no Codex/Claude/OpenCode adapter |
| P1 | Capability query could mutate session records | pass: Engine test compares record before/after query |
| P1 | Fake driver could retain sensitive prompt text | pass: fake stores only steered text length in tests |
| P1 | Wire spelling could drift across Swift/Rust | pass: Swift and Rust focused tests cover camelCase fields and `session.capabilities` method name |
| P2 | Static gate failed after import formatting | fixed: rustfmt import order corrected and FC-06 re-run |

## 3. Verification Reviewed

| Evidence | Result |
|---|---|
| `fc-03-driver-contract.log` | driver contract tests passed |
| `fc-04-session-capabilities.log` | read-only capability query tests passed |
| `fc-05-wire-compatibility.log` | Swift/Rust wire tests passed |
| `fc-06-static-gates.log` | shell syntax, rustfmt, diff whitespace passed |

## 4. Conclusion

Round 1 found no remaining P0/P1 issue after the rustfmt cleanup.
