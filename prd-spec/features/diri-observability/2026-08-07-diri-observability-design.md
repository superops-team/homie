# Diri observability/event/evidence parity 第一阶段设计文档

## 1. 概述

### 1.1 背景

Homie 的 Diri 复刻需要让 runtime、client、CLI、LLM proxy、usage、UI 和 release evidence 使用同一套安全可观测性合同。当前 `specs/observability/README.md` 已定义 safe logging、events、metrics 和 evidence helpers 的方向，但仍停留在愿景层：没有明确 safe field whitelist，没有逐项承接 Diri `EventBus`/`events.subscribe`/`events.wait` 的事件 envelope 语义，也没有把 `metrics.write_failed`、usage token/cost/cache 字段和验证 evidence 绑定成可测试模型。

`docs/verification/diri-module-inventory/bingo-component-spec-review-report.md` 对 observability 的审查结论是 `partial`：需要补齐 Diri EventBus、DaemonLog、metrics write failure 的具体事件 catalog，建立全局 safe field whitelist，并补充日志/metrics schema fixtures。本 PRD 是 `lane-foundation-observability` 的第一阶段闭环，目标是先落地跨 lane 可依赖的最小规格与纯模型实现，不改 runtime 运行逻辑。

### 1.2 目标

- 将 `specs/observability/README.md` 从愿景级合同升级为可执行的第一阶段合同。
- 建立全局 safe field whitelist，明确允许进入 logs/events/metrics/evidence 的字段、危险字段处理和 fail-closed 行为。
- 对齐 Diri EventBus 的核心语义：事件名目录、`seq`、`session_id` 过滤、`events.dropped` gap marker、订阅/等待结果 envelope。
- 定义 `metrics.write_failed` 的安全字段和不阻断主流程语义。
- 定义 usage evidence projection：只允许 token、cache、cost、latency、provider/model/profile/session 等摘要字段，不允许 raw request/response/prompt/tool args。
- 以 `homie-observability` 纯模型 crate 建立第一阶段可复用接口和测试 fixtures。
- 记录 PRD、spec review、functional cases、OpenSpec、TDD 和验证证据，保持 Beads `homie-wm7` 可追溯。

### 1.3 非目标

- 不在本阶段修改 `homie-runtime`、`homie-llm`、`homie-client`、`homie-cli` 或 storage schema。
- 不实现真实事件总线、socket 订阅、SQLite metrics repository 或 LLM proxy 写入路径。
- 不改变 runtime supervisor 的 session 生命周期、PTY、holder、resource governor 或 status reducer 行为。
- 不追求与 Diri Swift/Rust 代码的 ABI 或文件格式兼容；本阶段只定义 Homie 可消费的安全事件/metrics/evidence模型。
- 不把任何 raw provider key、Authorization、cookie、private key、raw prompt、raw response、完整 tool args/result 写入测试 fixture 或文档样例。

## 2. 用户场景

### 场景 1: runtime 发布 session 事件

**Given** 后续 runtime lane 需要发布 `session.updated` 或 `session.output`。
**When** 事件进入 Homie 可观测性层。
**Then** 事件必须被规范化为 `{name, seq, session_id, fields}`，字段只能来自 safe whitelist，且任何危险字段必须被剔除或脱敏。

### 场景 2: 慢订阅者丢事件

**Given** Diri EventBus 会在慢订阅者队列溢出时丢弃旧事件。
**When** Homie 后续实现订阅回放或等待 API。
**Then** 可观测性合同必须保留 `events.dropped` gap marker，标明 `dropped`、`from_seq`、`to_seq`，且该 marker 不被 session/kind 过滤隐藏。

### 场景 3: metrics 写失败

**Given** LLM proxy 或 usage 统计完成了主请求。
**When** metrics repository 写入失败。
**Then** 主流程不能被失败阻断，但必须生成 `metrics.write_failed` 安全事件，包含 metrics kind、scope、safe error code 和 retryable，不包含 raw request/response 或 secret-bearing 参数。

### 场景 4: usage evidence 进入验证报告

**Given** Diri usage accounting 会从 Claude/Codex transcript 或 node usage ledger 产生 token/cost/cache 统计。
**When** Homie 生成 release evidence 或 UI usage summary。
**Then** 只能记录 provider、profile、session、model、input/output/cache tokens、estimated/billed cost、value kind、source、latency 等摘要字段，并用 whitelist 测试防止 raw prompt 或 raw tool args 泄漏。

### 场景 5: verification worker 读取 evidence

**Given** dev-loop 要求功能验证 Case 前置设计并后置执行。
**When** 其他 lane 的 worker 引用 observability evidence。
**Then** 每个 evidence item 必须有 gate status、command 或 source、exit code、output summary、evidence path 和 safe fields，未运行不能写成 pass。

## 3. 功能需求

### FR-1: Safe field whitelist

Homie 必须定义全局 safe field whitelist：

- Whitelist 按领域分组：common、event、session、runtime、metrics、usage、evidence、verification、agent hook safe summary。
- 允许字段必须是稳定、低敏、可索引的字段，例如 `event.name`、`event.seq`、`session.id`、`runtime.component`、`metrics.kind`、`usage.input_tokens`、`evidence.status`。
- 禁止字段包括但不限于 raw key、Authorization、cookie、private key、raw prompt、raw request body、raw response body、full tool args、full tool result、env、headers、home path secret fragments。
- 任何未知字段默认丢弃；字段名命中危险模式时必须丢弃或返回 `RedactionBlocked`，不得原样进入输出。
- 文档和测试必须证明 whitelist 是 allow-by-default 的反面：只有显式允许字段才保留。

### FR-2: Diri EventBus parity mapping

第一阶段必须把 Diri 事件合同映射到 Homie 可观测性模型：

- Event envelope 包含 `name`、`seq`、`session_id`、`fields`。
- Event name catalog 至少覆盖：`session.updated`、`session.resources`、`session.removed`、`project.updated`、`session.spawned`、`session.status`、`session.needs_input`、`session.output`、`session.artifact`、`session.archived`、`worktree.created`、`worktree.removed`、`events.dropped`、`metrics.write_failed`、`verification.functional_case_executed`。
- `seq` 从 1 开始单调递增；`events.dropped` 使用 `seq=0` 表示 synthetic marker，不参与客户端 last-seq 前进。
- session filter 不应接收无 session 的 project/worktree 事件，但 `events.dropped` 必须永远可见。
- kind filter 只接收指定 event names，但 `events.dropped` 必须永远可见。
- 事件字段必须先通过 safe whitelist 投影。

### FR-3: Metrics write failure contract

`metrics.write_failed` 必须是跨 LLM proxy、usage、runtime metrics 的统一失败事件：

- 主业务流程已经完成时，metrics 写失败不改变主流程结果。
- 事件字段只允许：`metrics.kind`、`metrics.scope`、`component`、`safe_error_code`、`retryable`、`occurred_at`、`operation`。
- 禁止记录 raw SQL、raw request、raw response、provider header、Authorization、secret-bearing argv/env。
- 功能验证必须包含一次模拟 metrics sink 失败，证明返回结果仍为成功，同时产生 safe failure event。

### FR-4: Usage evidence projection

Homie 必须定义 usage evidence 的安全投影：

- 对齐 Diri `UsageEvent`/`UsageTotals` 字段：provider、profile_id、session_id、model、input_tokens、output_tokens、cache_read_tokens、cache_write_tokens、estimated_usd、billed_usd、value_kind、source、occurred_at、updated_at、events。
- 允许后续 Homie LLM proxy 增加 first_token_latency_ms、total_latency_ms、tool_call_count、cache_hit_ratio 等摘要指标。
- 禁止 raw transcript line、prompt text、message body、tool args/result、provider request/response 进入 evidence。
- token 和 cost 必须校验非负；非有限 cost 不能进入 safe evidence。
- 测试必须覆盖 valid usage projection、negative token rejection、secret-bearing field stripping。

### FR-5: Evidence helper model

第一阶段必须定义 release/functional verification 可复用的 evidence model：

- `CommandEvidence` 或等价模型包含 command/source、exit_code、gate_status、output_summary、evidence_path、safe fields。
- Gate status 只允许 `pass`、`blocked`、`not_run`、`partial`、`fail`。
- evidence item 必须能记录 `verification.functional_case_executed` 安全事件。
- 未运行门禁必须写 `not_run`，不能被 formatter 或 helper 自动转成 pass。

### FR-6: Component spec and OpenSpec traceability

- `specs/observability/README.md` 必须增加 Diri EventBus/metrics/usage/evidence mapping、safe whitelist 和第一阶段验收引用。
- `docs/verification/diri-observability/` 必须包含 spec review、functional cases、TDD report、functional verification、code review 和 release readiness。
- `openspec/changes/diri-observability/` 必须包含 plan、tasks、alignment-report，并证明每个 FR 有 task 与 functional case。

## 4. 实现方案

### 4.1 规格层

更新 `specs/observability/README.md`：

- 增加 `Diri EventBus parity contract`。
- 增加 `Safe field whitelist` 表。
- 增加 `Metrics write failure contract`。
- 增加 `Usage evidence projection`。
- 增加 `Phase 1 verification gates`。

### 4.2 纯模型 crate

新建 `crates/homie-observability`，作为第一阶段 observability owner。该 crate 暂不接入 workspace 根，使用 `cargo test --manifest-path crates/homie-observability/Cargo.toml` 独立验证，避免改动其他 lane 的根 workspace 变更。

核心模块：

- `SafeFields`：按 whitelist 过滤 `serde_json::Value`。
- `SafeEvent`：事件 envelope、event catalog、filter semantics。
- `MetricsWriteFailure`：安全失败事件投影。
- `UsageEvidence`：usage event/totals 的安全投影与校验。
- `CommandEvidence`：功能验证和 release evidence 的状态模型。

### 4.3 TDD 路径

按纵向小步执行：

1. RED：写 safe field whitelist 测试；GREEN：实现 whitelist projector。
2. RED：写 EventBus filter/drop marker 测试；GREEN：实现 `SafeEvent::visible_to`。
3. RED：写 metrics sink failure projection 测试；GREEN：实现 `MetricsWriteFailure::to_event`。
4. RED：写 usage projection 测试；GREEN：实现 token/cost 校验与字段过滤。
5. RED：写 evidence gate status 测试；GREEN：实现 evidence model。

### 4.4 后续接入边界

本阶段完成后，其他 lane 必须通过 observability crate 或其规格消费安全字段，不应在 runtime/LLM/CLI 各自定义 ad hoc 日志字段。后续接入需要单独 PRD/OpenSpec，因为会触碰 runtime、LLM proxy、storage 或 client 协议边界。

## 5. 涉及文件

| 路径 | 类型 | 说明 |
|------|------|------|
| `prd-spec/features/diri-observability/2026-08-07-diri-observability-design.md` | 新增 | 中文 PRD |
| `specs/observability/README.md` | 更新 | 长期组件合同 |
| `docs/verification/diri-observability/*` | 新增 | spec review、functional cases、验证和评审证据 |
| `openspec/changes/diri-observability/*` | 新增 | OpenSpec plan/tasks/alignment |
| `crates/homie-observability/*` | 新增 | 第一阶段纯模型 crate 和测试 |

## 6. 组件 spec 影响

| 组件 spec | 影响 | 本阶段处理 |
|-----------|------|------------|
| `specs/observability/README.md` | 直接影响 | 更新 safe whitelist、event、metrics、usage、evidence 合同 |
| `specs/runtime-supervisor/README.md` | 只读依赖 | 不修改；后续 runtime lane 使用本合同 |
| `specs/llm-proxy/README.md` | 后续影响 | 本阶段不修改；后续 usage/metrics 接入时引用 |
| `specs/mcp-automation/README.md` | 后续影响 | 本阶段不修改；hook/MCP evidence 后续引用 |
| `specs/desktop-shell/README.md` | 后续影响 | 本阶段不修改；UI usage/evidence 展示后续引用 |

## 7. 边界情况

| 场景 | 处理方式 |
|------|----------|
| 未知字段进入日志/event | 默认丢弃 |
| 字段名命中 secret/header/prompt/tool args 危险模式 | 丢弃并可返回 redaction blocked 诊断 |
| `events.dropped` 被 kind/session filter 排除 | 禁止排除，必须可见 |
| usage token 为负数 | projection 返回错误，不产生 safe evidence |
| cost 为 NaN/Inf 或负数 | projection 返回错误，不产生 safe evidence |
| metrics write failure 本身的错误消息包含 secret | 只保留 safe error code，不保留原始 message |
| functional case 未执行 | evidence status 为 `not_run` 或 `blocked`，不能写 pass |

## 8. 测试计划

| 类型 | 覆盖点 | 命令 |
|------|--------|------|
| 单元测试 | safe field whitelist 保留允许字段、剔除 secret/raw prompt/tool args | `cargo test --manifest-path crates/homie-observability/Cargo.toml safe_field` |
| 单元测试 | event catalog、filter、`events.dropped` 永远可见 | `cargo test --manifest-path crates/homie-observability/Cargo.toml event` |
| 单元测试 | metrics write failure 不携带危险字段 | `cargo test --manifest-path crates/homie-observability/Cargo.toml metrics` |
| 单元测试 | usage projection 非负校验与字段过滤 | `cargo test --manifest-path crates/homie-observability/Cargo.toml usage` |
| 单元测试 | evidence gate status 不把未运行写成 pass | `cargo test --manifest-path crates/homie-observability/Cargo.toml evidence` |
| 格式 | Rust 格式 | `cargo fmt --manifest-path crates/homie-observability/Cargo.toml -- --check` |
| 静态检查 | Rust 编译与 clippy | `cargo check --manifest-path crates/homie-observability/Cargo.toml`; `cargo clippy --manifest-path crates/homie-observability/Cargo.toml --all-targets -- -D warnings` |
| 文档/安全 | diff 空白与 secret scan hook | `git diff --check`; `.githooks/pre-commit` 如环境允许 |

## 9. 验收标准

- PRD、spec review、functional cases、OpenSpec plan/tasks/alignment 均已写入目标路径。
- `specs/observability/README.md` 包含第一阶段 Diri parity mapping 和 safe whitelist。
- `homie-observability` crate 的纯模型测试通过。
- 功能验证报告逐条记录 FC-OBS-001 到 FC-OBS-006 的命令、结果和证据路径。
- release readiness report 不把未运行门禁写成 pass。
- 本阶段不修改其他 lane 文件，不改 runtime 行为，不标记 broader Diri parity 已完成。

## 10. Beads 跟踪

| 字段 | 值 |
|------|----|
| Bead | `homie-wm7` |
| Change ID | `diri-observability` |
| Lane | `lane-foundation-observability` |
| Spec ID | `prd-spec/features/diri-observability/2026-08-07-diri-observability-design.md` |
| 状态 | open，完成验证后再由维护者决定是否关闭 |
