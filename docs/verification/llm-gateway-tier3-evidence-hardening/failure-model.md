# LLM Gateway Failure Model

> change_id: `llm-gateway-tier3-evidence-hardening`
> 范围：`homie-gateway` 库 crate（虚拟密钥、上游转发、模型路由、策略/配额、凭据来源）。
> 口径：每条 failure mode 标注「伤害方式 → 触发条件 → 预期行为 → 捕获层 → 证据 → 状态」。
> 状态：`covered` = 有测试捕获；`known-limit` = 未覆盖，显式记录为 Tier-3 known limit（符合 standards §6.3）。

## 伤害方式总览

| ID | 伤害方式 | 状态 |
|----|---------|------|
| FM-01 | 凭据泄露（上游 key / master key / 原始虚拟密钥） | covered（+ FC-02 负向对照） |
| FM-02 | 虚拟密钥重放 / 未授权转发 | covered |
| FM-03 | 模型重写被畸形 body 绕过 | covered |
| FM-04 | 策略/配额被绕过（0 语义 / 边界） | covered + 2 known-limit |
| FM-05 | 并发竞态（rate-limit / quota / key 创建） | covered（rate-limit）+ 1 known-limit（quota 非原子） |
| FM-06 | 部分写入 / 数据丢失（SQLite） | covered（单语句原子）+ 1 known-limit（无 crash-replay） |
| FM-07 | 恶意输入（超大 body / 负 token / 畸形 JSON） | covered（畸形 JSON）+ 2 known-limit |
| FM-08 | 重启后密钥/配额状态不一致 | known-limit（无 reopen 测试） |

## 详细分析

### FM-01 凭据泄露

- **伤害方式**：上游 `api_key`、`master_key`、原始虚拟密钥落入日志、错误体、审计表或 agent env。
- **触发条件**：任何日志打印、错误响应体、`list()` 返回值、deny 响应体、usage/audit 行。
- **预期行为**：
  - 虚拟密钥只存 SHA-256 哈希（`auth::hash_key`），原始 key 仅在 `create()` 返回一次；
  - `list()` 返回 `ApiKeyRecord` 不含 raw key；
  - `deny_response` / `BAD_GATEWAY` 响应体不含 key/model/prompt；
  - 上游 key 仅由 `upstream::forward` 服务端附加，客户端 payload key 被忽略。
- **捕获层**：
  - `auth::tests::list_never_returns_raw_key`：断言序列化记录不含 raw key；
  - `policy::tests::deny_response_is_429_and_sanitized` + 集成测试断言 deny body 只含 `error.type/message`；
  - FC-02 负向对照：grep gate 对含 `sk-` 明文的坏样本必须命中、对源码必须零命中。
- **证据**：`fc-02-negative-control.log`、`fc-03-adversarial.log`。
- **状态**：`covered`。

### FM-02 虚拟密钥重放 / 未授权转发

- **伤害方式**：无效/已撤销密钥或伪造 `Authorization` 通过鉴权并转发到上游。
- **触发条件**：`Authorization: Bearer <bogus>` 或 `x-api-key`；已 `delete` 的密钥再次使用。
- **预期行为**：401，不触发上游调用，不写 usage。
- **捕获层**：
  - `bad_key_is_rejected_and_never_forwarded`（无 mock 挂载，转发会 404，断言 401）；
  - `revoked_key_returns_unauthorized`；
  - `extract_key` 的 `Bearer` 优先于 `x-api-key` 语义由 `resolve_caller` 保障（单测覆盖间接）。
- **证据**：`fc-03-adversarial.log`。
- **状态**：`covered`。

### FM-03 模型重写被畸形 body 绕过

- **伤害方式**：恶意/畸形 body 使模型重写崩溃、改写错误字段或把 gateway 配置的模型泄露成客户端模型。
- **触发条件**：非 JSON body、`model` 非字符串、映射缺失、映射值为空白。
- **预期行为**：pass-through 原样，不 panic、不重写。
- **捕获层**：
  - `apply_model_route_passes_through_non_json`；
  - `apply_model_route_passes_through_non_string_model`；
  - `apply_model_route_passes_through_blank_targets`；
  - `apply_model_route_passes_through_when_key_missing`。
- **证据**：`fc-01-baseline.log`（33 单元测试，含 apply_model_route 各用例）。
- **状态**：`covered`。

### FM-04 策略/配额被绕过

- **伤害方式**：`0` 语义被误判、每日配额跨自然日边界误算、token 聚合漏算 output。
- **触发条件**：`requests_per_minute=0` / `daily_token_limit=0`；跨 UTC 午夜；聚合公式错误。
- **预期行为**：`0` = 不限制；配额按 `SUM(input+output)` 且 `sum < limit` 才放行。
- **捕获层**：
  - `rate_limiter_zero_means_unconfigured`；
  - `quota_zero_means_unconfigured`；
  - `quota_checks_cumulative_tokens`（含 output 计入）；
  - FC-04 变异 M3（rate-limit off-by-one 被 `rate_limiter_allows_within_window` 杀死）。
- **known-limit**：
  - KL-04a：无显式跨 UTC 午夜（`day_start = now - now.rem_euclid(86400)`）边界测试；
  - KL-04b：负 token 注入（见 FM-07）会使 `sum as u64` 回绕为巨大值，导致误拒绝而非放行（DoS 方向，非配额绕过）。
- **证据**：`fc-01-baseline.log`、`fc-04f-restored.log`。
- **状态**：`covered` + 2 known-limit。

### FM-05 并发竞态

- **伤害方式**：rate-limit 窗口计数、quota 聚合、密钥创建在并发下产生竞态。
- **触发条件**：多请求并发。
- **预期行为**：rate-limit 计数不丢失；quota 不超发（理想）；密钥 id 唯一。
- **捕获层**：
  - `rate_limiter_allows_within_window` / `rate_limiter_resets_on_new_window`（单线程确定性）；
  - `random_hex_is_unique_and_sized`（唯一性）；
  - 结构性防护：`RateLimiter` 由 `Arc<Mutex<...>>` 包裹，`Db` 由 `Arc<Mutex<Connection>>` 串行化。
- **known-limit**：
  - KL-05a：`check_policy`（读 usage 聚合）与 `usage.record`（写 usage）是两次独立锁获取，quota 检查与记录之间非原子，并发下可轻微超发。与 `specs/llm-gateway.md`「usage 是估算、非权威计费」契约一致，接受为 known limit。
  - KL-05b：无并发 stress 测试（tokio 多任务同时打 `/v1/responses`）。
- **证据**：`fc-01-baseline.log`（单元段）。
- **状态**：`covered`（rate-limit）+ 2 known-limit。

### FM-06 部分写入 / 数据丢失

- **伤害方式**：usage/audit 半写、密钥表损坏。
- **触发条件**：进程在写中断电/崩溃。
- **预期行为**：单语句 INSERT 原子；WAL 模式。
- **捕获层**：
  - `db.rs` 开启 `journal_mode=WAL`；
  - `records_usage_per_key` 验证单次 record 原子写入；
  - 每次 `record` / `record_audit` 为单条 INSERT（无跨表事务）。
- **known-limit**：
  - KL-06a：无 crash-recovery / 关闭后重开数据库的持久化重放测试。
- **证据**：`fc-03-adversarial.log`（usage 行断言）。
- **状态**：`covered`（单语句原子）+ 1 known-limit。

### FM-07 恶意输入

- **伤害方式**：超大 body 压垮内存；负 token 注入破坏配额；畸形 JSON 导致 panic。
- **触发条件**：>64MB body；上游返回负 `usage.input_tokens/output_tokens`；畸形 JSON。
- **预期行为**：>64MB 返回 413；畸形 JSON 走 pass-through/零值；负 token 不崩溃。
- **捕获层**：
  - `apply_model_route_passes_through_non_json` / `extract_model` 对垃圾输入返回 `unknown`；
  - `extract_usage` 对非 JSON 返回 `(0,0)`（`usage_absent_is_zero`）。
- **known-limit**：
  - KL-07a：`DefaultBodyLimit::max(64MB)` 未做显式 413 测试（依赖 axum 框架层）；
  - KL-07b：`extract_usage` 用 `as_i64()` 接受负数，恶意上游可注入负 token → `sum as u64` 回绕导致误拒绝（与 KL-04b 同一根因）。
- **证据**：`fc-03-adversarial.log`。
- **状态**：`covered`（畸形 JSON）+ 2 known-limit。

### FM-08 重启后密钥/配额状态不一致

- **伤害方式**：重启后虚拟密钥丢失或配额/usage 丢失。
- **触发条件**：daemon 重启。
- **预期行为**：密钥、usage、audit 持久化于 SQLite，重启恢复。
- **捕获层**：无（无 reopen/重启测试）。
- **known-limit**：KL-08a：集成测试 `Harness` 用 `std::mem::forget(dir)` 保持 tempdir 存活，从不关闭再重开 DB，故无「重启恢复」证据。
- **状态**：`known-limit`。

## Known Limits 汇总

| ID | 描述 | 严重度 | 后续建议 |
|----|------|--------|---------|
| KL-04a | 无跨 UTC 午夜配额边界测试 | L | 补 `QuotaChecker` 跨日边界单测 |
| KL-04b/KL-07b | 负 token 注入 → `sum as u64` 回绕误拒绝 | M | `extract_usage` 用 `max(0, v)` 或 `try_into` 钳制 |
| KL-05a | quota 检查与 usage 记录非原子 | M | 将 quota 检查 + 记录放入同一事务/锁区间 |
| KL-05b | 无并发 stress 测试 | M | 补 tokio 并发测试 |
| KL-06a | 无 crash-recovery 测试 | L | 补关闭重开持久化测试 |
| KL-07a | 无 64MB 413 测试 | L | 补 oversized body 集成测试 |
| KL-08a | 无重启恢复测试 | M | 补 reopen 后密钥/usage 恢复测试 |

> 以上 known limits 均属于「加固后续项」，不在本 change 内修复（本 change 不改生产代码）。KL-04b/KL-07b 为同根因合并；如后续将其中任一升级为 bugfix，建议一并修复负 token 钳制。
