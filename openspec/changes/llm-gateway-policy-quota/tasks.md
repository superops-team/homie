# OpenSpec Tasks — llm-gateway-policy-quota

## T1: Policy 配置加载

- 交付：`GatewayConfig` 增 `policy: Option<Policy>`；`Policy`/`RateLimit`/`Quota` 结构
  `#[serde(default)]`；`from_file` 填充。
- 验收：`homie.local.json` 的 `policy` 反序列化；缺失为 `None`；camelCase 对齐；0 值视为未配置。
- 关联验证 Case：FC-1。

## T2: AppState 携带 policy + RateLimiter

- 交付：`state.rs` `AppState` 增 `policy` + `RateLimiter`；`main.rs` 传入。
- 验收：编译通过，`AppState::new` 签名更新且调用点一致。
- 关联验证 Case：FC-1。

## T3: 速率限制（内存滑动窗口）

- 交付：`policy.rs` `RateLimiter::allow(key, rpm, now)` 分钟粒度窗口。
- 验收：窗口内超限拒绝、窗口切换清零、rpm=0 不限制。
- 关联验证 Case：FC-2。

## T4: 配额（SQLite 聚合）

- 交付：`policy.rs` `QuotaChecker::allow(key, daily_limit, now)` 读 `gateway_usage` SUM。
- 验收：无记录/低于/高于阈值；聚合输入+输出；limit=0 不限制。
- 关联验证 Case：FC-3。

## T5: 429 拒绝响应 + 审计事件

- 交付：`deny_response(DenyReason)` 构造 429 结构化 body；`db.rs` 增 `gateway_audit` 表 +
  `record_audit`；`forward_and_record` 转发前判定并记录拒绝事件。
- 验收：429 body 不含密钥/敏感 prompt；拒绝事件落库；拒绝不写 `gateway_usage`。
- 关联验证 Case：FC-4、FC-5。

## T6: 单测 + 集成测试

- 交付：`policy.rs` 单测；`tests/gateway.rs` 限流/配额/默认不限制集成测试。
- 验收：`cargo test -p homie-gateway` 全绿。
- 关联验证 Case：FC-5。

## T7: 门禁 + 证据 + 关闭

- 交付：`cargo fmt`/`test`/`clippy` 绿；spec review + release readiness 证据；Beads `homie-n6a`
  关闭。
- 关联验证 Case：FC-6。
