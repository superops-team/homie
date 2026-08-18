# Spec Review Report — llm-gateway-policy-quota

## 1. 范围

本报告记录 `llm-gateway-policy-quota`（Beads `homie-n6a`）实现前的 spec 与可行性评审结论，
覆盖 `policy` 配置加载、每 key 速率限制、每 key 配额、429 拒绝响应、审计事件与安全边界。

## 2. 技术选型评审（可行性）

| 决策 | 结论 | 依据 |
|------|------|------|
| 限流内存滑动窗口，配额查 SQLite 聚合 | **采纳** | 限流高频短时可随重启归零；配额低频长时需持久化，用量表已承载 |
| `policy` 可选、默认不限制 | **采纳** | `#[serde(default)]`，向后兼容；与 PRD3「未配置透传」覆盖语义一致 |
| 0 值视为未配置 | **采纳** | 避免 `daily_token_limit=0` 误锁死全部请求 |
| 429 统一 `rate_limit_error` / `quota_error` | **采纳** | 复用标准状态码，不新增语义复杂度 |
| 拒绝请求不写 `gateway_usage`，仅写 `gateway_audit` | **采纳** | 用量只记实际转发；审计只记拒绝，最小落库 |
| 无新增第三方依赖 | **采纳** | 复用 `rusqlite`、`axum`、标准库 `HashMap` |

## 3. 依赖评估

| 依赖 | 状态 | 理由 |
|------|------|------|
| `rusqlite` | workspace 已有 | `gateway_audit` 表 + `sum_tokens_since` 聚合 |
| `axum` | workspace 已有 | `429` 结构化响应 |
| `std::collections::HashMap` | 标准库 | 内存速率窗口 |
| 上游 `llm-gateway-virtual-keys` | 已✓ | 提供用量存储与虚拟 key |
| 上游 `llm-gateway-model-routing` | 已✓ | 策略与路由同层判定 |
| 无新增包 | — | 符合依赖添加政策 |

## 4. 组件合同评审

`specs/llm-gateway.md` 新增 §9 Policy And Quota，§10 Security And Recovery 序号后移。评审结论：

- §9 定义 `policy` 可选、`requests_per_minute` / `daily_token_limit` 语义、0 值视为未配置、
  429 拒绝、`gateway_audit` 拒绝事件、不写 `gateway_usage`。
- §9 与 PRD FR-1~FR-5 一一对应，无缺口。
- §3/§4/§7/§8 既有语义未被改动，仅新增 §9。

## 5. 边界情况核对

| 场景 | 处理 | 已验证 |
|------|------|--------|
| `policy` 整体缺失 | `None`，不限制 | `policy_defaults_to_none` |
| 仅 `rate_limit` / 仅 `quota` | 只做对应限制 | 单测 `zero_means_unconfigured` |
| `daily_token_limit = 0` | 不限制 | `quota_zero_means_unconfigured` |
| `requests_per_minute = 0` | 不限制 | `rate_limiter_zero_means_unconfigured` |
| key 无用记录 | SUM 空 → 0，不触发 | `quota_checks_cumulative_tokens` |
| 拒绝请求 | 不写 `gateway_usage`，仅 `gateway_audit` | 集成测试断言 usage len=1 |

## 6. 安全 Tier 分级

限流/配额判定发生在鉴权通过后、转发前；拒绝响应与审计 detail 不含密钥/敏感 prompt。属
Tier 1（非 credential custody，紧邻代理路径），覆盖限流/配额/429 分支单测 + 集成测试。

## 7. 与既有权威的一致性

- 不改虚拟 key 签发/鉴权/用量合同（§3/§4/§8）。
- 不改 `homie-engine::inject`、不改 `Upstream::forward`、不改模型路由（§7）。
- master key 不参与限流/配额（受信管理面），集成测试 `master_key_bypasses_policy` 覆盖。

## 8. 结论

方案可行，规格齐备，边界与安全边界明确，可进入实现阶段。
