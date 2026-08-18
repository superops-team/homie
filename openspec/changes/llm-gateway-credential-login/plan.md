# llm-gateway-credential-login OpenSpec Plan

## 1. 目标

把 Claude Code / Codex 登录凭证接入网关上游（Phase 1：Codex API-key 模式最小闭环）：

- `homie-node` 新增 `credentials` 模块，以库内嵌方式暴露 `resolve_default_codex_credential` /
  `resolve_codex_api_key`，按 profile 返回短期上游 token。
- `homie-gateway` 新增可选 `credential_source`（`static` 默认 / `node`），经库内嵌调用 node
  凭证函数动态解析，失败回退静态 `upstream.api_key`。

本变更不实现 Claude OAuth / Codex ChatGPT 登录 token 的 refresh（Phase 2），不改注入/虚拟 key/
模型路由/策略。

## 2. 输入文档

- PRD：`prd-spec/features/llm-gateway-credential-login/2026-08-19-llm-gateway-credential-login-design.md`
- Spec：`specs/llm-gateway.md` §11 Credential Source
- Beads：`homie-gmq`

## 3. 模块规划

### M1: node 侧库内嵌凭证解析（受限凭证解析）

职责：

- `homie-node` 新增 `credentials` 模块，暴露 `resolve_default_codex_credential(paths)` /
  `resolve_codex_api_key(paths, profile_id)`，出 `ResolvedCredential { kind, base_url, token }`。
- Phase 1 仅 `kind == codex_api_key`：读 `accounts/codex/<profile_id>/auth.json` 的
  `OPENAI_API_KEY`。
- 缺失/坏 JSON/非 API-key 模式 → `NodeError::NotFound`；不暴露任意文件读取、不返回 refresh token。

涉及：

- `homie/crates/homie-node/src/credentials.rs`（新模块）
- `homie/crates/homie-node/src/lib.rs`（`pub mod credentials` + re-export）

### M2: gateway 侧可选凭证源 + 回退

职责：

- `GatewayConfig` 增 `credential_source`（`#[serde(default)]` = `static`）。
- `Upstream` 支持动态凭证解析器（trait/`enum`），`forward` 前解析当前 token。
- `node` 模式：库内嵌调用 `homie_node::credentials::resolve_default_codex_credential`；
  失败回退 `upstream.api_key`；皆空返回配置错误（无密钥）。
- token 仅内存、不进 SQLite/日志。

涉及：

- `homie/crates/homie-gateway/src/config.rs`、`upstream.rs`、`main.rs`
- `homie/crates/homie-gateway/Cargo.toml` 新增 `homie-node.workspace = true` 依赖

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
| 网关引入 node 依赖 | 库内嵌仅调用纯函数，`static` 模式不触发 node 凭证解析 |
| 凭证泄露 | token 仅内存、审计去 token、单测断言无明文 |
