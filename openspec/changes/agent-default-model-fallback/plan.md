# OpenSpec Plan — agent-default-model-fallback

change_id: `agent-default-model-fallback` · Beads: `homie-qzv`

## 1. 目标

修复空模型配置导致 New Agent 启动后的网关请求被改写为 `model: ""` 的问题。用户没有配置
Homie 模型映射时，agent 必须使用自身默认模型；只有非空模型映射才参与网关路由。

## 2. 实施范围

```text
homie/crates/homie-gateway/
├── src/config.rs       # 加载时过滤空白 model override
├── src/routes.rs       # 转发时防御空白 model override
└── tests/gateway.rs    # 空白模型透传集成测试

Sources/homie-cli/
├── HomieConfigStore.swift # 默认空配置不带空 model override
└── ConfigCommand.swift    # --model-codex 空白值删除 override

Tests/HomieCLITests/
└── ConfigOpsTests.swift   # CLI 配置 schema/默认值回归
```

同步更新：

- `specs/llm-gateway.md`
- `specs/homie-cli-config-ops.md`
- `docs/verification/agent-default-model-fallback/`

## 3. 设计约束

- 不新增依赖。
- 不改 `codexGateway` manifest 开关。
- 不让 spawn-time injection 设置 `model`。
- 不改变 `upstream.baseUrl/apiKey` 缺失时 gateway listener 禁用的语义。
- 不输出真实 provider key 或虚拟 key。

## 4. 测试策略

本变更触碰 LLM gateway 模型路由与配置写入，按 Tier 3 处理，但实现面窄：

- RED：先补 `apply_model_route` / `GatewayConfig` / integration / Swift 默认配置测试，确认至少 Rust 空白路由用例失败。
- GREEN：实现空白过滤与 CLI 默认模型空映射。
- REFACTOR：保持 helper 小而纯，不引入新抽象。
- Manual mutation：临时移除空白检查或改回空占位，确认测试失败后恢复。
- Gates：`cargo fmt --check`、`cargo test -p homie-gateway --offline`、相关 Swift tests，最后 `git diff --check`。

## 5. 风险模型

| 风险 | 防护 |
|------|------|
| 空白模型覆盖 agent 默认模型 | Rust 单测 + gateway 集成测试 |
| 历史空配置继续生效 | `GatewayConfig::from_file` normalize 测试 |
| CLI 首次写入继续制造空占位 | Swift 单测 |
| 显式非空模型路由回归 | 既有改写测试保留 |
| 凭证泄露 | 不新增 key 输出；验证 grep/diff |

## 6. 出口标准

- PRD / specs / OpenSpec alignment 完整。
- 回归测试覆盖空白模型配置。
- 本次代码 diff 不覆盖既有未提交 listener 修复。
- Beads `homie-qzv` 关闭前证据文件完整。
