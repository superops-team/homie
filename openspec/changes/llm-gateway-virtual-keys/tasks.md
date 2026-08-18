# OpenSpec Tasks — llm-gateway-virtual-keys

本变更为 Rust 实现，覆盖网关 crate、虚拟 key、鉴权、上游转发、用量、agent 注入与端到端验证。

## T1: 网关 crate 脚手架与依赖（Cargo.toml / main.rs / workspace）

- 交付：新增 `homie/crates/homie-gateway`，注册为 `homie/Cargo.toml` workspace member +
  workspace dependency；`axum` 依赖按 owner crate + feature flags 声明；`main.rs` 可解析配置并
  绑 `127.0.0.1:<port>`。
- 验收：`cargo check -p homie-gateway` 通过；监听地址强制为回环地址。
- 关联验证 Case：FC-1。

## T2: 本地配置加载（config.rs）

- 交付：`gateway.local.toml`（或 `HOMIE_GATEWAY_CONFIG`）加载 `listen`/`base_url`/`api_key`/
  `master_key`；缺失上游凭证时启动失败并给出明确错误。
- 验收：配置加载单测 + 缺失凭证失败路径 + 非回环 bind 拒绝。
- 关联验证 Case：FC-2。

## T3: 虚拟 key store + 持久化（auth/api_keys.rs + usage SQLite）

- 交付：`GatewayApiKeyStore`（vendor 自 litellm-rust，`sk-` 生成、`create`/`delete`/`list`/
  `accepts`、`last_used_at` 更新），用 `rusqlite` 替换内存 HashMap 实现持久化。
- 验收：创建/校验/撤销/列表单测；重启后 key 仍可校验；撤销后 401。
- 关联验证 Case：FC-3。

## T4: 鉴权中间件（auth/master_key.rs + auth/mod.rs）

- 交付：master key + 虚拟 key，`Authorization: Bearer` 与 `x-api-key` 双 header，Bearer 优先；
  无 master key + 非回环 bind 硬失败；401 标准错误体脱敏。
- 验收：master/virtual/优先级/401 单测；负向控制（已知坏 key 触发失败路径）。
- 关联验证 Case：FC-4。

## T5: 上游 OpenAI-compatible 转发（providers/openai.rs）

- 交付：`/v1/responses` 与 `/v1/messages` 转发到单一上游 `base_url`+`api_key`，SSE 流式保留；
  上游 key 仅服务端附加，忽略调用方 key。
- 验收：wiremock 模拟上游，断言请求构造、响应回显、流式、错误脱敏。
- 关联验证 Case：FC-5。

## T6: HTTP 路由（http/mod.rs + responses.rs + messages.rs）

- 交付：`POST /v1/responses`、`POST /v1/messages` 路由注册，经鉴权中间件后进入转发。
- 验收：路由命中/鉴权顺序/协议形状单测。
- 关联验证 Case：FC-6。

## T7: 用量记录（usage.rs）

- 交付：按虚拟 key 记录 `model`/`occurred_at`/`input_tokens`/`output_tokens`，复用
  `homie-usage::openai_estimate`。
- 验收：转发后用量落库单测；估算字段标记为非权威。
- 关联验证 Case：FC-7。

## T8: agent 配置自动注入（homie-engine inject.rs + agent.rs + manifests）

- 交付：`InjectionSpec` 新增 `codex_gateway`/`claude_gateway`；`injection_args()` 为 Codex 追加
  `-c model_provider="homie" -c model_providers.homie.base_url=<gateway>/v1 -c
  model_providers.homie.wire_api="responses" -c model_providers.homie.env_key=<env>`；Claude 注入
  `ANTHROPIC_BASE_URL`/`ANTHROPIC_AUTH_TOKEN` env；`codex.json`/`claude.json` manifest 声明注入。
- 验收：argv/env 形状单测；虚拟 key 随 session 注入且不泄露到其他 agent。
- 关联验证 Case：FC-8。

## T9: 端到端集成测试（tests/gateway.rs + engine spawn）

- 交付：真实启动网关（127.0.0.1），虚拟 key 走 `/v1/responses` 与 `/v1/messages` 转发到
  wiremock 上游，断言用量落库；spawn 路径断言 session argv/env 含正确网关指向。
- 验收：全链路通过；`cargo test -p homie-gateway` 与 `cargo test -p homie-engine inject` 绿。
- 关联验证 Case：FC-9。

## T10: 规范记录 + 依赖记录 + 证据 + 关闭

- 交付：`docs/architecture/project-layout.md` 登记 `homie-gateway`；
  `docs/research/rust-package-selection.md` 记录 axum 选择；`.gitignore` 忽略 `gateway.local.toml`；
  OpenSpec alignment + coverage/mutation 证据齐备；Beads `homie-f91` 关闭。
- 关联验证 Case：FC-10。
