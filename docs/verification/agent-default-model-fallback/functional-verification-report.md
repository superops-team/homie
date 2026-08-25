# Functional Verification Report — agent-default-model-fallback

- Beads: `homie-qzv`
- change_id: `agent-default-model-fallback`
- 日期: 2026-08-25

## 1. RED 证据

新增回归测试后、实现前，以下测试按预期失败：

| 测试 | 失败点 |
|------|--------|
| `blank_model_overrides_are_ignored_at_config_load` | `GatewayConfig` 仍保留 `models.codex = ""` |
| `apply_model_route_passes_through_blank_targets` | 路由层把请求体 `model` 改写为 `""` |
| `blank_configured_model_passes_agent_model_through` | usage 记录的 model 为 `""` 而不是 agent 原始模型 |

## 2. GREEN 结果

| Case | 命令/证据 | 结果 |
|------|-----------|------|
| FC-2/3/4 | `cargo test --manifest-path homie/Cargo.toml -p homie-gateway --offline` | 33 lib + 11 integration + 0 doc 全部通过 |
| FC-5 | `swift build` | 通过，CLI 源码编译成功 |
| FC-5 | `fc-05-cli-config-behavior.log` | 非空 `--model-codex` trim 后写入；空白 `--model-codex` 删除 override；配置文件 0600 |
| FC-6 | `cargo test --manifest-path homie/Cargo.toml -p homie-engine --lib --offline inject` | 12 passed，验证注入路径不额外设置 model |
| FC-6 | `cargo clippy --manifest-path homie/Cargo.toml -p homie-gateway --all-targets --offline` | 通过 |
| FC-6 | `cargo check --manifest-path homie/Cargo.toml --workspace --offline` | 通过 |
| FC-6 | `cargo test --manifest-path homie/Cargo.toml --workspace --offline` | 通过 |
| FC-6 | `cargo fmt --manifest-path homie/Cargo.toml --all --check` | 通过 |
| FC-6 | `git diff --check` | 通过 |

## 3. Swift 测试状态

`swift test --filter ConfigOpsTests` 与 `swift test --enable-swift-testing --filter ConfigOpsTests`
均在测试 target 编译前失败：

```text
error: no such module 'Testing'
```

本机工具链信息：

```text
swift-driver version: 1.148.6 Apple Swift version 6.3.2
Target: arm64-apple-macosx26.0
xcode-select: /Library/Developer/CommandLineTools
```

因此 Swift 测试层未通过，原因是当前 Command Line Tools 环境无法解析 Swift Testing 模块；本次 Swift
改动用 `swift build` 和真实 CLI 临时配置黑盒验证覆盖。

## 4. Manual Mutation

临时删除 `routes.rs::apply_model_route` 的 `target.is_empty()` 保护后运行：

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-gateway --offline apply_model_route_passes_through_blank_targets
```

结果按预期失败：实际 body 被改写为 `{"model":""}`，证明新增测试能捕获空模型覆盖回归。随后已恢复实现并重新跑最终门禁。

## 5. 验收结论

- 空白 `models.codex` 不再覆盖 agent 请求体模型。
- 历史空配置在 Rust gateway 加载与路由层均被防御。
- CLI 默认配置不再制造空 `models.codex` 占位。
- 显式非空模型路由保留。
