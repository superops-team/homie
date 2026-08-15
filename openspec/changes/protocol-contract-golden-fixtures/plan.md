# Protocol Contract Golden Fixtures Plan

## 1. Scope

First slice for `protocol-contract-golden-fixtures`.

## 2. In Scope

- Add shared control-message fixtures under `protocol-fixtures/control-message/`.
- Add fixture README documenting envelope discrimination and safety rules.
- Extend Rust `homie-proto` tests to read the shared fixture directory.
- Extend Swift `HomieProtocolTests` to read the shared fixture directory.
- Add focused fixture checks to local `scripts/check.sh`.
- Record evidence under `docs/verification/protocol-contract-golden-fixtures/`.

## 3. Out Of Scope

- New wire methods.
- NDJSON format change.
- Schema/codegen.
- Protobuf/Cap'n Proto or binary protocol migration.
- Runtime compatibility layer.
- Packaging fixture files into app bundles.

## 4. Durable Spec Impact

This first slice only codifies current `ControlMessage` wire envelope behavior:

- request: `method` present and non-null;
- event: `event` present and non-null;
- failure response: `err` present and non-null;
- success response: otherwise `ok`, defaulting to null when absent;
- event params default to null when absent.

No durable runtime behavior changes are introduced, so no `specs/` update is required in this slice.

## 5. Design

Use one language-neutral JSON fixture file for valid roundtrip cases:

```text
protocol-fixtures/control-message/roundtrip-cases.json
```

Each case contains:

- `name`
- `wire`
- `canonical`
- `kind`

Use a separate JSON fixture file for invalid cases:

```text
protocol-fixtures/control-message/invalid-cases.json
```

Each invalid case contains:

- `name`
- `wire`

Both Swift and Rust tests must read these files directly from the repo root.

## 6. Evidence

- Spec review: `docs/verification/protocol-contract-golden-fixtures/spec-review-report.md`
- Functional cases: `docs/verification/protocol-contract-golden-fixtures/functional-cases.md`
- Functional verification: `docs/verification/protocol-contract-golden-fixtures/functional-verification-report.md`
- Code review: `docs/verification/protocol-contract-golden-fixtures/code-review-round-1.md`, `code-review-round-2.md`
- Release readiness: `docs/verification/protocol-contract-golden-fixtures/release-readiness-report.md`

## 7. Risks

| Risk | Control |
|---|---|
| Fixture locks in wrong behavior | This slice codifies only existing Swift/Rust-matched envelope rules |
| Fixture leaks sensitive data | Use synthetic payloads and run sensitive-term scan |
| CI grows too much | Add focused tests to `scripts/check.sh`; broader CI already runs Swift/Rust suites |
| Runtime package gets test data | Fixture directory is test-only and not copied by package/dev scripts |
