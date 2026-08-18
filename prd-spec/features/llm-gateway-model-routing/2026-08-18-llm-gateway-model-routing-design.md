# Homie LLM 网关模型路由：按 agent 改写转发 model 设计文档

## 1. 概述

### 1.1 问题/背景

PRD2 `homie-cli-config-ops`（Beads `homie-ys0`，已关闭）交付了 `homie config set
--model-codex <m> --model-claude <m>`，把 per-agent 模型映射写入 `homie.local.json` 的
`models` 字段，`config show` 也能回显该映射。

但网关侧该映射**完全不生效**：

- `homie/crates/homie-gateway/src/config.rs` 中 `FileConfig.models` 被标为
  `#[allow(dead_code)]`，注释明写「the gateway MVP reads but does not use it」；
- `GatewayConfig`（运行时结构）没有 `models` 字段，`state.rs` 的 `AppState` 也没有；
- `upstream.rs::Upstream::forward` 是纯透传（`POST {base_url}{path}` 原样转发 body），
  不改写请求里的 `model`；
- `routes.rs::forward_and_record` 用 `extract_model` 读请求 body 的原始 `model` 只用于
  用量记录，转发前不改写。

结果：用户在 `config set --model-codex gpt-5.2-codex` 之后，Codex 会话经网关仍以 agent 自身
默认模型请求上游，「不同 agent 配置不同模型」这一 Homie 核心能力没有真正落地。

本 PRD 补齐这条断点：让 `models.codex` / `models.claude` 在网关转发时**真正改写**请求的
`model`，成为模型路由的单一事实来源。

### 1.2 目标

1. 网关运行时加载 `homie.local.json` 的 `models` 映射（去掉 dead_code）。
2. 转发前按 agent 改写请求 body 的 `model` 字段：
   - `POST /v1/responses`（Codex）→ 改写为 `models.codex`（若已配置）；
   - `POST /v1/messages`（Claude）→ 改写为 `models.claude`（若已配置）。
3. 未配置对应映射时，透传原 `model`（向后兼容现状，不强制要求配置）。
4. 用量记录使用**改写后**的 model（反映实际路由的模型）。

### 1.3 非目标

- 不做 Anthropic Messages → OpenAI 协议的语义转换（`/v1/messages` 继续走 PRD1 既有的透传
  语义；模型名是否被上游识别由用户配置决定）。
- 不做 per-agent 模型映射的图形 UI（后续）。
- 不做配额/限流/策略/审计（child Bead `llm-gateway-policy-quota`）。
- 不改注入逻辑（spawn 时不在 `-c`/env 里额外注入 model；模型路由由网关统一改写）。
- 不接入 Claude/Codex 登录凭证为上游（child Bead `llm-gateway-credential-login`）。

### 1.4 关键设计决策

#### 决策 A：路由键 = 请求路径（`/v1/responses` ↔ codex、`/v1/messages` ↔ claude）

路由键用 HTTP 路径而非虚拟 key 的 label。理由：路径天然区分 agent 协议（PRD1 已定
`/v1/responses` 供 Codex、`/v1/messages` 供 Claude Code），无需给虚拟 key 增加 agent 标签，
也无需改签发逻辑。这是最小改动。

#### 决策 B：网关层改写，而非注入层注入 model

模型路由的单一事实来源放在网关（`forward_and_record` 改写 body），而非在 spawn 注入时给
Codex 加 `-c model=...` / 给 Claude 加 model env。理由：

- 网关是统一入口，改写一次即对全部 agent 生效；
- 避免改 `homie-engine::inject` 与各 manifest，改动面最小；
- 用户改 `models` 配置后即时生效，无需重 spawn。

#### 决策 C：覆盖语义 = 配置则改写，未配置则透传

`models` 是可选映射（`#[serde(default)]`）。未配置某 agent 的模型时，保持原样透传，保证
MVP 向后兼容（已有部署不受影响）。这避免引入「必须配置才能用」的强制前置。

## 2. 用户场景

### 场景 1：Codex 走指定上游模型

**Given** 用户已 `homie config set --model-codex gpt-5.2-codex`。  
**When** 一个 Codex 会话经网关请求 `POST /v1/responses`，body 内 `model` 为 agent 默认值。  
**Then** 网关改写 `model` 为 `gpt-5.2-codex` 后转发上游，用量记录该模型。

### 场景 2：Claude 走指定上游模型

**Given** 用户已 `homie config set --model-claude claude-sonnet-4-5`。  
**When** 一个 Claude 会话经网关请求 `POST /v1/messages`。  
**Then** 网关改写 `model` 为 `claude-sonnet-4-5` 后转发，用量记录该模型。

### 场景 3：未配置模型则透传

**Given** 用户未配置 `models.codex`（或整个 `models` 缺失）。  
**When** Codex 经网关请求，body `model` 为 `gpt-5`。  
**Then** 网关不改写，透传 `gpt-5`，行为与当前一致。

### 场景 4：配置即时生效

**Given** 网关运行中，用户 `homie config set --model-codex gpt-5.2-codex` 并重启网关。  
**When** 后续 Codex 请求。  
**Then** 网关按新配置改写（网关启动时加载 `homie.local.json`，重启后生效）。

## 3. 功能需求

### FR-1: 运行时加载 models 映射

- `GatewayConfig` 新增 `models: BTreeMap<String, String>`（key 为 `codex` / `claude`），
  从 `homie.local.json` 的 `models` 字段反序列化，`#[serde(default)]` 允许缺失。
- 去掉 `FileConfig.models` 的 `#[allow(dead_code)]`，`from_file` 填充到 `GatewayConfig`。

### FR-2: 按路径改写转发 model

- `POST /v1/responses`：若 `models` 含 `codex`，改写 body 顶层 `model` 字段为
  `models["codex"]`；否则透传。
- `POST /v1/messages`：若 `models` 含 `claude`，改写 body 顶层 `model` 字段为
  `models["claude"]`；否则透传。
- 改写仅针对 JSON body 的顶层 `model` 字符串；无法解析 JSON 时按原样透传（不报错）。

### FR-3: 用量记录使用改写后 model

- `forward_and_record` 的 `model` 取**改写后**的值，写入用量（反映实际路由模型）。

### FR-4: 安全边界（不引入新泄露面）

- 改写仅涉及 `model` 字段，不触碰/不回显 `api_key`、master key、虚拟 key、敏感 prompt。
- 错误/日志不新增 key 泄露。

## 4. 实现方案

### 4.1 改动点

```text
homie/crates/homie-gateway/
├── src/config.rs     # GatewayConfig 增 models；from_file 填充；去 dead_code
├── src/state.rs      # AppState 增 models（或 model routing 视图）
├── src/routes.rs     # forward_and_record 前按 path 改写 body model
└── src/upstream.rs   # 复用现有 forward，不改协议
```

### 4.2 数据流

```text
agent → /v1/{responses|messages} (body.model=X)
        → route_model(path, models, body)  # 改写 model 为 models[codex|claude]
        → upstream.forward(path, rewritten_body)
        → usage.record(id, rewritten_model, …)
```

### 4.3 核心函数（routes.rs）

```rust
/// 路由键：路径 → agent 键。
fn route_key(path: &str) -> Option<&'static str> {
    match path {
        "/responses" => Some("codex"),
        "/messages" => Some("claude"),
        _ => None,
    }
}

/// 改写 body 顶层 `model` 字段（配置存在时）；无配置/非 JSON 则原样返回。
fn apply_model_route(models: &BTreeMap<String, String>, key: &str, body: &[u8]) -> Vec<u8> {
    // 1. models.get(key) 缺失 → 返回 body 原样
    // 2. serde_json 解析为 Value，顶层 object["model"] 覆盖为 target
    // 3. 解析失败 → 返回 body 原样
}
```

### 4.4 路径契约

`models` 映射 key 固定为 `codex` / `claude`，与 `homie.local.json` 的 `models` 字段、
Swift `HomieConfigStore` 的 `models: [String: String]` 对齐（PRD2 已定 camelCase schema）。

## 5. 边界情况

| 场景 | 处理 |
|------|------|
| `models` 整体缺失 | `#[serde(default)]` → 空映射，全部透传 |
| `models.codex` 缺失、仅配 `models.claude` | codex 请求透传，claude 请求改写 |
| 请求 body 非 JSON / 无顶层 `model` | 原样透传，不改写，不报错 |
| `model` 字段为 null / 非字符串 | 仅当为字符串时改写，否则透传 |
| 网关启动时 `homie.local.json` 损坏 | 沿用 PRD1 `load` 硬失败语义，不静默降级 |

## 6. 涉及文件

- `homie/crates/homie-gateway/src/config.rs`（models 加载）
- `homie/crates/homie-gateway/src/state.rs`（AppState 携带 models）
- `homie/crates/homie-gateway/src/routes.rs`（改写 + 用量 model）
- `homie/crates/homie-gateway/src/main.rs`（传 models 进 AppState）
- `specs/llm-gateway.md`（新增 Model Routing 合同章节）
- `docs/verification/llm-gateway-model-routing/`（证据）

## 7. 验证计划

### 7.1 单元测试（Rust）

- `route_key`：路径 → 键映射。
- `apply_model_route`：配置存在则改写、缺失则透传、非 JSON 透传、model 非字符串透传。
- `GatewayConfig::from_file`：models 反序列化、缺失为空、camelCase 对齐。

### 7.2 集成测试（tests/gateway.rs 或新增）

- 配置 `models.codex` 后，`/v1/responses` 转发 body 的 `model` 被改写（用 mock upstream 断言
  收到的 body）。
- 配置 `models.claude` 后，`/v1/messages` 同理。
- 未配置时透传原 model。
- 用量记录 model 为改写后的值。

### 7.3 门禁

- `cargo fmt --all --check`
- `cargo test -p homie-gateway --offline`
- `cargo clippy -p homie-gateway --all-targets --offline`（干净）

## 8. 验收标准

1. `GatewayConfig` 加载 `models`，`homie.local.json` 的 `models.codex/claude` 生效。
2. `/v1/responses` 按 `models.codex` 改写 model；`/v1/messages` 按 `models.claude` 改写。
3. 未配置时透传，向后兼容。
4. 用量记录为改写后 model。
5. 无新增 key/敏感信息泄露面。

## 9. Beads 追踪

- Beads: `homie-48w`
- change_id: `llm-gateway-model-routing`
- 类型: feature
- 优先级: P0
- 依赖: `llm-gateway-virtual-keys`（已✓）、`homie-cli-config-ops`（已✓，提供 models 录入）
