# OpenSpec Plan — llm-gateway-model-routing

## 1. 变更概述

让 `homie.local.json` 的 `models` 映射（PRD2 已录入）在网关转发时真正生效：按请求路径
（`/v1/responses` ↔ codex、`/v1/messages` ↔ claude）改写请求 body 的 `model` 字段，实现
「不同 agent 配置不同模型」。未配置则透传，向后兼容。

## 2. 模块划分

```text
homie/crates/homie-gateway/
├── src/config.rs   # GatewayConfig 增 models；from_file 填充；去 dead_code
├── src/state.rs    # AppState 增 models
├── src/routes.rs   # route_key + apply_model_route + forward 前改写
└── src/main.rs     # 传 models 进 AppState
```

依赖：`llm-gateway-virtual-keys`（网关/转发/用量，已✓）、`homie-cli-config-ops`（models 录入，
已✓）。无新增第三方依赖。

## 3. 层级关系

| 层 | 产物 |
|----|------|
| 需求 | `prd-spec/features/llm-gateway-model-routing/2026-08-18-llm-gateway-model-routing-design.md` |
| 规范 | `specs/llm-gateway.md` §7 Model Routing（新增） |
| 执行 | 本 OpenSpec + `homie-gateway/{config,state,routes,main}.rs` |
| 证据 | `docs/verification/llm-gateway-model-routing/` |

## 4. 与既有权威的关系

- 不改 `homie-engine::inject`（模型路由由网关统一改写，非注入层）。
- 不改 `specs/llm-gateway.md` §5 Protocol Contract / §6 Upstream Forwarding 的既有透传语义，
  仅在 §6 后新增 §7 Model Routing。
- 不改虚拟 key / 鉴权 / 用量合同的既有语义（§3/§4/§8 仅序号后移）。

## 5. 安全边界

改写仅触碰 JSON body 顶层 `model` 字段，不涉及 `api_key`/master key/虚拟 key/敏感 prompt，
无新增泄露面。属 Tier 1（非 credential custody，但紧邻代理路径），仍需覆盖改写/透传分支的
单测 + 集成测试。

## 6. 后续 child Bead（本变更只声明，不实现）

- `llm-gateway-policy-quota`（配额/限流/策略/审计，依赖本变更的按模型路由）。
- `llm-gateway-credential-login`（Claude/Codex 登录凭证接入上游，依赖本变更明确模型-凭证绑定）。
- per-agent 模型映射图形 UI。
