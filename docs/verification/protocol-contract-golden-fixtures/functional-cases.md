# Swift/Rust 协议契约 Golden Fixtures 功能验证 Case

## 1. 验证目标

本验证 Case 面向 `protocol-contract-golden-fixtures` 首阶段，目标是证明：

- Swift/Rust control envelope 使用同一批共享 fixture；
- fixture 覆盖 request、event、ok response、null ok、error response、params 缺省/null、invalid cases；
- fixture 不包含真实 prompt、Authorization、cookie、provider token、私有路径或用户会话内容；
- 双端 focused tests 通过；
- 本地 contributor gate 能运行该漂移检查。

## FC-01: PRD/spec 和 review 风险已收敛

```bash
test -s docs/verification/protocol-contract-golden-fixtures/spec-review-report.md
rg -n "首阶段关闭口径|fixture 不包含真实 prompt|不引入 schema/codegen|OpenSpec alignment" \
  prd-spec/refactors/protocol-contract-golden-fixtures/2026-08-13-protocol-contract-golden-fixtures-design.md \
  docs/verification/protocol-contract-golden-fixtures/spec-review-report.md
```

通过标准：命中首阶段边界、安全规则和 OpenSpec 对齐要求。

证据路径：`docs/verification/protocol-contract-golden-fixtures/fc-01-spec-review.log`

## FC-02: OpenSpec 三件套完整并覆盖 Case

```bash
test -s openspec/changes/protocol-contract-golden-fixtures/plan.md
test -s openspec/changes/protocol-contract-golden-fixtures/tasks.md
test -s openspec/changes/protocol-contract-golden-fixtures/alignment-report.md
rg -n "FC-01|FC-02|FC-03|FC-04|FC-05|FC-06|FC-07" \
  openspec/changes/protocol-contract-golden-fixtures/tasks.md \
  openspec/changes/protocol-contract-golden-fixtures/alignment-report.md
```

通过标准：三件套存在，并覆盖 FC-01 至 FC-07。

证据路径：`docs/verification/protocol-contract-golden-fixtures/fc-02-openspec-alignment.log`

## FC-03: 共享 fixture 目录和安全扫描

```bash
test -s protocol-fixtures/control-message/README.md
test -s protocol-fixtures/control-message/roundtrip-cases.json
test -s protocol-fixtures/control-message/invalid-cases.json
rg -n "request-with-params|event-without-params|response-ok-null|response-error" \
  protocol-fixtures/control-message/roundtrip-cases.json
if rg -n "Authorization|cookie|token|/Users/|prompt|private_key|provider" \
  protocol-fixtures/control-message/*.json; then
  exit 1
fi
```

通过标准：fixture 文件存在、覆盖核心 case，敏感词扫描无命中。

证据路径：`docs/verification/protocol-contract-golden-fixtures/fc-03-fixture-contract.log`

## FC-04: Rust 读取共享 fixture 并验证 envelope

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-proto shared_control_fixture -- --nocapture
```

通过标准：Rust focused tests 通过，测试读取 `protocol-fixtures/control-message/*`。

证据路径：`docs/verification/protocol-contract-golden-fixtures/fc-04-rust-fixtures.log`

## FC-05: Swift 读取共享 fixture 并验证 envelope

```bash
swift test --package-path . --filter sharedControlFixture
```

通过标准：Swift focused tests 通过，测试读取 `protocol-fixtures/control-message/*`。

证据路径：`docs/verification/protocol-contract-golden-fixtures/fc-05-swift-fixtures.log`

## FC-06: 本地 gate 包含共享 fixture 漂移检查

```bash
rg -n "sharedControlFixture|shared_control_fixture" scripts/check.sh
bash -n scripts/check.sh
```

通过标准：`scripts/check.sh` 包含 Swift/Rust fixture focused checks，脚本语法通过。

证据路径：`docs/verification/protocol-contract-golden-fixtures/fc-06-local-gate.log`

## FC-07: 静态门禁

```bash
bash -n scripts/*.sh homie/scripts/*.sh
cargo fmt --manifest-path homie/Cargo.toml --all -- --check
git diff --check
git diff --name-only -- protocol-fixtures Tests/HomieProtocolTests homie/crates/homie-proto/tests scripts/check.sh
git status --short -- protocol-fixtures Tests/HomieProtocolTests homie/crates/homie-proto/tests scripts/check.sh
```

通过标准：脚本语法、Rust 格式和 diff whitespace 通过；范围守卫只显示预期测试/fixture/gate 文件。

证据路径：`docs/verification/protocol-contract-golden-fixtures/fc-07-static-gates.log`
