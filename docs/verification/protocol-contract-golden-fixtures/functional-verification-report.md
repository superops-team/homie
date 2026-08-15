# Protocol Contract Golden Fixtures Functional Verification Report

## 1. 结论

`protocol-contract-golden-fixtures` 首阶段功能验证通过。

已完成：

- 新增共享 fixture 目录 `protocol-fixtures/control-message/`。
- Rust `homie-proto` focused tests 读取共享 fixtures。
- Swift `HomieProtocolTests` focused tests 读取共享 fixtures。
- `scripts/check.sh` 增加共享 fixture focused gate。
- 不改变 NDJSON wire format，不新增 wire method，不引入 schema/codegen。

## 2. Case 执行结果

| Case | 状态 | 证据 |
|---|---|---|
| FC-01 PRD/spec 和 review 风险已收敛 | pass | `fc-01-spec-review.log` |
| FC-02 OpenSpec 三件套完整并覆盖 Case | pass | `fc-02-openspec-alignment.log` |
| FC-03 共享 fixture 目录和安全扫描 | pass | `fc-03-fixture-contract.log` |
| FC-04 Rust 读取共享 fixture 并验证 envelope | pass | `fc-04-rust-fixtures.log` |
| FC-05 Swift 读取共享 fixture 并验证 envelope | pass | `fc-05-swift-fixtures.log` |
| FC-06 本地 gate 包含共享 fixture 漂移检查 | pass | `fc-06-local-gate.log` |
| FC-07 静态门禁 | pass | `fc-07-static-gates.log` |

## 3. 关键证据

- Rust focused tests: `shared_control_fixture_invalid_cases_fail_to_decode` and `shared_control_fixture_round_trips_to_canonical_json` passed.
- Swift focused tests: `sharedControlFixtureInvalidCasesFailToDecode` and `sharedControlFixtureRoundTripsToCanonicalJSON` passed.
- Fixture sensitive-term scan passed against `protocol-fixtures/control-message/*.json`.
- `cargo fmt --manifest-path homie/Cargo.toml --all -- --check` passed.
- `git diff --check` passed.

## 4. 修复记录

- Swift test initially looked under `Tests/protocol-fixtures/...`; fixed fixture root resolution to walk to repo root.
- Rust test needed rustfmt line wrapping for `MAX_CONTROL_LINE_BYTES`; fixed and re-ran format check.

## 5. Residual Risk

- Full `scripts/check.sh` was not run end-to-end in this slice; focused Swift/Rust fixture checks and static gates were run. The check script now includes these focused gates for broader contributor runs.
