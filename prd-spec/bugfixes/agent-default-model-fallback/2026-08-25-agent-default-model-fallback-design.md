# 空模型配置不阻塞 agent 启动设计文档

## 1. 概述

### 1.1 问题

Homie 的 LLM 网关支持通过 `homie.local.json` 的 `models.codex` 覆盖 Codex 请求里的
`model`。但当前 Swift CLI 的默认空配置会生成：

```json
"models": { "codex": "" }
```

Rust 网关只检查 `models` 中是否存在 `codex` key，没有检查值是否为空白。历史配置或默认配置中
出现空字符串时，网关会把 agent 请求体里的默认 `model` 改写成空字符串，导致 OpenAI-compatible
agent 无法使用自己的默认模型完成启动后的请求。

用户未配置模型时，New Agent 不应该被 Homie 的空模型占位阻塞；应允许 agent 使用自身默认配置启动。

### 1.2 根因

根因是“未配置”和“配置为空字符串”没有被统一建模：

- `HomieConfigStore.empty` 把 `models.codex` 初始化为空字符串，形成看似存在的模型映射；
- `GatewayConfig::from_file` 原样保留空白模型值；
- `routes.rs::apply_model_route` 只按 key 是否存在判断是否改写，不校验目标模型是否非空。

因此一个空占位值会穿透到转发路径，覆盖 agent 自带的默认模型。

## 2. 用户场景

### 场景 1：未配置模型时使用 agent 自身默认模型

**Given** 用户只配置了上游 `baseUrl/apiKey`，没有设置 `models.codex`。
**When** 用户创建 Codex New Agent，Codex 经网关请求 `/v1/responses`，请求体包含自身默认
`model`。
**Then** 网关不改写该 `model`，请求继续转发，agent 不因 Homie 模型未配置而阻塞。

### 场景 2：历史空模型配置不覆盖请求

**Given** 历史 `homie.local.json` 中存在 `"models": { "codex": "" }` 或空白字符串。
**When** Codex 请求进入网关。
**Then** 空白值被视为未配置，网关透传请求体中的原始 `model`。

### 场景 3：显式非空模型仍然覆盖

**Given** 用户执行 `homie config set --model-codex gpt-5.2-codex`。
**When** Codex 请求进入网关。
**Then** 网关继续把请求体 `model` 改写为 `gpt-5.2-codex`，保持既有模型路由能力。

## 3. 功能需求

### FR-1：空白模型等同未配置

`models` map 中值为 `""` 或仅包含空白字符的条目，必须在配置加载或转发改写前被视为不存在。

### FR-2：New Agent 默认自配置启动

在没有非空 Homie 模型映射时，daemon spawn 和网关转发不得额外设置或覆盖 agent 的模型。agent
应使用自身 CLI/config/auth 中的默认模型。

### FR-3：CLI 不再写空模型占位

`HomieConfigStore.empty` 不应默认写入 `models.codex = ""`。用户没有传 `--model-codex` 时，
首次 `homie config set` 创建的配置应保持 `models` 为空映射。

### FR-4：保留显式模型路由

非空 `models.codex` 的行为保持不变：网关改写请求 model，并以改写后的 model 记录用量。

### FR-5：安全边界不变

本变更不新增凭证读取、虚拟 key 签发路径或日志输出。真实 provider key 和虚拟 key 仍只保留在
本地 ignored 配置与 SQLite 内。

## 4. 实现方案

### 4.1 Rust 网关

- `GatewayConfig::from_file` 对 `file.models` 做 normalize：trim 后为空的 key 移除，非空值写入
  trim 后的模型名。
- `apply_model_route` 在改写前检查 `target.trim().is_empty()`，空白则直接透传原 body。
- 增加单元测试：
  - 配置加载过滤空白 `models.codex`；
  - 空白模型路由透传，不改写为 `""`；
  - 非空模型路由保持改写。
- 增加集成测试：`models.codex = ""` 时 `/v1/responses` 的用量记录仍为请求原始 model。

### 4.2 Swift CLI

- `HomieConfigStore.empty.models` 改为空字典。
- `ConfigSet` 处理 `--model-codex` 时：
  - 非空白值写入 `models["codex"]`；
  - 空白值移除 `models["codex"]`。
- `config show/get models.codex` 对缺失值继续显示空字符串，保持 CLI 显示兼容。
- 增加 Swift 测试覆盖默认空配置不带空模型占位。

### 4.3 不改动项

- 不把 `codexGateway` 从 `false` 改为 `true`；
- 不新增模型选择 UI；
- 不改变上游凭证缺失时 gateway listener 禁用的语义；
- 不改变 Claude Code 原生凭证路径。

## 5. 边界情况

| 场景 | 处理方式 |
|------|---------|
| `models` 字段缺失 | 反序列化为空映射，透传 agent model |
| `models.codex` 是空字符串 | 视为未配置，透传 agent model |
| `models.codex` 只有空白字符 | 视为未配置，透传 agent model |
| `models.codex` 是非空字符串 | 改写请求 model |
| 请求体非 JSON 或无字符串 model | 沿用现有透传语义 |
| 用户传 `--model-codex ""` | 删除模型覆盖，恢复 agent 默认模型 |

## 6. 受影响文件

- `homie/crates/homie-gateway/src/config.rs`
- `homie/crates/homie-gateway/src/routes.rs`
- `homie/crates/homie-gateway/tests/gateway.rs`
- `Sources/homie-cli/HomieConfigStore.swift`
- `Sources/homie-cli/ConfigCommand.swift`
- `Tests/HomieCLITests/ConfigOpsTests.swift`
- `specs/llm-gateway.md`
- `specs/homie-cli-config-ops.md`

## 7. 验收标准

1. `models.codex` 缺失、空字符串、空白字符串均不会覆盖请求体模型。
2. `homie config set` 首次创建配置时不生成空 `models.codex`。
3. 用户显式设置非空 `--model-codex` 时，模型路由能力保持不变。
4. New Agent 在用户未配置 Homie 模型时继续使用 agent 自身默认配置。
5. Rust gateway 测试和 Swift CLI 配置测试通过。

## 8. Beads 追踪

- Beads: `homie-qzv`
- change_id: `agent-default-model-fallback`
- 类型: bug
- 优先级: P1
