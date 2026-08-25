# Code Review Round 1 — agent-default-model-fallback

- Beads: `homie-qzv`
- change_id: `agent-default-model-fallback`
- 日期: 2026-08-25

## 1. 显性问题检查

| 检查项 | 结论 | 依据 |
|--------|------|------|
| Rust 编译 | 通过 | `cargo check --manifest-path homie/Cargo.toml --workspace --offline` 通过 |
| Rust 格式 | 通过 | `cargo fmt --manifest-path homie/Cargo.toml --all --check` 通过 |
| Gateway 测试 | 通过 | 33 lib + 11 integration + 0 doc 全部通过 |
| Gateway lint | 通过 | `cargo clippy --manifest-path homie/Cargo.toml -p homie-gateway --all-targets --offline` 通过 |
| Swift 编译 | 通过 | `swift build` 通过 |
| Swift tests | 环境阻塞 | 测试 target 编译失败：`no such module 'Testing'` |
| Diff whitespace | 通过 | `git diff --check` 通过 |

## 2. 代码级检查

| 文件 | 检查结果 |
|------|----------|
| `homie-gateway/src/config.rs` | `normalize_models` 在配置加载入口统一处理空白值，避免历史配置污染运行时状态 |
| `homie-gateway/src/routes.rs` | 路由层再次检查空白 target，防御非 `GatewayConfig` 构造的 `AppState` |
| `homie-gateway/tests/gateway.rs` | 集成测试通过 usage 记录确认请求没有被空模型改写 |
| `HomieConfigStore.swift` | 默认配置改为空 `models`；缺失 `models` 可解码为空字典 |
| `ConfigCommand.swift` | `--model-codex` 非空写入，空白删除 override |

## 3. 结论

未发现 P0/P1 显性问题。Swift 测试因当前命令行工具链缺 Swift Testing 模块未能运行，已用
`swift build` 和 CLI 黑盒行为测试补充验证。
