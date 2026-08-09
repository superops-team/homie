# Runtime Supervisor 组件规格

## 1. 组件定位

`homie-runtime` 是 Homie 的后台运行时，负责 agent session、PTY/process、output log、headless screen、状态检测、session registry、resource governor、事件发布和恢复。UI、CLI、MCP 和远端节点都必须通过协议访问 runtime。

## 2. 来源需求映射

- PRD: `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`
- Gap-closure PRD: `prd-spec/features/diri-engine-migration/2026-08-06-diri-engine-migration-gap-closure-design.md`
- OpenSpec: `openspec/changes/reference-parity-v1/`
- Gap-closure OpenSpec: `openspec/changes/diri-engine-migration/`
- 功能验证: FC-006, FC-007, FC-010, FC-015, FC-018
- Gap-closure 功能验证: FC-DIRI-001, FC-DIRI-002, FC-DIRI-003

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | `homie-client`, `homie-cli`, MCP bridge | 通过 control/data channel 调 runtime |
| 下游 | `homie-agents` | 读取 runtime descriptor 和 status rules |
| 下游 | `homie-storage` | 持久化 session、output index、events |
| 下游 | `homie-llm` | 获取 virtual key/proxy config |
| 下游 | OS PTY/process | 启动和管理 agent child process |

## 4. 职责边界

负责：

- session spawn/list/attach/input/resize/kill/archive/hibernate/wake/history。
- PTY owner 或 holder-equivalent，降低 app/runtime crash 对 live session 的影响。
- offset-addressed output log 与 headless terminal screen。
- 状态检测、resource governor、events ring、state snapshot。
- remote host/node 执行边界的 runtime 入口。

不负责：

- GPUI 渲染。
- provider raw key 存储。
- 长期 memory/task 业务策略。
- update bundle 安装。

Gap-closure 边界：

- 本轮 `diri-engine-migration` 先交付本地 live PTY session registry、真实 shell spawn、PTY input、output log 和失败不落半成品 session。
- 当前已具备最小 holder-equivalent：`homie-runtime-holder` 子进程拥有 PTY 和 output log，`RuntimeSupervisor` 重开后可通过 holder socket 重新 adopt live session，并能从 holder status 文件区分正常 `exited` 与异常 `detached`。holder terminate 已验证可清理脱离 root shell process group 的子进程。完整 Diri 级 holder manager、资源治理、崩溃矩阵和 protocol/client resume 仍需后续任务补齐。
- Holder `Stat` 必须返回 child pid、状态、进程树规模、当前 geometry、epoch offset 和 log offset；client/protocol attach 可以用这些元数据判断可重放范围和当前终端尺寸。
- Runtime attach snapshot 必须一次性组合 session metadata、holder stat、status report 和 offset replay，避免 client/protocol 未来自行拼装出不一致状态。
- CLI snapshot 入口必须调用 runtime attach snapshot，而不能绕过 runtime 直接读 storage。
- Screen checkpoint 必须持久化 output offset、content seq 和 headless screen lines，作为后续 session migration 的基础恢复点。
- `archive`/`hibernate` 必须先停止 holder/process tree 再更新状态；`wake` 必须重启 holder-owned PTY 并验证可交互后再标记 running。
- `send_text` 对非 live session 必须 fail closed，不能退回到追加 output log 来模拟输入成功。

## 5. 核心接口

```rust
#[async_trait]
pub trait RuntimeSupervisor {
    async fn spawn(&self, request: SessionSpawnRequest) -> Result<SessionRecord, RuntimeError>;
    async fn list(&self) -> Result<StateSnapshot, RuntimeError>;
    async fn attach(&self, session_id: SessionId, role: ClientRole) -> Result<Attachment, RuntimeError>;
    async fn send_text(&self, session_id: SessionId, text: String, submit: bool) -> Result<(), RuntimeError>;
    async fn resize(&self, session_id: SessionId, cols: u16, rows: u16) -> Result<(), RuntimeError>;
    async fn archive(&self, session_id: SessionId) -> Result<(), RuntimeError>;
    async fn hibernate(&self, session_id: SessionId) -> Result<(), RuntimeError>;
    async fn wake(&self, session_id: SessionId) -> Result<(), RuntimeError>;
}
```

## 6. 数据模型

```rust
pub struct SessionSpawnRequest {
    pub agent_profile_id: AgentProfileId,
    pub cwd: PathBuf,
    pub title: Option<String>,
    pub initial_prompt: Option<String>,
    pub parent: Option<SessionId>,
    pub initial_cols: Option<u16>,
    pub initial_rows: Option<u16>,
    pub new_worktree: Option<WorktreeRequest>,
    pub host: Option<HostId>,
    pub same_repo_as: Option<SessionId>,
}

pub struct OutputLogIndex {
    pub session_id: SessionId,
    pub path: PathBuf,
    pub next_offset: u64,
}
```

## 7. 运行模型与状态机

```text
created -> starting -> running -> needs_input -> running
running -> idle -> hibernated -> waking -> running
running -> archived
running -> exited
exited -> resumed
```

Attachment lifecycle:

```text
attach -> full_snapshot -> diff frames -> ping/pong -> detach -> reattach
```

Gap-closure live session lifecycle:

```text
validate cwd/binary/permission
  -> spawn PTY child
  -> create/update session as starting
  -> pump PTY output to offset log and headless screen
  -> mark running/idle/needs_input/exited from reducer signals
```

## 8. 安全与权限

- process env 只注入 virtual key/proxy URL，不注入 provider raw key。
- output log 写入前不做语义清洗，但所有外发 event/report 必须脱敏。
- remote path 操作必须在 remote host/node 执行，不能由 UI 拼本机路径。
- kill/remove/archive/handoff 必须检查 permission profile。

## 9. 可观测性

事件：

- runtime.ready / runtime.unhealthy。
- session.spawned / session.updated / session.status / session.output / session.artifact / session.archived / session.removed。
- metrics.write_failed。

指标：

- session count、resident attachments、output bytes、status latency、resource memory、event lag。

## 10. 失败与恢复

| 场景 | 行为 |
|------|------|
| app 退出 | session 保持运行 |
| runtime crash | holder-equivalent 保持 PTY/output，重启后 registry 恢复；当前实现已覆盖 supervisor drop/reopen adopt 和 detached child kill，完整 holder-manager/resource-governor 崩溃矩阵仍未完成 |
| output log 写失败 | session 标记 degraded，继续保留 process 状态 |
| event ring gap | client 重新请求 state snapshot |
| remote host 不可达 | session spawn fail closed，已有 session 标记 unreachable |

当前 gap-closure 已验证 holder 恢复路径：supervisor drop/reopen 后可 adopt live holder，正常 holder exit 恢复为 `exited`，缺失 holder/status 证据恢复为 `detached`，terminate 可清理脱组子进程。完整 Diri 级 holder-manager/resource-governor crash matrix 仍由 parity lock 后续项跟踪。

## 11. 测试计划与验收引用

- FC-006: protocol/event contract。
- FC-007: runtime lifecycle and recovery。
- FC-010: worktree safety。
- FC-015: remote/node handoff。
- FC-018: full local quality gate。
- FC-DIRI-001: live PTY shell spawn/input/output。
- FC-DIRI-002: PTY spawn failure does not persist half-created session。
- FC-DIRI-003: non-live session input fails closed。
- FC-DIRI-010: holder-owned PTY survival/adoption, explicit terminate cleanup, exited/detached restore semantics。
- FC-DIRI-011: holder output log -> headless screen -> screen observation -> status reducer projection。
- FC-DIRI-012: holder process-tree termination for detached child processes。
- FC-DIRI-013: holder stat metadata, resize, and log offset reporting。
- FC-DIRI-015: runtime attach snapshot after supervisor reopen。
- FC-DIRI-016: CLI session snapshot via runtime snapshot。
- FC-DIRI-017: screen checkpoint persistence after supervisor reopen。
- FC-DIRI-018: hibernate/wake holder lifecycle。

## Diri Parity Mapping

This section is mandatory for Diri-to-Homie parity planning and fixes the review gap recorded in `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`.

| Field | Value |
|-------|-------|
| Owned feature atoms | M03-F002, M07-F001, M10-F001 runtime methods, M11-F002, M14-F001, M14-F002, M15-F001, M15-F002 |
| Required Diri test mapping | SessionIntegrationTests, OutputLogTests, HeadlessScreenTests, EventSubscriptionTests, HolderTests, ProcessTreeTests, ResourceGovernorSettingsTests, ScreenCheckpointTests |
| Pre-implementation gaps | runtime subdomain sections and method/event table |

Rules:

- A PRD/OpenSpec change touching this component must cite the owned feature atom ids above.
- Implementation is not complete until the required Diri test mapping has an equivalent Homie verification gate.
- If a new Diri source/test is discovered for this component, update `docs/research/diri-module-inventory.md` and this section before coding.

## 12. Diri 7ba3407 重基线修订

权威来源：

- PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`
- 能力矩阵: `docs/research/diri-7ba3407-capability-matrix.md`
- OpenSpec: `openspec/changes/diri-7ba3407-parity-rebaseline/`
- Beads: `homie-t3u`

当前结论为 `partial`。已有 local supervisor、holder protocol、output log、headless screen 和部分生命周期测试不能代表完整 runtime parity。当前 `session_lifecycle` 中 live PTY 和 holder adoption 测试返回 `detached`，因此 RT-001、RT-006、RT-007 必须保持 `partial`。

### 12.1 强制进程边界

- `homie-runtime` 必须提供独立 daemon binary，并作为 live session、PTY、holder、event bus 和 resource state 的唯一 owner。
- app、CLI、MCP、remote 和测试外的 client 不得在本进程创建 `RuntimeSupervisor`。
- runtime 对外只暴露 versioned control/data protocol；storage repository、holder socket 和 live registry 不得泄漏给调用方。
- daemon 必须实现 `prepare_shutdown` 和 `shutdown`，按固定顺序停止接收新请求、flush event/usage/context/output index、处理 attachment、保留或终止 session。

### 12.2 完整会话合同

必须覆盖：

- agent-aware spawn，而不是固定 `/bin/sh`；
- attach/read/send/resize/wait/kill/release；
- archive/unarchive、hibernate/wake；
- checkpoint、history/resume、session migrate；
- holder adopt、process-tree stop/continue/terminate；
- memory sampling、resource governor 和 crash recovery；
- output offset、screen checkpoint、holder stat 的一致 snapshot。

禁止：

- 向 output log 追加文本来模拟 PTY input 成功；
- 在 holder 状态不确定时把 session 标为 running；
- 用 storage row 存在替代 live registry/holder 证据；
- 为旧 in-process client 保留生产 fallback。

### 12.3 完成门禁

本组件只有在以下条件全部满足后才可声明 `implemented`：

1. 独立 daemon/client transport E2E 通过。
2. app/runtime crash 后 holder-owned PTY 可重连并继续交互。
3. 19 个 manifest 中可用 agent 至少按 manifest binary/argv/env/injection 启动，shell 只作为显式 agent kind。
4. process tree、resource、hibernate/wake、archive/unarchive、migrate 和 shutdown failure matrix 通过。
5. `cargo test -p homie-runtime --test session_lifecycle` 当前失败项修复且重复运行无 flake。
6. `cargo test --workspace` 不再被 runtime 测试阻断。
7. 证据记录在对应 wave 的 `docs/verification/<change-id>/`，状态词符合 observability 合同。

## 13. Wave 1A Daemon/Actor 修订

权威来源：

- PRD: `prd-spec/features/diri-runtime-daemon-client-transport/2026-08-08-diri-runtime-daemon-client-transport-design.md`
- OpenSpec: `openspec/changes/diri-runtime-daemon-client-transport/`
- Beads: `homie-nep`

### 13.1 Process Ownership

- 每个 absolute data dir 只能有一个 `homie-runtime-daemon`。
- daemon 是 `RuntimeSupervisor`、SQLite connection、live registry、event ring 和 terminal source 的唯一 production owner。
- daemon 必须先获得 non-blocking singleton lock，再清理 stale socket。
- app、CLI、MCP 和 `homie-client` 不得构造 production `RuntimeSupervisor`。

### 13.2 RuntimeActor

- `RuntimeSupervisor` 由单一 `RuntimeActor` 持有，不要求 rusqlite connection 跨线程共享。
- `RuntimeActor` 运行在名为 `homie-runtime-actor` 的专用 OS thread，独占 SQLite、PTY、live registry 和 runtime mutation。
- UDS connection tasks 通过容量 256 的 bounded command channel 和 oneshot reply 调 actor。
- actor 不执行 async socket write；connection hub 不直接读写 SQLite、holder socket 或 live registry。
- actor 不执行 git、history scan 或 bounded output scan；这些操作进入 13.5 的 `LongRunningLane`。
- actor queue 满返回 `backpressure`，不得阻塞 Tokio worker。
- event wait、heartbeat 和 stream scheduling 在 actor 外执行。
- internal `RuntimeBackend` 只作为 server library test seam；production daemon 不提供 fake backend/test mode 参数。

### 13.3 Daemon Lifecycle

启动顺序：

```text
paths/permissions
  -> singleton lock
  -> stale socket cleanup
  -> storage migration and holder adoption
  -> RuntimeActor
  -> UDS listener
  -> runtime.ready
```

关闭顺序：

```text
prepare_shutdown
  -> reject new mutation
  -> finish current control responses
  -> flush/checkpoint durable facts
  -> shutdown ACK
  -> close listener/streams
  -> stop actor
```

- SIGTERM/SIGINT 使用相同 drain。
- graceful daemon shutdown 不终止 holder-owned session。
- hard crash 后的新 daemon 从 storage、holder status 和 output log 恢复。
- holder adoption 当前 `detached` 回归仍由 T-102 关闭，Wave 1A 不得伪造 `running`。

### 13.4 Terminal Source

- holder output log 是 Wave 1A terminal replay/live source。
- daemon terminal stream hub 对每个 attached session 只创建一个共享 `TerminalSource`，多个 client stream 订阅该 source。
- source 活跃时最高 20 Hz tail，idle 时退避到 250 ms，每次最多读取 64 KiB。
- actor 只返回验证后的 daemon-internal source descriptor；stream hub 在 actor 外读取 output log 和维护 `HeadlessScreen`。
- full grid 从现有 `HeadlessScreen` 生成；缺失 style 使用协议默认值。
- T-202 可增加 styled grid diff，但不得改变 Wave 1A frame、offset、sequence 或 reset 合同。
- terminal stream 的 input/resize 必须回到 actor 的 session command，不能追加 output log 模拟成功。

### 13.5 Long-Running Operations

- daemon 使用一个名为 `homie-runtime-long-running` 的专用 OS worker 和容量 32 的 job queue。
- lane 只接收 actor prepare 阶段生成的 owned path/DTO snapshot，不得持有 `Storage`、`RuntimeSupervisor`、live registry 引用。
- handler 顺序是 `actor prepare -> lane execute -> actor commit`；lane 失败、超时或取消时不得 commit 部分 storage state。
- 固定 deadlines：output/artifact/status/snapshot 10s；git list/diff/locate 15s；history 30s；worktree create/remove 60s。
- queued job 超时、connection 断开或 waiter 已取消时不得启动。
- 已启动 read-only job 在 deadline/cancel 时终止；已启动 worktree mutation 忽略 caller cancel，继续到成功或 60s hard deadline。
- git child 使用独立 process group，hard deadline 时终止整个 group 并回收；mutation 不自动重放或自动 prune。
- 单 worker 串行所有 git jobs，Wave 1A 不增加 repo-key coordinator 或多个 blocking pools。
- graceful shutdown 拒绝新 jobs，丢弃 queued/read-only jobs，并等待已启动 mutation 到 hard deadline。
