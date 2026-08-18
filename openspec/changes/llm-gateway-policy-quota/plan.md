# OpenSpec Plan — llm-gateway-policy-quota

## 1. 变更概述

在既有用量链路上补上每虚拟 key 的限流与配额准入控制：`homie.local.json` 可选 `policy` 段
声明 `requests_per_minute`（内存滑动窗口）与 `daily_token_limit`（SQLite 聚合），转发前判定，
超限返回 `429` 并记录拒绝审计事件。未配置则完全向后兼容。

## 2. 模块划分

```text
homie/crates/homie-gateway/
├── src/config.rs   # Policy 反序列化（Option，默认 None）
├── src/state.rs    # AppState 增 policy + RateLimiter
├── src/policy.rs   # 新增：RateLimiter + QuotaChecker + deny_response（纯逻辑）
├── src/routes.rs   # forward_and_record 转发前判定
├── src/db.rs       # gateway_audit 表 + 拒绝事件写入
└── src/main.rs     # 传 policy 进 AppState
```

依赖：`llm-gateway-virtual-keys`（用量存储，已✓）、`llm-gateway-model-routing`（路由同层，已✓）。
无新增第三方依赖。

## 3. 层级关系

| 层 | 产物 |
|----|------|
| 需求 | `prd-spec/features/llm-gateway-policy-quota/2026-08-18-llm-gateway-policy-quota-design.md` |
| 规范 | `specs/llm-gateway.md` §9 Policy And Quota（新增） |
| 执行 | 本 OpenSpec + `homie-gateway/{config,state,policy,routes,db,main}.rs` |
| 证据 | `docs/verification/llm-gateway-policy-quota/` |

## 4. 与既有权威的关系

- 不改虚拟 key 签发/鉴权/用量合同（§3/§4/§8 语义不变，仅新增 §9）。
- 不改 `homie-engine::inject`、不改 `Upstream::forward`、不改模型路由（§7）。
- 拒绝请求不写 `gateway_usage`（用量只记实际转发），新增 `gateway_audit` 仅记拒绝事件。

## 5. 安全边界

限流/配额判定发生在鉴权通过后、转发前；拒绝响应与审计 detail 不含密钥/敏感 prompt。属
Tier 1（非 credential custody，紧邻代理路径），覆盖限流/配额/429 分支的单测 + 集成测试。

## 6. 后续 child Bead（本变更只声明，不实现）

- `llm-gateway-credential-login`（Claude/Codex 登录凭证接入上游，需先 design/research）。
- per-key 策略覆盖。
- 限流/配额可视化 UI。
