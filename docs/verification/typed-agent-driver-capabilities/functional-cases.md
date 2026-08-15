# Typed Agent Driver Capability 功能验证 Case

## 1. 验证目标

本验证 Case 面向 `typed-agent-driver-capabilities` 首阶段，目标是证明：

- Homie 有明确 typed driver capability 数据模型；
- 默认 driver 操作返回稳定 unsupported 错误；
- fake driver contract tests 覆盖 capability、steer、cancel、model discovery；
- Engine 通过只读 `session.capabilities` 查询暴露能力；
- manifest-only / shell / generic agent 不暴露 typed capabilities；
- 本阶段不接真实 provider、不改 MCP、不新增 steer/cancel wire method。

## FC-01: PRD/spec 和 review 风险已收敛

```bash
test -s docs/verification/typed-agent-driver-capabilities/spec-review-report.md
rg -n "首阶段关闭口径|fake driver|不接真实 provider|不改 MCP|session authority" \
  prd-spec/features/typed-agent-driver-capabilities/2026-08-13-typed-agent-driver-capabilities-design.md \
  docs/verification/typed-agent-driver-capabilities/spec-review-report.md
```

通过标准：命中首阶段 fake driver、非真实 provider、非 MCP 扩面和 authority 边界。

证据路径：`docs/verification/typed-agent-driver-capabilities/fc-01-spec-review.log`

## FC-02: OpenSpec 三件套完整并覆盖 Case

```bash
test -s openspec/changes/typed-agent-driver-capabilities/plan.md
test -s openspec/changes/typed-agent-driver-capabilities/tasks.md
test -s openspec/changes/typed-agent-driver-capabilities/alignment-report.md
rg -n "FC-01|FC-02|FC-03|FC-04|FC-05|FC-06" \
  openspec/changes/typed-agent-driver-capabilities/tasks.md \
  openspec/changes/typed-agent-driver-capabilities/alignment-report.md
```

通过标准：三件套存在，并覆盖 FC-01 至 FC-06。

证据路径：`docs/verification/typed-agent-driver-capabilities/fc-02-openspec-alignment.log`

## FC-03: capability DTO 和 fake driver contract

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-engine driver -- --nocapture
```

通过标准：

- default capabilities 全部为 false；
- unsupported 操作返回稳定 `unsupported` 错误；
- fake driver 能声明 steer/cancel/model/native cursor capabilities；
- fake driver 不记录 secret、Authorization、cookie 或完整敏感 prompt payload。

证据路径：`docs/verification/typed-agent-driver-capabilities/fc-03-driver-contract.log`

## FC-04: `session.capabilities` 查询路径

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-engine session_capabilities -- --nocapture
```

通过标准：

- 对不存在 session 返回 not_found；
- shell/generic 或 manifest-only session 返回全 false capabilities；
- fake-driver session 返回 fake capabilities；
- 查询不改变 session status、record 或 persistence。

证据路径：`docs/verification/typed-agent-driver-capabilities/fc-04-session-capabilities.log`

## FC-05: wire compatibility 和 Swift/Rust method vocabulary

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-proto session_capabilities -- --nocapture
swift test --package-path . --filter sessionCapabilities
```

通过标准：Rust DTO 和 Swift `HomieProtocol` 方法/DTO 使用同一 wire spelling，且缺省 capability 为 false。

证据路径：`docs/verification/typed-agent-driver-capabilities/fc-05-wire-compatibility.log`

## FC-06: 静态门禁和范围守卫

```bash
bash -n scripts/*.sh homie/scripts/*.sh
cargo fmt --manifest-path homie/Cargo.toml --all -- --check
git diff --check
git diff --name-only -- homie/crates/homie-engine homie/crates/homie-proto Sources/HomieProtocol Tests/HomieProtocolTests
```

通过标准：脚本语法、Rust 格式和 diff whitespace 通过；范围守卫只显示预期 Engine/Proto/Swift protocol/test 文件。

证据路径：`docs/verification/typed-agent-driver-capabilities/fc-06-static-gates.log`
