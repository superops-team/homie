# LLM Proxy 组件规格

## 1. 组件定位

`homie-llm` 提供 Homie 的 OpenAI-compatible 本机代理、provider routing、model alias、streaming 转发、usage/cost/cache/tool metrics 和 safe error mapping。Managed agent 不直接访问真实 provider key，只访问 Homie local proxy。

## 2. 来源需求映射

- PRD: `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`
- OpenSpec: `openspec/changes/reference-parity-v1/`
- 功能验证: FC-013, FC-018

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | managed agent | 通过 virtual key 调 OpenAI-compatible endpoint |
| 上游 | `homie-runtime` | session spawn 时请求 proxy config |
| 下游 | provider HTTP APIs | 转发 chat/completions/responses 等请求 |
| 下游 | `homie-storage` | 记录 usage、pricing snapshot、metrics |
| 下游 | `homie-observability` | safe metrics/events |

## 4. 职责边界

负责：

- virtual key 鉴权后的 provider/model route。
- OpenAI-compatible request/response/streaming passthrough。
- token usage、cache read/write、latency、cost、tool metrics。
- provider failure safe error mapping。
- metrics write failure 不阻塞响应。

不负责：

- provider raw key 加密存储格式。
- agent process 启动。
- UI 展示。
- durable memory 写入策略。

## 5. 核心接口

```rust
pub trait LlmProxy {
    async fn issue_session_proxy_config(
        &self,
        session_id: SessionId,
        profile_id: AgentProfileId,
    ) -> Result<ManagedLlmProxyConfig, LlmProxyError>;

    async fn handle_openai_request(
        &self,
        request: ProxyRequest,
    ) -> Result<ProxyResponse, LlmProxyError>;
}

pub struct ManagedLlmProxyConfig {
    pub base_url: String,
    pub virtual_key: SecretString,
    pub expires_at: OffsetDateTime,
}
```

## 6. 数据模型

```rust
pub struct UsageRecord {
    pub request_id: RequestId,
    pub session_id: SessionId,
    pub provider_id: ProviderId,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub first_token_ms: Option<i64>,
    pub total_ms: i64,
    pub estimated_cost: Decimal,
    pub pricing_snapshot_id: PricingSnapshotId,
}
```

### 6.1 Diri-compatible API-equivalent pricing

`homie-llm` owns the local API-equivalent pricing helper used by transcript fallback and proxy usage estimates:

- Claude/OpenAI model matching follows the Diri `diri-usage` specific-to-generic order.
- `estimated_cost` is explicitly an estimate and must not be labeled as provider billed spend.
- Cache read uses `input * 0.1`.
- Claude cache write 5m uses `input * 1.25`.
- Claude cache write 1h uses `input * 2.0`.
- Unknown models return no estimate instead of inventing a price.

### 6.2 Transcript usage parser

`homie-llm` exposes a pure Claude/Codex transcript parser that turns JSONL files into neutral usage events before storage import:

- The parser tolerates bad JSON and non-usage lines by skipping them.
- Claude assistant `message.usage` rows preserve input/output/cache read/cache write and cache write 5m/1h fields.
- Codex `token_count` rows preserve the current model from `session_meta` or `turn_context`.
- Parser output is not persisted by itself; a later importer maps it into `homie-storage::RecordUsage`.
- Watchers, offset cache, fleet merge, and UI projection are separate lanes.

## 7. 运行模型与状态机

```text
agent request
  -> validate virtual key
  -> resolve provider/model/policy
  -> inject provider auth in-memory
  -> forward request/stream
  -> collect safe usage
  -> write metrics
  -> return provider-compatible response
```

## 8. 安全与权限

- raw provider key 只在 secret resolver 和短期内存出现。
- virtual key 必须绑定 session/profile/provider/model scope。
- revoked/expired/wrong-scope virtual key fail closed。
- logs/events/metrics 不保存 raw prompt、raw response、Authorization、cookie、完整 tool args/result。
- provider error body 返回前必须 safe mapping。

## 9. 可观测性

事件：

- llm.request.started。
- llm.request.completed。
- llm.request.failed。
- metrics.write_failed。

指标：

- request count、latency、token usage、cache hit、cost、provider error code。

## 10. 失败与恢复

| 场景 | 行为 |
|------|------|
| provider timeout | 返回 retryable safe error |
| metrics 写失败 | 响应不中断，记录 `metrics.write_failed` |
| virtual key 过期 | 401 safe error |
| provider auth 失败 | safe error，不泄漏 upstream body/header |

## 11. 测试计划与验收引用

- FC-013: usage、cost、LLM proxy custody。
- FC-018: full local quality gate。

## Diri Parity Mapping

This section is mandatory for Diri-to-Homie parity planning and fixes the review gap recorded in `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`.

| Field | Value |
|-------|-------|
| Owned feature atoms | M19-F001 plus Homie virtual-key proxy behavior |
| Required Diri test mapping | usage/pricing/token fixtures and remote raw-key denial |
| Pre-implementation gaps | usage accounting mapping |

Rules:

- A PRD/OpenSpec change touching this component must cite the owned feature atom ids above.
- Implementation is not complete until the required Diri test mapping has an equivalent Homie verification gate.
- If a new Diri source/test is discovered for this component, update `docs/research/diri-module-inventory.md` and this section before coding.

## 12. Diri 7ba3407 重基线修订

权威来源：

- PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`
- 能力矩阵: `docs/research/diri-7ba3407-capability-matrix.md`
- Requirements: FR-13, FR-16
- Beads: `homie-t3u`

本组件当前状态是 `partial`。pricing、transcript parser 和 in-memory virtual key tests 已存在，但当前 crate 没有 HTTP server、provider client、SSE forwarding 或 upstream credential injection，不能声明 LLM proxy 已实现。

### 12.1 HTTP Proxy 合同

- 使用项目已批准的 Tokio/Axum/Tower/Reqwest 技术栈，不自研 HTTP/SSE parser。
- 至少提供 OpenAI-compatible health/model/request routes，以及 agent profile 所需的 chat/completions/responses 子集。
- request 在进入 provider 前完成 virtual key validation、provider/model policy 和 route。
- upstream Authorization 只在单次请求短期内存注入。
- streaming 必须保留 chunk ordering、finish/error semantics 和 cancel。
- provider response/error 在返回 agent 前执行 compatible safe mapping，不泄漏 upstream header/body 中的 secret。

### 12.2 Usage 合同

- proxy usage、Claude/Codex transcript fallback 和 fleet usage 都写入同一 neutral ledger。
- transcript scanner 必须按 saved offset、inode/device、size、modified time 和 tail hash 增量处理 rotation/truncation。
- pricing 必须保存 snapshot；estimated 与 billed 分离。
- usage UI/CLI 只展示 safe aggregate，不读取 raw transcript 内容。
- metrics write failure 不能改变已经成功的 provider response，只发布 `metrics.write_failed`。

### 12.3 完成门禁

- fake provider 覆盖 success、stream、timeout、429、401、malformed chunk、client cancel。
- virtual key 的 expired/revoked/wrong session/profile/provider/model 全部拒绝。
- raw provider key、Authorization、request/response body 不进入 log/event/storage/context/memory/evidence。
- incremental transcript watcher、pricing snapshot、fleet merge、CLI 和 UI 通过 E2E。
- managed agent 通过真实 local proxy endpoint 完成至少一个 fake-provider streaming session。
