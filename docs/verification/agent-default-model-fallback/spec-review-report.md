# Spec Review Report — agent-default-model-fallback

- Beads: `homie-qzv`
- change_id: `agent-default-model-fallback`
- 日期: 2026-08-25

## 1. 范围

本报告评审“用户未配置模型时 New Agent 不应被 Homie 模型路由阻塞”的 bugfix 设计，覆盖：

- `homie.local.json` 中 `models` 映射的空白值语义；
- Rust gateway 转发前模型改写；
- Swift CLI 默认配置与 `config set --model-codex` 行为；
- 既有 agent spawn/injection 边界。

## 2. 结论

采纳最小修复方案：空白模型值统一视为未配置。该方案直接修复历史空配置与 CLI 默认空占位造成的
模型覆盖问题，同时保留非空模型路由能力。

## 3. 评审发现

| 级别 | 领域 | 发现 | 决策 |
|------|------|------|------|
| P0 | 运行时语义 | `models.codex = ""` 会被当成有效 override，转发请求被改写成空 model | 必须修复：空白值不参与路由 |
| P1 | 配置写入 | `HomieConfigStore.empty` 默认生成空 `models.codex`，会制造历史兼容风险 | 改为空字典；空白 set 删除 override |
| P1 | 历史配置 | 只修 CLI 无法处理用户已有空配置 | Rust 加载和路由层都做防御 |
| P2 | scope | 本问题不要求开启 `codexGateway` 或新增 UI | 明确非目标，避免扩大变更 |

## 4. 风险模型

本变更紧邻 LLM gateway 与 agent 启动路径，按 Tier 3 记录风险，但实现保持窄面：

| 风险 | 验证 |
|------|------|
| 空白模型覆盖 agent 默认模型 | Rust 单测 + integration usage 断言 |
| 历史空配置仍污染运行时 | `GatewayConfig::from_file` normalize 单测 |
| CLI 再次写出空占位 | Swift CLI 单测 |
| 非空模型 override 回归 | 既有 gateway routing 测试保留 |
| 凭证泄露 | 无新增 secret 输出；diff/code review 核对 |

## 5. 与现有规范一致性

- `specs/llm-gateway.md` §7 已更新：只有非空模型映射才覆盖，空白等同未配置。
- `specs/homie-cli-config-ops.md` §3/§4.2 已更新：CLI 默认不写空模型占位，空白值删除 override。
- `homie-engine::inject` 仍不注入 `model`；模型覆盖只由网关完成。

## 6. 结论

规格明确、风险可控，可进入 RED → GREEN → REFACTOR 实现。
