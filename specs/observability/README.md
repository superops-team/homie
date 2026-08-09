# Observability 组件规格

## 1. 组件定位

`homie-observability` 定义 Homie 的 safe logging、events、metrics、trace、usage projection、evidence helpers 和 redaction rules。它为 runtime、LLM proxy、UI、MCP、remote/node、updater 提供一致的可观测性合同。

## 2. 来源需求映射

- PRD: `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`
- Gap-closure PRD: `prd-spec/features/diri-engine-migration/2026-08-06-diri-engine-migration-gap-closure-design.md`
- Observability PRD: `prd-spec/features/diri-observability/2026-08-07-diri-observability-design.md`
- OpenSpec: `openspec/changes/reference-parity-v1/`
- Gap-closure OpenSpec: `openspec/changes/diri-engine-migration/`
- Observability OpenSpec: `openspec/changes/diri-observability/`
- 功能验证: FC-001, FC-012, FC-013, FC-015, FC-017, FC-018
- Gap-closure 功能验证: FC-DIRI-002, FC-DIRI-005, FC-DIRI-009
- Observability 功能验证: FC-OBS-001, FC-OBS-002, FC-OBS-003, FC-OBS-004, FC-OBS-005, FC-OBS-006

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | all Homie crates | 写 structured logs/events/metrics |
| 下游 | `homie-storage` | 持久化 usage、metrics failures、audit |
| 下游 | verification docs | 输出 release evidence |

## 4. 职责边界

负责：

- safe field schema。
- redaction helpers。
- metrics write failure contract。
- evidence report templates。
- release readiness gate status vocabulary。
- runtime spawn/input/output failure 的安全事件词汇。
- hook/notify parser 失败和 status transition 的安全诊断边界。
- Diri EventBus envelope/filter/drop marker 的 Homie 合同。
- usage evidence projection 的安全字段集合。

不负责：

- 决定业务流程。
- 存储 raw logs。
- 运行测试命令本身。
- 在本组件内实现 runtime socket、LLM proxy metrics sink、SQLite repository 或 UI rendering。

## 5. 核心接口

```rust
pub trait Redactor {
    fn redact_text(&self, input: &str) -> RedactedText;
    fn redact_json(&self, input: &serde_json::Value) -> serde_json::Value;
}

pub trait EvidenceRecorder {
    fn record_command(&self, result: CommandEvidence) -> Result<(), EvidenceError>;
}
```

## 6. 数据模型

```rust
pub struct CommandEvidence {
    pub command: String,
    pub exit_code: Option<i32>,
    pub status: GateStatus,
    pub output_summary: String,
    pub evidence_path: PathBuf,
    pub fields: SafeFields,
}

pub enum GateStatus {
    Pass,
    Blocked,
    NotRun,
    Partial,
    Fail,
}
```

Phase 1 safe event model:

```rust
pub struct SafeEvent {
    pub name: EventName,
    pub seq: u64,
    pub session_id: Option<String>,
    pub fields: SafeFields,
}

pub struct MetricsWriteFailure {
    pub metrics_kind: String,
    pub metrics_scope: String,
    pub component: String,
    pub operation: String,
    pub safe_error_code: String,
    pub retryable: bool,
    pub occurred_at: i64,
}

pub struct UsageEvidence {
    pub provider: String,
    pub profile_id: Option<String>,
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub estimated_usd: Option<f64>,
    pub billed_usd: Option<f64>,
    pub value_kind: String,
    pub source: String,
    pub occurred_at: i64,
}
```

## 7. 运行模型与状态机

```text
operation starts
  -> emit safe started event
  -> collect safe metrics
  -> write evidence/metrics
  -> on failure emit safe failed event
```

Gap-closure events:

- runtime.spawn_failed。
- runtime.session_not_live。
- runtime.output_log_write_failed。
- agent.hook_parsed。
- agent.hook_parse_failed。
- agent.status_transitioned。
- verification.functional_case_executed。

Diri EventBus parity events:

- session.updated。
- session.resources。
- session.removed。
- project.updated。
- session.spawned。
- session.status。
- session.needs_input。
- session.output。
- session.artifact。
- session.archived。
- worktree.created。
- worktree.removed。
- events.dropped。
- metrics.write_failed。
- verification.functional_case_executed。

EventBus contract:

- `seq` 从 1 开始单调递增。
- `events.dropped` 是 synthetic marker，`seq=0`，不推进客户端 last-seq。
- session filter 只接收带匹配 `session_id` 的 session 事件；project/worktree 这类无 session 事件不进入 session filter。
- kind filter 只接收指定 event names。
- `events.dropped` 不受 session/kind filter 排除，慢订阅者必须知道自己看到的 slice 有 gap。
- 所有 event fields 在发布前必须经过 safe field whitelist projection。

## 8. 安全与权限

- raw key、Authorization、cookie、private key、raw prompt、完整 tool args/result 禁止进入 logs/events/metrics/report。
- redaction failure fail closed。
- command evidence 可记录命令结构，但 secret-bearing 参数必须同长度掩码或替换。
- hook/notify payload 只允许记录解析后的稳定事件类型、agent kind、session id、safe error code 和脱敏摘要。
- spawn failure 只允许记录 binary/cwd 的安全摘要和错误类别，不记录完整 env 或 secret-bearing argv。

Safe field whitelist:

| 分组 | 允许字段 |
|------|----------|
| common | `component`, `operation`, `safe_error_code`, `retryable`, `occurred_at`, `duration_ms` |
| event | `event.name`, `event.seq`, `event.kind`, `event.from_seq`, `event.to_seq`, `event.dropped` |
| session | `session.id`, `session.status`, `session.from_status`, `session.to_status`, `session.needs_input_kind`, `session.content_seq` |
| runtime | `runtime.binary`, `runtime.cwd_summary`, `runtime.exit_code`, `runtime.output_offset`, `runtime.cols`, `runtime.rows` |
| metrics | `metrics.kind`, `metrics.scope`, `metrics.value`, `metrics.unit`, `metrics.count` |
| usage | `usage.provider`, `usage.profile_id`, `usage.session_id`, `usage.model`, `usage.input_tokens`, `usage.output_tokens`, `usage.cache_read_tokens`, `usage.cache_write_tokens`, `usage.estimated_usd`, `usage.billed_usd`, `usage.value_kind`, `usage.source`, `usage.first_token_latency_ms`, `usage.total_latency_ms`, `usage.tool_call_count`, `usage.cache_hit_ratio` |
| evidence | `evidence.command`, `evidence.source`, `evidence.exit_code`, `evidence.status`, `evidence.output_summary`, `evidence.path`, `evidence.case_id` |
| agent hook safe summary | `agent.kind`, `agent.event_type`, `agent.is_subagent`, `agent.blocker_kind`, `agent.risk_level` |

Dangerous fields are never safe, even if nested under an otherwise safe object:

- `authorization`, `cookie`, `set_cookie`, `api_key`, `provider_key`, `private_key`, `token`, `secret`, `password`。
- `raw_prompt`, `prompt`, `raw_request`, `raw_response`, `request_body`, `response_body`, `headers`。
- `tool_args`, `tool_result`, `full_tool_args`, `full_tool_result`。
- `env`, `argv`, `stdin`, `stdout`, `stderr` when they contain unstructured or secret-bearing data。
- unknown fields: default drop.

## 9. 可观测性

此组件定义全局可观测性，不再依赖下游组件提供自定义状态词。所有 release report 使用 pass/blocked/not_run/partial/fail。

Metrics write failure contract:

- 主流程已经完成时，metrics 写失败不得改变主流程结果。
- 只发布 `metrics.write_failed` safe event。
- 允许字段：`metrics.kind`, `metrics.scope`, `component`, `operation`, `safe_error_code`, `retryable`, `occurred_at`。
- 禁止 raw SQL、raw provider request/response、headers、Authorization、secret-bearing argv/env。

Usage evidence projection:

- 对齐 Diri `UsageEvent`/`UsageTotals` 的摘要字段：provider、profile_id、session_id、model、input/output/cache tokens、estimated/billed cost、value_kind、source、occurred_at、updated_at、events。
- Homie LLM proxy 后续可添加 first-token latency、total latency、tool call count、cache hit ratio 等摘要字段。
- token/cost 必须非负；cost 必须 finite。
- raw transcript line、prompt/message body、tool args/result、provider request/response 不进入 evidence。

## 10. 失败与恢复

| 场景 | 行为 |
|------|------|
| metrics 写失败 | 不阻断主流程，记录 `metrics.write_failed` |
| evidence 写失败 | gate 标记 blocked |
| redaction 规则不匹配 | fail closed 或移除字段 |
| hook payload 解析失败 | fail-open 继续 agent 流程，只记录 safe summary |
| functional case 未执行 | release readiness 标记 blocked，不允许写 pass |

## 11. 测试计划与验收引用

- FC-001: 文档和路径扫描。
- FC-012: browser/test artifact evidence。
- FC-013: LLM metrics no leak。
- FC-015: remote handoff no credential evidence。
- FC-017: release readiness。
- FC-018: full local quality gate。
- FC-DIRI-002: spawn failure safe evidence and no half-created session。
- FC-DIRI-005: hook parser redaction。
- FC-DIRI-009: OpenSpec/evidence state consistency。
- FC-OBS-001: safe field whitelist。
- FC-OBS-002: Diri EventBus envelope/filter/drop marker。
- FC-OBS-003: metrics.write_failed safe projection。
- FC-OBS-004: usage evidence projection。
- FC-OBS-005: evidence helper gate status honesty。
- FC-OBS-006: PRD/spec/OpenSpec/evidence traceability。

## Diri Parity Mapping

This section is mandatory for Diri-to-Homie parity planning and fixes the review gap recorded in `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`.

| Field | Value |
|-------|-------|
| Owned feature atoms | M14-F002, M19-F001, M19-F002, cross-cutting evidence |
| Required Diri test mapping | EventSubscriptionTests, event schema, metrics failure, redaction whitelist tests, usage fixture tests |
| Pre-implementation gaps | safe field whitelist, Diri event mapping, metrics.write_failed projection, usage evidence projection |
| Phase 1 owner | `prd-spec/features/diri-observability/2026-08-07-diri-observability-design.md` and `openspec/changes/diri-observability/` |

Rules:

- A PRD/OpenSpec change touching this component must cite the owned feature atom ids above.
- Implementation is not complete until the required Diri test mapping has an equivalent Homie verification gate.
- If a new Diri source/test is discovered for this component, update `docs/research/diri-module-inventory.md` and this section before coding.

## 12. Diri 7ba3407 重基线修订

权威来源：

- PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`
- 能力矩阵: `docs/research/diri-7ba3407-capability-matrix.md`
- Requirements: FR-01, FR-13, FR-15, FR-16
- Beads: `homie-t3u`

### 12.1 Evidence 状态合同

所有当前和后续 evidence frontmatter、表格结果和 gate decision 只能使用：

- `pass`
- `blocked`
- `not_run`
- `partial`
- `fail`

`pass_with_scope_limit`、`pass_with_note`、`ready_for_next_loopx_slice`、`pass_with_screenshot_blocker` 等状态非法。带限制的结果必须写成 `partial` 或 `blocked`，并在 reason 中说明范围。

历史 evidence 保留原文，但新的 release gate 不得把非法历史状态计为 pass。Wave 0 必须生成状态审计清单；后续按 owning change 修正或由新 evidence 明确 supersede。

### 12.2 整体与切片准出

- 单个 parser、DTO、UI surface 或 wave 的 `pass` 只表示该 change 通过。
- Diri parity overall 由 `docs/research/diri-7ba3407-capability-matrix.md` 决定。
- 任一必要能力不是 `implemented` 时，整体必须报告 `partial` 或 `blocked`。
- `not_run` 不能通过文字说明提升为 pass。
- source-text test、文档扫描和 catalog count 不能替代 runtime/product E2E。

### 12.3 完成门禁

- evidence schema validator 拒绝非法状态词和缺失 command/exit code/reason。
- capability matrix、Beads、OpenSpec task 和 evidence path 可双向追踪。
- runtime/LLM/MCP/remote/updater 的安全事件通过 hostile-field tests。
- final parity report 自动确认没有 incomplete capability、非法状态和 not-run gate。

## 13. Wave 1A Transport Evidence 修订

权威来源：

- PRD: `prd-spec/features/diri-runtime-daemon-client-transport/2026-08-08-diri-runtime-daemon-client-transport-design.md`
- OpenSpec: `openspec/changes/diri-runtime-daemon-client-transport/`
- Beads: `homie-nep`

Wave 1A 允许记录：

- daemon instance id、pid、endpoint kind；
- client role、connection state、method、stream kind/id；
- frame kind、sequence、safe output offset、queue depth；
- retry count/backoff、duration、safe error code。

Wave 1A 禁止记录：

- raw frame/control payload；
- terminal input/output bytes；
- argv/env；
- Authorization/cookie/provider key/virtual key；
- raw tool args/result。

必须覆盖 safe events：

- `daemon.starting/ready/draining/stopped/start_failed`
- `client.connect_started/connected/disconnected/reconnect_scheduled`
- `client.handshake_failed`
- `client.event_gap`
- `client.stream_reset/resynced`
- `client.backpressure`
- `runtime.actor_backpressure`

event gap、slow consumer 和 daemon restart 的 evidence 必须同时记录 command、exit code、expected/actual、safe cursor/offset 和最终 `pass/blocked/not_run/partial/fail`。holder adoption 仍失败时必须引用 T-102 blocker，不能把 transport pass 提升为完整 runtime pass。
