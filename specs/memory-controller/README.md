# Memory Controller 组件规格

## 1. 组件定位

`homie-memory` 管理 durable memory 的候选写入、来源引用、权限、检索边界和回收策略。Reference parity 首版只要求可追踪、安全的 memory candidate，不要求完整语义向量检索。

## 2. 来源需求映射

- PRD: `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`
- OpenSpec: `openspec/changes/reference-parity-v1/`
- 功能验证: FC-016, FC-018

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | `session-context-store` | 从 safe context event 产生 memory candidate |
| 上游 | `intent-orchestrator` | 检索或提交 memory candidate |
| 下游 | `homie-storage` | 持久化 memory records 和 source refs |

## 4. 职责边界

负责：

- memory write candidate。
- source attribution。
- redaction verification。
- permission-aware retrieval boundary。

不负责：

- raw transcript 保存。
- embedding/vector backend 选型，除非后续 PRD 启用。
- task 状态机。

## 5. 核心接口

```rust
pub trait MemoryController {
    fn write_candidate(&self, request: MemoryCandidateRequest) -> Result<MemoryCandidate, MemoryError>;
    fn approve_candidate(&self, id: MemoryCandidateId) -> Result<MemoryRecord, MemoryError>;
    fn search(&self, request: MemorySearchRequest) -> Result<Vec<MemoryRecord>, MemoryError>;
}
```

## 6. 数据模型

```rust
pub struct MemoryCandidate {
    pub id: MemoryCandidateId,
    pub source_event_id: ContextEventId,
    pub content: String,
    pub sensitivity: Sensitivity,
    pub status: MemoryCandidateStatus,
}
```

## 7. 运行模型与状态机

```text
candidate_created -> approved -> active
candidate_created -> rejected
active -> archived
```

## 8. 安全与权限

- candidate content 必须来自 redacted safe context。
- raw provider key、Authorization、cookie、raw prompt、完整 tool args/result 禁止写入。
- retrieval 必须按 workspace/profile/session 权限过滤。

## 9. 可观测性

- memory.candidate_created。
- memory.candidate_rejected。
- memory.record_approved。
- memory.search_performed。

## 10. 失败与恢复

| 场景 | 行为 |
|------|------|
| source event 不存在 | reject candidate |
| redaction 未通过 | fail closed |
| search backend 不可用 | 返回 degraded empty result 和 safe error |

## 11. 测试计划与验收引用

- FC-016: memory write candidate requires source and redaction。
- FC-018: full local quality gate。

## Diri Parity Mapping

This section is mandatory for Diri-to-Homie parity planning and fixes the review gap recorded in `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`.

| Field | Value |
|-------|-------|
| Owned feature atoms | Homie extension supporting M05/M19 context enrichment |
| Required Diri test mapping | memory candidate lifecycle tests |
| Pre-implementation gaps | mark as extension; redaction/source gates |

Rules:

- A PRD/OpenSpec change touching this component must cite the owned feature atom ids above.
- Implementation is not complete until the required Diri test mapping has an equivalent Homie verification gate.
- If a new Diri source/test is discovered for this component, update `docs/research/diri-module-inventory.md` and this section before coding.

## 12. Diri 7ba3407 重基线修订

权威来源：

- PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`
- Requirements: FR-14, FR-16
- Beads: `homie-t3u`

本组件属于 Homie 扩展，当前状态是 `partial`。candidate validation 单测存在，但 durable repository、permission-aware retrieval 和产品入口尚未形成闭环。

强制合同：

- candidate 必须引用已持久化 safe context event，不能直接接收 raw transcript。
- approve/reject/archive/search 通过 storage repository，并保留 source attribution。
- retrieval 按 workspace、profile、session 和 permission profile 过滤。
- 首个纵向切片使用结构化/文本检索即可；没有明确 PRD 前不引入 embedding/vector backend。
- memory 失败不改变 runtime/task 主流程，只返回 degraded safe result。

完成门禁：

- context event -> candidate -> approve/reject -> permission-filtered search -> UI/CLI/MCP consumer E2E。
- missing source、redaction denial、cross-workspace denial、storage failure 和 archive 测试。
- memory corpus/evidence 的 secret/raw-payload scan 为零。
