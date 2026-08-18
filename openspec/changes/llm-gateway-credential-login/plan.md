# llm-gateway-credential-login OpenSpec Plan

## 1. 目标

把 Claude Code / Codex 登录凭证接入网关上游（Phase 1：Codex API-key 模式最小闭环）：

- `homie-node` 新增受限 `credential.resolve`，按 profile 返回短期上游 token。
- `homie-gateway` 新增可选 `credential_source`（`static` 默认 / `node`），经 node 动态解析，
  失败回退静态 `upstream.api_key`。

本变更不实现 Claude OAuth / Codex ChatGPT 登录 token 的 refresh（Phase 2），不改注入/虚拟 key/
模型路由/策略。

## 2. 输入文档

- PRD：`prd-spec/features/llm-gateway-credential-login/2026-08-19-llm-gateway-credential-login-design.md`
- Spec：`specs/llm-gateway.md` §11 Credential Source
- Beads：`homie-gmq`

## 3. 模块规划

### M1: node 侧 `credential.resolve`（受限凭证解析）

职责：

- `homie-node` 新增 `credential.resolve` 方法（入 `profile_id`，出 `{ kind, base_url, token }`）。
- Phase 1 仅 `kind == codex_api_key`：读 `config_home/auth.json` 的 `OPENAI_API_KEY`。
- 缺失/坏 JSON/非 API-key 模式 → `NotAuthenticated`；不暴露任意文件读取、不返回 refresh token。

涉及：

- `homie/crates/homie-proto/src/node.rs`（`NodeMethod`、请求/响应类型）
- `homie/crates/homie-node/src/provider/`（解析 + `credential.resolve` 分发）
- `homie/crates/homie-node/src/service.rs`（RPC 接线）

### M2: gateway 侧可选凭证源 + 回退

职责：

- `GatewayConfig` 增 `credential_source`（`#[serde(default)]` = `static`）。
- `Upstream` 支持动态凭证解析器（trait/`enum`），`forward` 前解析当前 token。
- `node` 模式：经 `NodeClient` 调 `credential.resolve`；失败回退 `upstream.api_key`；皆空返回
  `503` 配置错误 body（无密钥）。
- token 仅内存、不进 SQLite/日志。

涉及：

- `homie/crates/homie-gateway/src/config.rs`、`upstream.rs`、`state.rs`、`routes.rs`
- 新增 node 客户端依赖（复用 `homie-client` 或最小 `credential.resolve` 调用点）

### M3: 测试 + 安全 + 证据

职责：

- 单测（auth.json 各形态解析、配置解析、回退、503 无泄露）。
- 集成测试（node 模式转发用解析 token；回退静态 key；无凭证 503）。
- 安全断言（日志/SQLite 不含 token 明文）。
- 证据落 `docs/verification/llm-gateway-credential-login/`。

涉及：

- `homie/crates/homie-gateway/tests/`、`homie/crates/homie-node/tests/`
- `docs/verification/llm-gateway-credential-login/`

## 4. 验收标准

- Codex API-key 登录后 `credential_source = "node"` 可无手动 `api_key` 完成端到端转发。
- 默认 `static` 模式既有行为不变。
- 无凭证时明确报错且不泄露密钥。

## 5. 风险

| 风险 | 控制 |
|---|---|
| auth.json 私有格式易碎 | Phase 1 仅解析 `OPENAI_API_KEY` 字段，坏格式→NotAuthenticated，不 panic |
| 网关引入 node 客户端依赖过重 | 复用 `homie-client` 最小调用面，`static` 模式零 node 依赖 |
| 凭证泄露 | token 仅内存、审计去 token、单测断言无明文 |
