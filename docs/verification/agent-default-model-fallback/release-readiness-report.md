# Release Readiness Report — agent-default-model-fallback

- Beads: `homie-qzv`
- change_id: `agent-default-model-fallback`
- 日期: 2026-08-25

## 1. 交付内容

| 组件 | 路径 | 状态 |
|------|------|------|
| Gateway config normalize | `homie/crates/homie-gateway/src/config.rs` | 完成 |
| Gateway model route fallback | `homie/crates/homie-gateway/src/routes.rs` | 完成 |
| Gateway integration regression | `homie/crates/homie-gateway/tests/gateway.rs` | 完成 |
| Engine injection regression | `homie/crates/homie-engine/src/inject.rs` | 完成 |
| Swift CLI config defaults | `Sources/homie-cli/HomieConfigStore.swift` | 完成 |
| Swift CLI model override write semantics | `Sources/homie-cli/ConfigCommand.swift` | 完成 |
| Swift CLI tests | `Tests/HomieCLITests/ConfigOpsTests.swift` | 已补测试，当前环境无法运行 |
| PRD/OpenSpec/spec updates | `prd-spec/`、`openspec/changes/`、`specs/` | 完成 |
| Verification evidence | `docs/verification/agent-default-model-fallback/` | 完成 |

## 2. 最终门禁结果

| 门禁 | 命令 | 结果 |
|------|------|------|
| 文档链路 | `test -s ... && rg -n ...` | 通过 |
| Rust 格式 | `cargo fmt --manifest-path homie/Cargo.toml --all --check` | 通过 |
| Gateway 全测 | `cargo test --manifest-path homie/Cargo.toml -p homie-gateway --offline` | 33 lib + 11 integration + 0 doc 全部通过 |
| Engine 注入回归 | `cargo test --manifest-path homie/Cargo.toml -p homie-engine --lib --offline inject` | 12 passed，0 failed |
| Gateway clippy | `cargo clippy --manifest-path homie/Cargo.toml -p homie-gateway --all-targets --offline` | 通过 |
| Rust workspace check | `cargo check --manifest-path homie/Cargo.toml --workspace --offline` | 通过 |
| Rust workspace test | `cargo test --manifest-path homie/Cargo.toml --workspace --offline` | 通过 |
| Swift build | `swift build` | 通过 |
| Swift tests | `swift test --filter ConfigOpsTests` 与 `swift test --enable-swift-testing --filter ConfigOpsTests` | 未通过：当前 CLT 环境 `no such module 'Testing'` |
| CLI 黑盒行为 | `HOMIE_CONFIG=<tmp> .build/debug/homie config set ...` | 通过：非空写入，空白删除，文件 0600 |
| Diff whitespace | `git diff --check` | 通过 |

## 3. RED / Mutation 证据

- RED：实现前新增的三条 Rust 测试均失败，分别证明配置加载、路由层、集成 usage 会被空模型污染。
- Manual mutation：临时移除 `routes.rs::apply_model_route` 的 `target.is_empty()` 保护后，
  `apply_model_route_passes_through_blank_targets` 按预期失败；恢复后最终门禁重新通过。

## 4. 验收标准核对

1. ✅ `models.codex` 缺失、空字符串、空白字符串均不会覆盖请求体模型。
2. ✅ `homie config set` 首次创建配置时不生成空 `models.codex`。
3. ✅ 用户显式设置非空 `--model-codex` 时，模型路由能力保持不变。
4. ✅ New Agent 在用户未配置 Homie 模型时继续使用 agent 自身默认配置；engine 注入测试确认不设置 `model`。
5. ⚠️ Swift 单元测试受本机 `Testing` 模块不可用阻塞；Swift build 与 CLI 黑盒行为验证通过。

## 5. 残余风险

- 当前未启动真实 GPUI app。变更不涉及 UI 渲染，核心风险由 gateway 转发测试、engine 注入测试和 CLI 行为测试覆盖。
- 若未来开启更多 OpenAI-compatible agent 的 gateway opt-in，需要为新 route key 复用同一空白模型语义。

## 6. 结论

本变更已满足主要验收标准，可关闭 Beads `homie-qzv`。Swift 测试环境问题需要在独立工具链维护任务中处理。
