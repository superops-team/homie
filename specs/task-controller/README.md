# Task Controller 组件规格

## 1. 组件定位

`homie-task` 维护 Homie 的任务事实源。任务不归属于单一 agent；用户、orchestrator 和 agent session 都可以创建、领取、更新、阻塞或交还任务。

## 2. 来源需求映射

- PRD: `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`
- OpenSpec: `openspec/changes/reference-parity-v1/`
- 功能验证: FC-016, FC-018

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | `homie-app` | 展示和操作任务 |
| 上游 | `homie-runtime` | agent session claim/update task |
| 上游 | `intent-orchestrator` | 将用户意图转成 task |
| 下游 | `homie-storage` | 持久化 task state |
| 下游 | `session-context-store` | 关联 session lineage 和事件 |

## 4. 职责边界

负责：

- task create/list/update。
- claim、block、complete、return 状态机。
- task 与 session、workspace、artifact、memory reference 的关联。
- Beads 边界说明：Homie task 是运行时产品状态，Beads 是本仓库需求管理。

不负责：

- 运行 agent。
- 修改 Beads issue 状态，除非后续显式集成。
- 生成长期 memory。

## 5. 核心接口

```rust
pub trait TaskController {
    fn create(&self, request: CreateTaskRequest) -> Result<TaskRecord, TaskError>;
    fn claim(&self, task_id: TaskId, session_id: SessionId) -> Result<TaskRecord, TaskError>;
    fn update(&self, task_id: TaskId, update: TaskUpdate) -> Result<TaskRecord, TaskError>;
    fn return_task(&self, task_id: TaskId, reason: String) -> Result<TaskRecord, TaskError>;
}
```

## 6. 数据模型

```rust
pub enum TaskStatus {
    Open,
    Claimed,
    Blocked,
    Completed,
    Returned,
    Cancelled,
}

pub struct TaskRecord {
    pub id: TaskId,
    pub title: String,
    pub status: TaskStatus,
    pub claimed_by: Option<SessionId>,
    pub workspace: Option<WorkspaceScope>,
}
```

## 7. 运行模型与状态机

```text
open -> claimed -> completed
open -> claimed -> blocked -> claimed
claimed -> returned -> open
open/claimed -> cancelled
```

## 8. 安全与权限

- agent 只能更新其 permission profile 允许的 task。
- task note 不得保存 raw secret、raw prompt、完整 tool args/result。
- 跨 workspace claim 必须被权限策略允许。

## 9. 可观测性

- task.created。
- task.claimed。
- task.blocked。
- task.completed。
- task.returned。

## 10. 失败与恢复

| 场景 | 行为 |
|------|------|
| claimed session exited | task 保持 claimed 并标记 owner inactive，orchestrator 可回收 |
| task update 写失败 | 返回错误，不更新 projection |
| 权限不足 | fail closed |

## 11. 测试计划与验收引用

- FC-016: task claim/update/block/return。
- FC-018: full local quality gate。

## Diri Parity Mapping

This section is mandatory for Diri-to-Homie parity planning and fixes the review gap recorded in `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`.

| Field | Value |
|-------|-------|
| Owned feature atoms | Homie extension supporting M12/M13 orchestration handoff |
| Required Diri test mapping | task claim/block/handoff fixtures |
| Pre-implementation gaps | mark as extension and define Beads boundary |

Rules:

- A PRD/OpenSpec change touching this component must cite the owned feature atom ids above.
- Implementation is not complete until the required Diri test mapping has an equivalent Homie verification gate.
- If a new Diri source/test is discovered for this component, update `docs/research/diri-module-inventory.md` and this section before coding.

## 12. Diri 7ba3407 重基线修订

权威来源：

- PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`
- Requirements: FR-14, FR-16
- Beads: `homie-t3u`

本组件属于 Homie 扩展，当前状态是 `partial`。状态机模型存在，但尚未接入 storage repository、runtime session lifecycle、CLI/MCP 或桌面产品。

强制合同：

- task create/claim/block/complete/return/cancel 通过 service/repository 原子执行。
- claim 必须校验 session、workspace 和 permission profile。
- session exited/detached/unreachable 时标记 owner inactive；回收必须是显式 orchestrator/user action。
- task event 写入 context，并允许 UI、CLI 和 MCP 观察同一状态。
- Homie task 与 Beads 严格分离，不自动修改仓库需求 issue。

完成门禁：

- UI 或 CLI 创建 task -> agent session claim -> block/complete/return -> durable reload 的纵向 E2E。
- concurrent claim、inactive owner、permission denial 和 transaction rollback 测试。
- task note/context 不含 raw prompt、secret 或完整 tool payload。
