# Session Context Store 组件规格

## 1. 组件定位

`homie-context` 维护 agent session、conversation、workspace facts、lineage、artifact summary、safe prompt/tool metadata、task/memory references 和 session summary。它是 UI、runtime、task、memory 和 orchestrator 共享上下文的事实边界。

## 2. 来源需求映射

- PRD: `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`
- Gap-closure PRD: `prd-spec/features/diri-engine-migration/2026-08-06-diri-engine-migration-gap-closure-design.md`
- OpenSpec: `openspec/changes/reference-parity-v1/`
- Gap-closure OpenSpec: `openspec/changes/diri-engine-migration/`
- 功能验证: FC-007, FC-011, FC-016, FC-018
- Gap-closure 功能验证: FC-DIRI-001, FC-DIRI-002, FC-DIRI-003, FC-DIRI-006

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | `homie-runtime` | 写入 session events、lineage、artifact refs |
| 上游 | `homie-app` | 经 client/runtime service 读取 summary 和 inspector projection |
| 上游 | `homie-task`, `homie-memory` | 绑定 task/memory references |
| 下游 | `homie-storage` | 由 runtime owning service 持久化 context events and summaries |

## 4. 职责边界

负责：

- session context event append/read。
- session summary projection。
- parent/child lineage。
- history resume metadata。
- task/memory/artifact safe references。
- live session status/output index 的安全摘要投影，供 UI、history 和 inspector 读取。

不负责：

- raw output log 存储。
- live PTY registry 或 PTY 写入。
- 长期 memory 检索排序。
- task 状态机。
- provider credential。
- runtime holder/process live 状态判定。
- remote handoff 和 updater 工作流。

## 5. 核心接口

```rust
pub trait SessionContextStore {
    fn append_event(&self, event: ContextEvent) -> Result<(), ContextError>;
    fn summary(&self, session_id: SessionId) -> Result<SessionContextSummary, ContextError>;
    fn lineage(&self, session_id: SessionId) -> Result<SessionLineage, ContextError>;
}
```

## 6. 数据模型

```rust
pub struct ContextEvent {
    pub id: ContextEventId,
    pub session_id: SessionId,
    pub kind: ContextEventKind,
    pub safe_payload: serde_json::Value,
    pub source: ContextSource,
    pub created_at: OffsetDateTime,
}
```

Forbidden payload fields:

- raw provider key。
- raw Authorization/cookie。
- raw prompt unless explicitly redacted and classified。
- complete tool args/result。

Gap-closure session/output semantics:

- Live PTY bytes 写入 runtime output log，不写入 context event payload。
- Context 只可引用 output log offset、tail offset、status、title、cwd、agent kind、safe artifact summary 等安全摘要。
- Spawn 失败时不得创建 context root 或留下 `created` 状态 summary；失败证据只记录 safe error code。
- Runtime restart 后，如果 live PTY 不存在，context/history 可继续读取历史 output 摘要，但 live input 操作必须返回明确不可用状态。
- `sessions.parent_session_id` 是 direct parent 的唯一事实源；context 不维护第二套 parent graph。
- durable recovery row 中的 PID/status/instance 是恢复提示，不是 context 可声明 live 的证据。

## 7. 运行模型与状态机

```text
session created
  -> context root created
  -> runtime/events append facts
  -> summary projection updates
  -> UI/task/memory/orchestrator consume safe summary
```

## 8. 安全与权限

- context 写入前必须通过 redaction policy。
- memory write candidate 只能引用 context event id，不复制敏感原文。
- task projection 只暴露必要 task state，不泄漏 tool payload。

## 9. 可观测性

- context.event_appended。
- context.summary_updated。
- context.redaction_applied。
- context.write_failed。
- context.session_status_projected。
- context.output_index_updated。

## 10. 失败与恢复

| 场景 | 行为 |
|------|------|
| context 写失败 | runtime 继续运行，事件标记 degraded |
| summary projection 失败 | 可由 events 重建 |
| redaction policy 失败 | fail closed，不写入 |

## 11. 测试计划与验收引用

- FC-007: session lifecycle context。
- FC-011: history resume metadata。
- FC-016: context/memory/task/orchestration。
- FC-018: full local quality gate。
- FC-DIRI-001: live PTY output can be read through runtime output path。
- FC-DIRI-002: failed spawn leaves no half-created context/session summary。
- FC-DIRI-003: non-live input is not represented as successful context activity。
- FC-DIRI-006: scrollback/history reads rely on explicit row/offset metadata。

## Diri Parity Mapping

This section is mandatory for Diri-to-Homie parity planning and fixes the review gap recorded in `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`.

| Field | Value |
|-------|-------|
| Owned feature atoms | M05-F002, M17-F001 session context subset |
| Required Diri test mapping | HistoryScannerTests and transcript resume negative cases |
| Pre-implementation gaps | transcript/history mapping |

Rules:

- A PRD/OpenSpec change touching this component must cite the owned feature atom ids above.
- Implementation is not complete until the required Diri test mapping has an equivalent Homie verification gate.
- If a new Diri source/test is discovered for this component, update `docs/research/diri-module-inventory.md` and this section before coding.

## 12. Diri 7ba3407 重基线修订

权威来源：

- PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`
- Requirements: FR-11, FR-14, FR-16
- Beads: `homie-t3u`

本组件属于 Homie 扩展，当前状态是 `partial`。纯 `SessionContextSummary`/redaction 单测不能证明产品接入完成。

强制合同：

- context root 只能随 runtime 成功创建 session 后提交。
- runtime event、lineage、artifact、task、memory 和 output offset 通过 repository 持久化。
- MCP `whoami`、children、`summarize_children`、`report_to_parent` 必须读取同一 lineage/context 事实。
- UI inspector/history 和 orchestrator 只消费 safe summary，不复制 raw PTY/output/tool payload。
- projection 必须可从 durable events 重建，并处理 event gap/duplicate。

完成门禁：

- runtime spawn -> context events -> MCP lineage -> UI/CLI summary 真实 E2E。
- failed spawn、duplicate event、redaction failure 和 storage failure 的一致性测试。
- raw key、Authorization、raw prompt、raw output、完整 tool args/result 的 context fixture scan 为零。

## 13. T-103 Durable Lineage Foundation 修订

权威来源：

- PRD:
  `prd-spec/features/diri-storage-core-facts/2026-08-09-diri-storage-core-facts-design.md`
- OpenSpec: `openspec/changes/diri-storage-core-facts/`
- Bead: `homie-t3u.2`
- Master task: `T-103`

### 13.1 事实 ownership

- runtime/context domain service 是 durable context/lineage repository 的 production owner；
  app、CLI 和 MCP adapter 不直接打开 storage。
- `sessions.parent_session_id` 继续作为 direct parent 单一事实源。parent、children、
  context summary 和后续 MCP lineage projection 必须从同一关系事实读取。
- session、parent 关系和 frozen effective config 创建必须具备单 transaction 语义；失败
  不得留下半创建 context root、orphan config 或不一致 lineage。
- full recursive authorization、`summarize_children`、`report_to_parent` 和 UI inspector E2E
  仍由后续 change 完成；T-103 只提供 durable foundation。

### 13.2 Safe lineage audit

Lineage audit 至少包含：

- 唯一 operation id；
- actor session/service identity；
- subject session；
- relation/action；
- decision；
- safe reason code；
- created timestamp。

重复 operation id 必须幂等返回原结果或 stable conflict，不得重复追加。audit 禁止保存 raw
prompt/output、完整 tool args/result、provider key、virtual key material、Authorization 或
cookie。

### 13.3 Durable event/provenance 规则

- context event 和 lineage audit 必须具有 stable id/source reference，支持 duplicate
  rejection 和 gap 检测。
- output/checkpoint 只以 path/hash/offset/epoch/sequence safe reference 进入 context
  projection；terminal bytes、grid 和 checkpoint blob 不复制到 context。
- recovery 完成后，context projection 只消费 runtime 已验证并提交的 authoritative
  assessment，不直接把 storage 的 `last_observed_status` 投影为 running。
- projection rebuild 必须处理 unknown event/snapshot version、corrupt safe JSON、
  duplicate operation 和缺失 parent，均 fail closed。

### 13.4 验收边界

T-103 必须验证：

- parent/children repository 一致；
- lineage audit operation id 幂等；
- session/parent/effective config 原子提交；
- restart 后 safe lineage/context facts 可读；
- sensitive fixture scan 为零。

这些验证不单独证明 MCP lineage workflow、UI inspector/history、remote handoff 或完整
context/task/memory 产品接线完成；本组件状态继续保持 `partial`，直到对应 E2E 通过。
