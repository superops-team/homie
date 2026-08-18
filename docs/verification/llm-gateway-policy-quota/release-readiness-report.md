# Release Readiness Report — llm-gateway-policy-quota

- Beads: `homie-n6a`
- change_id: `llm-gateway-policy-quota`
- 日期: 2026-08-18

## 1. 交付内容

| 组件 | 路径 | 状态 |
|------|------|------|
| `Policy` 配置加载（可选） | `homie/crates/homie-gateway/src/config.rs` | 完成 |
| 限流 + 配额判定 + 429 + 审计 | `homie/crates/homie-gateway/src/policy.rs`（新增） | 完成 |
| `AppState` 携带 policy + rate limiter | `homie/crates/homie-gateway/src/state.rs` | 完成 |
| 转发前策略判定 | `homie/crates/homie-gateway/src/routes.rs` | 完成 |
| `gateway_audit` 表 + `sum_tokens_since` | `homie/crates/homie-gateway/src/db.rs`、`usage.rs` | 完成 |
| 组件合同 §9 Policy And Quota | `specs/llm-gateway.md` | 完成 |
| 集成测试（限流/配额/master 旁路） | `homie/crates/homie-gateway/tests/gateway.rs` | 完成 |

## 2. 门禁结果

| 门禁 | 命令 | 结果 |
|------|------|------|
| 格式 | `cargo fmt --all --check` | ✅ 通过 |
| 网关单测 | `cargo test -p homie-gateway --offline` | ✅ 28 lib + 13 integration + 0 doc 全部通过 |
| 网关 clippy | `cargo clippy -p homie-gateway --all-targets --offline` | ✅ 干净（0 warning） |

> `homie-engine` 的 4 条 clippy warning 为既有问题，不属于本变更范围，未在本 PRD 中引入或修复。

## 3. 功能验证

### 3.1 单元测试（lib，28 通过，新增 6）

- `loads_policy_when_present` / `policy_defaults_to_none`：policy 反序列化 + camelCase + 缺失为 None。
- `rate_limiter_allows_within_window` / `resets_on_new_window` / `zero_means_unconfigured`：
  分钟窗口计数、窗口切换清零、0 值不限制。
- `quota_zero_means_unconfigured` / `quota_checks_cumulative_tokens`：0 值不限制、聚合 input+output。
- `deny_response_is_429_and_sanitized`：429 状态码。

### 3.2 集成测试（tests/gateway.rs，13 通过，新增 3）

- `rate_limit_rejects_excess_requests`：`requests_per_minute=1`，第 2 次请求 `429
  rate_limit_error`，且仅第 1 次请求被转发并记录用量。
- `quota_rejects_when_daily_limit_exceeded`：`daily_token_limit=10`，首请求记录 5+5=10 token，
  第 2 次请求 `429 quota_error`，仅首请求计入用量。
- `master_key_bypasses_policy`：master key 不受限流，连续两次请求均 `200`。

## 4. 验收标准核对（PRD §8）

1. ✅ `policy` 段可选加载，缺失不限制（`policy_defaults_to_none`）。
2. ✅ 每 key 超 `requests_per_minute` 返回 `429 rate_limit`，不转发上游。
3. ✅ 每 key 超 `daily_token_limit` 返回 `429 quota`，不转发上游。
4. ✅ 拒绝事件写 `gateway_audit`，放行请求用量照旧写 `gateway_usage`。
5. ✅ 拒绝响应与审计不含密钥/敏感 prompt（429 body 仅 `error.type`/`error.message`）。

## 5. 安全验证

- 拒绝响应 body 仅含 `error.type` / `error.message`，不含 key、model、prompt。
- 策略判定在鉴权通过后、转发前；master key 旁路策略（受信管理面）。
- 拒绝请求不写 `gateway_usage`，避免污染用量统计。

## 6. 结论

所有验收标准满足，门禁全绿，证据齐备。可发布并打 tag `v0.5.0`。
