# Homie LLM 网关策略：虚拟 key 配额 / 限流 / 审计设计文档

## 1. 概述

### 1.1 问题/背景

PRD1 `llm-gateway-virtual-keys`（Beads 已关闭）交付了本地网关 + 虚拟 key 签发 + 上游转发 +
用量记录；PRD3 `llm-gateway-model-routing`（Beads `homie-48w`，已关闭）补齐了按 agent 改写
上游模型。当前网关对「调用频率」与「累计消耗」**没有任何准入控制**：

- 任一虚拟 key 可无限频次地请求上游，无每 key 速率上限；
- 任一虚拟 key 可无限累积 token 消耗，无每日/总配额；
- 超支只能靠上游 provider 自身的限额被动兜底，Homie 侧无法预警/拦截。

结果：本地代理对「防跑飞、防超支」缺乏第一道闸，与 AGENTS.md 声明的「Homie 再应用策略、
记录用量」中的「策略」环节缺失。本 PRD 在既有用量链路上补上**每虚拟 key 的限流与配额**，
并让用量记录成为审计的基础。

### 1.2 目标

1. `homie.local.json` 增加可选 `policy` 段，声明网关级限流与配额（默认关闭/宽松）。
2. 转发前对**每个虚拟 key** 做速率限制：超过每分钟请求数上限则拒绝，返回 `429`。
3. 转发前对**每个虚拟 key** 做配额检查：当前计费周期累计 token（输入+输出）超限则拒绝，
   返回 `429`。
4. 限流/配额命中与放行均计入本地 SQLite，作为审计依据。
5. 拒绝响应不触碰/不回显任何密钥或敏感 prompt，无新增泄露面。

### 1.3 非目标

- 不做 per-key 独立覆盖（仅网关级全局策略；per-key 覆盖属后续 child Bead）。
- 不做多租户/多组织配额、不做跨节点分布式限流（本地单进程）。
- 不做计费/账单/对账（用量仍是估算，非权威账单）。
- 不接入 provider 侧配额/限流（Claude/Codex 登录凭证接入上游属 `credential-login`）。
- 不做限流的可视化 UI（仅 `homie config` / `homie doctor` 回显策略）。

### 1.4 关键设计决策

#### 决策 A：限流在内存滑动窗口，配额查 SQLite 聚合

- 限流（每分钟请求数）用进程内 `HashMap<key_id, 环形窗口计数>`，O(1) 判定，不落库、不新增
  表。理由：限流是高频、短时、可随重启归零的语义，内存足够且避免每请求写库。
- 配额（周期内累计 token）查询既有 `gateway_usage` 表按 `key_id` + 周期 `SUM(input_tokens +
  output_tokens)`。理由：配额是低频、长时、需持久化的语义，用量表已天然承载。

#### 决策 B：策略可选，默认不限制（向后兼容）

`policy` 段 `#[serde(default)]`，整体缺失即「不启用任何限流/配额」，与既有部署零冲突；与
PRD3「未配置则透传」的覆盖语义一致。

#### 决策 C：拒绝统一返回 `429 Too Many Requests` + 结构化 body

命中限流返回 `429`，命中配额也返回 `429`（区分 `reason` 字段 `rate_limit` / `quota`）。不
新增 HTTP 状态码语义复杂度；不泄露 key、model、prompt。

#### 决策 D：策略在网关层判定，不改注入/上游

策略在 `forward_and_record` 转发前判定，与模型路由（PRD3）同层；不改 `homie-engine::inject`、
不改 `Upstream::forward`、不改虚拟 key 签发。

## 2. 用户场景

### 场景 1：限流拦截高频请求

**Given** 网关配置 `policy.rate_limit.requests_per_minute = 10`。  
**When** 某虚拟 key 在 60 秒内发起第 11 次请求。  
**Then** 网关拒绝该请求，返回 `429`，body 含 `reason:"rate_limit"`，且该请求不转发上游。

### 场景 2：配额拦截超支

**Given** 网关配置 `policy.quota.daily_token_limit = 100000`。  
**When** 某虚拟 key 在当天累计（输入+输出）token 已超 `100000`，再次请求。  
**Then** 网关拒绝，返回 `429`，body 含 `reason:"quota"`，不转发上游。

### 场景 3：默认不限制

**Given** `homie.local.json` 无 `policy` 段（或 `policy` 为空）。  
**When** 任意虚拟 key 请求。  
**Then** 网关不限流、不限配额，行为与 PRD3 一致。

### 场景 4：审计可见

**Given** 策略启用。  
**When** 请求被限流/配额拒绝，或正常放行。  
**Then** 本地 SQLite 记录该判定事件（key_id、时间、是否拒绝、原因），可 `homie doctor`/
后续查询审计。

## 3. 功能需求

### FR-1: 策略配置加载

- `GatewayConfig` 新增可选 `policy: Option<Policy>`，`#[serde(default)]`。
- `Policy` 结构：
  ```rust
  struct Policy {
      rate_limit: Option<RateLimit>,   // requests_per_minute: u32
      quota: Option<Quota>,            // daily_token_limit: u64
  }
  ```
- 缺失 → `None` → 不启用任何限制。

### FR-2: 每 key 速率限制（内存滑动窗口）

- 以「当前自然分钟」为窗口，统计每个 key 在窗口内的请求数。
- 请求数 > `requests_per_minute` 时，在 `upstream.forward` 之前拒绝，返回 `429`。
- 窗口切换时计数清零（无需精确滑动，分钟粒度即可，符合最小实现）。

### FR-3: 每 key 配额（SQLite 聚合）

- 以「当前自然日（本地时区或 UTC，定死一种）」为周期，聚合该 key 当日 `SUM(input_tokens +
  output_tokens)`。
- 累计 > `daily_token_limit` 时拒绝，返回 `429`。
- 聚合仅读 `gateway_usage`，不新增表；放行请求的用量仍由既有 `UsageStore::record` 写入。

### FR-4: 429 结构化拒绝响应

- 命中限流/配额返回 `429`，`application/json` body：`{"error":{"type":"rate_limit_error"|
  "quota_error","message":"..."}}`（不携带 key/model/prompt）。
- 拒绝请求**不**写 `gateway_usage`（用量只记实际转发的请求）。

### FR-5: 审计事件记录

- 新增 `gateway_audit` 表（key_id、event、occurred_at、detail），记录 `allow` / `rate_limited`
  / `quota_exceeded` 事件。可选：为避免每请求写库，`allow` 事件抽样或关闭，至少记录拒绝事件。
  本 PRD 最小实现：仅记录**拒绝**事件（rate_limited / quota_exceeded），放行不额外落库。

### FR-6: 安全边界

- 拒绝响应与审计 detail 不含密钥、master key、虚拟 key 明文、model 之外的敏感 prompt。
- 策略判定发生在鉴权通过后、转发前，不改鉴权/虚拟 key 语义。

## 4. 实现方案

### 4.1 改动点

```text
homie/crates/homie-gateway/
├── src/config.rs     # Policy 反序列化 + 默认 None
├── src/state.rs      # AppState 增 policy + rate limiter 状态
├── src/policy.rs     # 新增：速率窗口 + 配额判定 + 429 响应构造（纯逻辑，可单测）
├── src/routes.rs     # forward_and_record 转发前调用 policy 判定
├── src/db.rs         # 新增 gateway_audit 表 + 拒绝事件写入
└── src/usage.rs      # 复用 record；policy 读 SUM 聚合
```

### 4.2 数据流

```text
agent → /v1/{responses|messages}
        → 鉴权通过（虚拟 key）
        → model 路由改写（PRD3）
        → policy.check_rate(key)   # 内存窗口，超限 → 429 + audit
        → policy.check_quota(key)  # SQLite SUM，超限 → 429 + audit
        → upstream.forward(path, body)
        → usage.record(...)        # 仅放行请求
```

### 4.3 核心类型（policy.rs）

```rust
pub struct RateLimiter { /* HashMap<key_id, (window_start, count)> */ }
impl RateLimiter {
    pub fn allow(&mut self, key: &str, rpm: u32, now: i64) -> bool;
}

pub struct QuotaChecker { /* 依赖 Db 读 SUM */ }
impl QuotaChecker {
    pub fn allow(&self, key: &str, daily_limit: u64, now: i64) -> rusqlite::Result<bool>;
}

pub enum DenyReason { RateLimit, Quota }
pub fn deny_response(reason: DenyReason) -> Response; // 429 + 结构化 body
```

## 5. 边界情况

| 场景 | 处理 |
|------|------|
| `policy` 整体缺失 | `None`，不启用限制，向后兼容 |
| `rate_limit` 缺失、仅 `quota` 存在 | 只做配额，不限流 |
| `quota` 缺失、仅 `rate_limit` 存在 | 只做限流，不配额 |
| `daily_token_limit = 0` | 语义为「配额为 0」，全部拒绝（或视为未配置，定死一种：0 视为未配置） |
| key 从未有用量记录 | SUM 为空 → 0，配额不触发 |
| 拒绝请求的用量 | 不写 `gateway_usage`，仅写 `gateway_audit` 拒绝事件 |

## 6. 涉及文件

- `homie/crates/homie-gateway/src/config.rs`
- `homie/crates/homie-gateway/src/state.rs`
- `homie/crates/homie-gateway/src/policy.rs`（新增）
- `homie/crates/homie-gateway/src/routes.rs`
- `homie/crates/homie-gateway/src/db.rs`
- `homie/crates/homie-gateway/src/main.rs`（传入 policy）
- `specs/llm-gateway.md`（新增 §Policy/Quota 合同）
- `docs/verification/llm-gateway-policy-quota/`（证据）

## 7. 验证计划

### 7.1 单元测试（Rust）

- `Policy` 反序列化：缺失为 `None`、部分字段、camelCase 对齐。
- `RateLimiter::allow`：窗口内计数、超限拒绝、窗口切换清零。
- `QuotaChecker::allow`：无记录、低于/高于阈值、聚合输入+输出。
- `deny_response`：429 状态码 + 结构化 body 不含敏感字段。

### 7.2 集成测试（tests/gateway.rs）

- 配置 `requests_per_minute = 1`，同一 key 第 2 次请求返回 `429` 且不转发（mock upstream 无
  第二次调用）。
- 配置 `daily_token_limit` 极小值，超限请求返回 `429`。
- 未配置 policy 时行为与 PRD3 一致（透传、正常用量记录）。

### 7.3 门禁

- `cargo fmt --all --check`
- `cargo test -p homie-gateway --offline`
- `cargo clippy -p homie-gateway --all-targets --offline`（干净）

## 8. 验收标准

1. `policy` 段可选加载，缺失不限制（向后兼容）。
2. 每 key 超过 `requests_per_minute` 返回 `429 rate_limit`，不转发上游。
3. 每 key 超过 `daily_token_limit` 返回 `429 quota`，不转发上游。
4. 拒绝事件写入本地 `gateway_audit`，放行请求用量照旧写 `gateway_usage`。
5. 拒绝响应与审计不含密钥/敏感 prompt。

## 9. Beads 追踪

- Beads: `homie-n6a`
- change_id: `llm-gateway-policy-quota`
- 类型: feature
- 优先级: P0
- 依赖: `llm-gateway-virtual-keys`（已✓，提供用量存储）、`llm-gateway-model-routing`（已✓，
  策略与路由同层）
- 后续 child Bead: `llm-gateway-credential-login`（凭证接入上游，需先做 design/research）、
  per-key 策略覆盖、限流可视化 UI
