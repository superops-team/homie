# Intent Orchestrator 组件规格

## 1. 组件定位

`homie-orchestrator` 负责把用户意图、command palette action、New Agent 请求、MCP spawn 和 task routing 转换为可执行的 agent/session/workflow 操作。首版只实现最小可解释路由，不引入复杂自动规划。

## 2. 来源需求映射

- PRD: `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`
- OpenSpec: `openspec/changes/reference-parity-v1/`
- 功能验证: FC-014, FC-016, FC-018

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | `homie-app` | New Agent、palette、quick open action |
| 上游 | MCP tools | `spawn_agent`、`send_prompt`、`wait_for_agent` |
| 下游 | `homie-runtime` | spawn/session commands |
| 下游 | `homie-task` | create/claim/update task |
| 下游 | `session-context-store` | 读取 context summary 和 lineage |

## 4. 职责边界

负责：

- intent classification。
- default agent/profile selection。
- workspace scope selection。
- route decision audit。
- delegation lineage policy。

不负责：

- agent 内部推理。
- UI rendering。
- credential 解密。
- 长期 memory ranking。

## 5. 核心接口

```rust
pub trait IntentOrchestrator {
    fn route(&self, input: IntentRequest) -> Result<IntentDecision, IntentError>;
}

pub enum IntentDecision {
    SpawnSession(SessionSpawnRequest),
    SendPrompt { session_id: SessionId, text: String, submit: bool },
    CreateTask(CreateTaskRequest),
    OpenUiSurface(UiSurface),
}
```

## 6. 数据模型

```rust
pub struct IntentRequest {
    pub source: IntentSource,
    pub text: Option<String>,
    pub workspace: Option<WorkspaceScope>,
    pub parent_session: Option<SessionId>,
    pub requested_agent: Option<AgentProfileId>,
}
```

## 7. 运行模型与状态机

```text
receive intent
  -> classify source and requested action
  -> load allowed profiles/permissions
  -> produce deterministic decision
  -> execute through runtime/task/client boundary
  -> write route audit
```

## 8. 安全与权限

- MCP source 必须绑定 session identity 和 lineage。
- 跨 session 写入必须经 permission profile。
- intent request 不得携带 raw credential。
- route audit 只保存 safe fields。

## 9. 可观测性

- intent.received。
- intent.routed。
- intent.rejected。
- intent.executed。

## 10. 失败与恢复

| 场景 | 行为 |
|------|------|
| 无可用 default agent | 返回 stable error |
| 权限不足 | reject with safe error |
| ambiguous route | 要求用户选择，不自行猜测高风险操作 |

## 11. 测试计划与验收引用

- FC-014: MCP automation route。
- FC-016: intent routes new-agent/palette/MCP spawn。
- FC-018: full local quality gate。

## Diri Parity Mapping

This section is mandatory for Diri-to-Homie parity planning and fixes the review gap recorded in `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`.

| Field | Value |
|-------|-------|
| Owned feature atoms | M12-F002, M13-F001, M13-F002 |
| Required Diri test mapping | MCP lineage and tool routing fixtures |
| Pre-implementation gaps | MCP lineage/automation intent mapping |

Rules:

- A PRD/OpenSpec change touching this component must cite the owned feature atom ids above.
- Implementation is not complete until the required Diri test mapping has an equivalent Homie verification gate.
- If a new Diri source/test is discovered for this component, update `docs/research/diri-module-inventory.md` and this section before coding.

## 12. Diri 7ba3407 重基线修订

权威来源：

- PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`
- Requirements: FR-04, FR-11, FR-14, FR-16
- Beads: `homie-t3u`

本组件当前状态是 `partial`。deterministic route model 单测存在，但 route 没有通过统一 client/service 执行并形成 durable audit。

强制合同：

- UI、CLI 和 MCP 输入先标准化为 typed intent，再产生确定性 decision。
- decision 执行只能调用 runtime client、task、context 和 memory service，不直接操作 storage/live registry。
- agent/profile/workspace/host default 解析必须可解释并记录 safe decision facts。
- MCP source identity 来自可信 session binding，lineage/permission 在执行前校验。
- 高风险或多义 route 返回 explicit choice；不自动猜测 remote、delete、release 或 credential 操作。
- route audit 记录 decision、owner、source、safe ids 和 error code，不保存 raw prompt/secret。

完成门禁：

- New Agent、palette、MCP spawn 和 task route 至少各有一个真实执行 E2E。
- unavailable agent、ambiguous target、permission denial、runtime unavailable 和 duplicate request 测试。
- route 后 UI/CLI/MCP 观察同一 session/task/context 事实。
