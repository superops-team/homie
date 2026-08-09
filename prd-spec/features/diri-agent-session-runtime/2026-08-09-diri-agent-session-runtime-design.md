# Diri Agent Session Runtime 设计

```yaml
change_id: diri-agent-session-runtime
status: ready_for_review
date: 2026-08-09
beads: homie-t3u.1
parent_change_id: diri-7ba3407-parity-rebaseline
master_task: T-102
depends_on:
  - diri-runtime-daemon-client-transport
baseline_repository: diri/
baseline_commit: 7ba3407
checkpoint_commit: 48f522b
```

## 1. 概述

### 1.1 背景

Wave 1A 已交付独立 `homie-runtime-daemon`、单 UDS 多路复用 transport、纯 async
`homie-client`、共享 daemon consumer 和 holder-safe daemon shutdown。T-102 不重做这些
能力，而是在既有 daemon/actor/holder 边界内补齐真实 agent session 生命周期。

checkpoint `48f522b` 的当前实现具备：

- `homie-runtime-holder` 持有真实 PTY、child process 和 output log；
- holder 支持 write、raw bytes、resize、stat、terminate 和 process-tree kill；
- daemon 重启时可以重新打开 storage、holder socket 和 output log；
- `homie-agents` 已有 19 个 manifest、readiness projection、screen detector、hook/notify
  parser 和 `StatusReducer`；
- storage 已有 runtime descriptor、agent profile、effective config、session 和
  session metadata 表；
- hibernate/wake、screen checkpoint、history scan 和 resume command 存在局部实现。

但这些能力尚未形成同一个可信的 agent session runtime：

- runtime spawn 仍固定启动 `/bin/sh -i`，忽略 manifest binary、argv、env scrub、
  injection、status authority 和 resume；
- holder 存活可以被加入 live registry，但 startup reconciliation 会把 storage/projection
  留在 `detached`；
- runtime 每次读取 status 时重新创建一个固定 `ScreenPrimary` reducer，没有使用目标
  agent manifest，也没有把 hook、notify、screen、process、user input 和 tick 接入同一
  reducer instance；
- hibernate 当前终止 holder 后再启动新 shell，不是同一 PTY/process tree 的
  stop/continue；
- process tree 已能 terminate，但缺少 stop/continue、内存采样和自动 resource policy；
- history resume 通过“启动 shell 后注入命令”实现，不是 manifest-driven direct spawn；
- 本地 checkpoint/resume/relaunch 基础尚不能支撑后续 migration，但本 change 也不能
  提前承诺 RT-010 的远端 transfer/handoff。

### 1.2 2026-08-09 当前测试事实

在 checkpoint `48f522b` 上执行：

```text
cargo test -p homie-runtime --test session_lifecycle -- --nocapture
```

结果为 `14 tests: 12 passed, 2 failed`。

| 测试 | 当前状态 | Actual | Expected | T-102 用途 |
|------|----------|--------|----------|------------|
| `runtime_reopen_can_adopt_holder_and_continue_session` | RED | `detached` | `running` | holder adoption |
| `runtime_spawn_shell_uses_live_pty` | RED | reopen 后 `detached` | `running` | PTY continuity |
| `runtime_holder_stat_tracks_resize_and_log_offsets` | GREEN | geometry/epoch/log offset 正确 | 保持通过 | 回归门禁 |

2026-08-08 的 Wave 1A evidence 记录过 3 个失败。该历史报告不改写；本 PRD 以
checkpoint `48f522b` 的 2026-08-09 实测作为 T-102 RED 基线，因此不得再把
`runtime_holder_stat_tracks_resize_and_log_offsets` 写成 blocker。

同一轮风险复核发现：`session_lifecycle` 失败后曾残留临时 data dir 对应的
`target/debug/homie-runtime-holder` PID `87051`。该进程已由用户手工终止，用户 packaged
holder 未触碰。这证明 RED 失败路径本身也必须执行 panic-safe holder/process-group
cleanup，不能只在测试成功尾部清理。

### 1.3 已确认根因

`RuntimeSupervisor::open_inner` 当前按以下顺序恢复：

```text
storage.mark_interrupted_sessions_detached()
  -> adopt_live_holders()
```

`adopt_live_holders()` 收到 `HolderRequest::Stat` 的 `status=running` 后，仅在它读取到的
persisted status 是 `created|starting|running` 时调用
`mark_session_running_if_exists()`。但前一步已经把这些状态统一改成 `detached`。

结果是：

```text
holder Stat = running
live registry = adopted
storage status = detached
session projection = detached
```

这精确解释当前两个 RED。修复不能通过放宽测试、把所有 storage row 标为 running，或
让 projection 忽略 holder 事实实现。

### 1.4 目标

1. 修复 startup reconciliation，使 live holder evidence、live registry、storage projection
   和 client snapshot 一致。
2. 保持真实 PTY/output/resize/stat 行为，确保 daemon/supervisor 重开后可继续 input/output。
3. 让 manifest 驱动 binary、argv、sanitized env、injection、status authority 和 resume。
4. 冻结每个 session 的 `EffectiveAgentConfig` 和 sanitized holder launch plan。
5. 将 hook、notify、screen、process、PTY activity、user input 和 tick 接入每 session
   的同一状态 reducer。
6. 补齐 process-tree stop/continue、内存采样和有界 resource governor。
7. 让 hibernate/wake 保持同一 holder、PTY 和 child tree；archive/kill 才终止 tree。
8. 实现同机 manifest resume/relaunch，并交付后续 migration 所需的本地 checkpoint
   与失败安全基础。
9. 保持 Wave 1A daemon/client 边界和 graceful shutdown 语义。
10. 以真实 daemon/holder/fake-agent process E2E 证明行为，不增加 production fallback。

### 1.5 非目标

- 不实现 remote node、SSH/tmux、checkpoint transfer、move/fork handoff 或 remote lease。
- 不把 RT-010 提升为 `implemented`；本 change 后远端 migration 仍由 T-401 负责。
- 不实现后续 terminal UI、GPUI 交互、scrollback/selection 或视觉准出。
- 不实现 provider forwarding、virtual-key issuance 或 raw provider credential custody。
- 不新增环境变量配置、production test mode、fake backend flag 或 embedded runtime。
- 不保留固定 shell spawn 作为未知 agent 的自动 fallback；shell 只能是显式 agent kind。
- 不修改 Wave 1A wire framing、event recovery、terminal stream 或 launcher ownership。
- 不修改 parity lock、master tasks 或产品代码，直到本规格评审通过并进入独立实施阶段。

## 2. 用户场景

### 场景 1：daemon 重启后继续真实 PTY 会话

**Given** holder 的 child、PTY socket 和 output log 均存活
**When** runtime daemon 被终止并从同一 absolute data dir 重启
**Then** runtime 以 holder `Stat` 为 live evidence 完成 adoption，session 保持可输入、
可输出且 projection 不再错误显示 `detached`

### 场景 2：缺少 holder 时不伪造 running

**Given** storage 中存在此前为 live 的 session row
**When** startup 无法验证 holder socket、child identity 或 holder status
**Then** session 投影为 `detached` 或 `exited`，live input fail closed，storage row 本身
不能证明 running

### 场景 3：manifest 驱动 agent 启动

**Given** 启用的 agent profile 指向一个可解析的 manifest binary
**When** caller 创建 agent session
**Then** runtime 解析绝对 executable，冻结 effective config，按 manifest argv/env/
injection 在 holder-owned PTY 内直接启动 agent，而不是先启动 shell 再拼命令

### 场景 4：多信号状态一致

**Given** session manifest 声明 hooks、screen 或 process authority
**When** runtime 收到 holder process、PTY output、screen observation、hook/notify、
user input 或 tick signal
**Then** 同一个 reducer instance 产生 canonical status/needs-input/turn-complete，
storage、event 和 snapshot 使用同一结果

### 场景 5：hibernate 后保持进程连续

**Given** 一个 idle、未 attach 的 agent process tree 正在运行
**When** resource governor 或用户执行 hibernate，之后再 wake
**Then** holder 对同一 tree 执行验证后的 stop/continue，PTY/output offset/session id
不变，wake 后可继续输入

### 场景 6：resume 恢复原 agent conversation

**Given** session 已退出或归档，且 frozen manifest 声明可验证的 resume 语义
**When** caller 执行 resume
**Then** runtime 在同一 Homie session id 下使用 manifest resume argv 启动新的 holder
incarnation，并保持 title、parent、profile、output epoch 和 checkpoint 连续

### 场景 7：graceful shutdown 不杀 holder

**Given** runtime 正在管理 live 或 hibernated holder session
**When** daemon 执行 prepare-shutdown 和 shutdown
**Then** runtime 停止新 mutation、flush reducer/checkpoint/event facts、返回 ACK 并退出，
holder/PTY 继续存活供 replacement daemon adoption

## 3. 功能需求

### FR-01：当前 RED/GREEN 基线

- T-102 的 RED 只能是当前两个 `detached != running` 失败。
- `runtime_holder_stat_tracks_resize_and_log_offsets` 必须作为保持 GREEN 的回归门禁。
- 不得删除、ignore、放宽或改名规避这三个现有测试。
- 新 RED 必须先证明 manifest spawn、stateful reducer、resource、resume 或 shutdown 的
  当前缺口，再写 production code。

### FR-02：Startup Reconciliation 与事实权威

- startup 必须先收集 persisted session facts 和 holder evidence，再决定 projection；
  禁止先批量把候选 session 改成 detached 后再 adoption。
- holder live evidence 必须至少包含成功 holder IPC、`ok=true`、live child status，以及
  与目标 session holder path 的一致关系。
- `Holder Stat=running` 是 live process/PTY 的权威证据。
- storage row 只能是恢复输入，不能单独证明 running。
- 对于此前为 `running|starting|created|detached` 且 holder 已验证 live 的 session，
  runtime 必须加入 live registry 并投影为 `running`。
- 对于此前为 `idle|needs_input` 且 holder 已验证 live 的 session，runtime 必须加入 live
  registry，并保留 reducer 产生的更具体行为状态。
- holder 明确退出时投影 `exited`；live evidence 缺失时投影 `detached`。
- startup 完成后，list、status report、snapshot 和 event projection 不得互相矛盾。

### FR-03：Holder Adoption 与真实 PTY Continuity

- holder 继续是 PTY、child tree 和 output log 的 sole owner。
- daemon、actor、app 或 client 退出不得关闭 holder PTY。
- adoption 不得启动第二个 child、第二个 holder writer 或截断既有 output log。
- holder launch 必须使用 structured launch plan，不经 shell 字符串拼接。
- holder status 必须保留 child PID、tree size、geometry、epoch offset 和 log offset；
  现有 stat/resize/log-offset 语义不得回归。
- reopen 后 send/input/resize/read/snapshot 必须访问被 adoption 的同一 holder。
- holder IPC、spawn readiness 和 cleanup 必须有固定 timeout；不得无界等待。

### FR-04：Manifest-Driven Agent Spawn 与 Effective Config

- spawn request 必须明确选择 agent profile 或显式 shell kind。
- committed `assets/agent-descriptors/*.json` 必须通过
  `homie-agents` 内显式 `include_str!` 静态表编译成 immutable catalog；packaged daemon 和
  standalone CLI 运行时不得依赖 current working directory、PATH 或外部 resource 查找
  manifest。production 不接受环境变量 manifest override。
- readiness 只解析 executable，不执行 agent 本体；解析结果必须是绝对、可执行文件。
- T-102 必须解析并冻结 `ResolvedEffectiveAgentConfig` contract：
  - profile/runtime/LLM/permission identifiers；
  - manifest id/version/status authority；
  - absolute executable；
  - final argv；
  - sanitized env key/value；
  - injection decision；
  - resume semantics；
  - initial geometry、cwd、parent session。
- T-103 `homie-t3u.2` 是 schema、repository、effective-config persistence 的唯一 owner。
  T-102 G3 完成上述类型/字段 contract handoff 后，T-103 `S103-GREEN-02` 持久化、hash、
  atomic session binding 和 readback；T-102 G5 必须等待该 GREEN handoff。
- running session 不受后续 profile/manifest edit 影响。
- env 必须从 allowlisted baseline 构造，并应用 manifest env scrub 和 explicit env；
  不得继承 provider raw key、Authorization、cookie 或上游 agent session identity。
- injection 必须由 manifest 声明驱动；UI、CLI 和 caller 不得自行拼 agent command。
- binary unavailable、profile disabled、manifest invalid、injection build 失败、T-103
  effective-config repository commit 失败或 holder 未 ready 时，spawn 必须通过 repository
  transaction/compensation 回滚 session/holder/config，不留半成品。
- `shell` 使用显式 `/bin/sh -i`；未知 manifest 不得自动退化为 shell。

### FR-05：Status Reducer 与 Runtime Hook Wiring

- 每个 live session 必须持有一个由 frozen manifest authority 创建的 reducer。
- runtime screen detection 必须使用 `ManifestEngine`，不得继续用 agent-agnostic
  hard-coded screen phrase 作为完整 agent status 实现。
- 以下 signal 必须进入同一 reducer：
  - holder process ready/exit；
  - PTY output activity；
  - manifest screen observation；
  - Claude hook；
  - Codex notify；
  - user input；
  - periodic tick。
- `hook.report` 必须传递可验证的 structured event，而不是绕过 reducer 直接写最终状态。
- subagent hook 只能更新 subagent bookkeeping，不能覆盖 parent status/title/needs-input。
- reducer outcome 必须先持久化 canonical projection，再发布对应 event。
- snapshot/status read 是 side-effect-free projection，不得每次重新创建 reducer。
- daemon restart 后 reducer 可从 persisted status、needs-input、screen checkpoint 和 holder
  evidence 重建；无法恢复的 debounce 内部状态可以清零，但不得伪造 running。

### FR-06：Process Tree 与 Resource Sampling

- holder 必须对真实 child tree 执行 enumerate、stop、continue、terminate 和 kill。
- 发送 signal 前后必须使用 PID start-time 防止 PID reuse。
- stop 必须验证 tree 进入 stopped 状态；continue 必须从 leaves 到 root 恢复。
- terminate 必须先 TERM/CONT，超过 grace 后 KILL/CONT，并清理脱离 root process group 的
  descendant。
- resource sample 至少包含 tree size 和 memory footprint；采样失败返回 safe unknown，
  不得把 session 标为 exited。
- resource operation 不得运行在 Tokio socket worker；由 actor/专用 bounded lane 调度。

### FR-07：Resource Governor、Hibernate、Wake 与 Archive

- governor 使用固定、有界 tick；不得为每 session 创建无界 worker。
- governor 只能自动 hibernate `idle`、未 attach、未 pinned 的 session。
- `running|needs_input|starting` session 不得因 memory threshold 自动 hibernate。
- hard memory threshold 只能 hibernate 合格 idle session，不能静默 terminate。
- hibernate 使用 holder process-tree stop，保留 holder、PTY、output log 和 session id。
- wake 使用 holder process-tree continue，并在验证 live 后恢复 reducer/status。
- hibernated 状态下 input fail closed；resize 可以记录并在 wake 时应用。
- archive 显式终止 holder/tree 并保留 resumable record；unarchive 本身不自动 spawn。
- kill/release、archive、hibernate 的语义必须分离，不能都退化为更新 storage status。

### FR-08：Manifest Resume 与本地 Migration 基础

- `session.resume` 必须对 exited/archived session 使用 frozen manifest resume semantics。
- ID-based resume 只有在持久化 agent session id 存在时可用；latest resume 仅用于 manifest
  明确声明 `latest` 的 agent。
- resume 必须直接 launch resume argv，不得先启动 shell 再发送文本。
- resume 在同一 Homie session id 下建立新 holder epoch，保留 output log、title、parent、
  profile 和 checkpoint。
- resume 失败时保留原 record 和 output，不得把 session 伪装为 running。
- 本 change 只交付同机 migration substrate：
  - flush screen/output checkpoint；
  - 冻结 source effective config；
  - 同 session id 的 stop/relaunch/resume；
  - target readiness 前不提交 projection；
  - 失败后保留可重试的 source record/checkpoint。
- `session.migrate` remote method、git/transcript transfer、target quarantine、move/fork lease
  和 source/target host 切换不在本 change 发布，RT-010 保持 `partial`。

### FR-09：Durable Shutdown 与 Recovery

- `prepare_shutdown` 必须拒绝新的 spawn/resume/archive/hibernate/governor mutation。
- 已接受的 mutation 按 Wave 1A drain 规则完成或到达固定 timeout。
- shutdown 前必须 flush reducer projection、needs-input、screen checkpoint、event store、
  output index 和 SQLite WAL。
- shutdown ACK 必须先送达 client，再关闭 listener/actor。
- graceful daemon shutdown 不 terminate live 或 hibernated holder。
- hard crash 后 replacement daemon 使用同一 startup reconciliation 恢复。
- holder cleanup 只作用于明确 session/fixture，不得使用全局 `pkill`。

### FR-10：安全、错误与无 Fallback

- provider raw key、Authorization、cookie、完整 tool args 和 raw prompt 不得进入 holder argv、
  launch metadata、logs、events 或 evidence。
- holder/agent executable、cwd 和 config path 必须是 absolute path 或固定 package path。
- production 不新增环境变量开关、fake manifest、test mode、embedded runtime 或 shell
  fallback。
- stable error 至少区分 unknown agent、binary unavailable、invalid effective config、
  holder unavailable、session not live、session hibernated、not resumable、backpressure 和
  timeout。
- client capability 只发布已有 production handler 的方法；不得为 remote migration 或
  后续 UI 发布 placeholder。

### FR-11：SDD/TDD、E2E 与 Evidence

- tasks 必须按 RED -> GREEN -> REFACTOR -> EVIDENCE 执行。
- 每个 task 必须可由单次 TraeCLI 完成，并声明文件 ownership、timeout 和 cleanup。
- fake agent 测试必须启动真实本地 executable、真实 holder、真实 PTY；不能使用
  production fake backend。
- 必须有真实 daemon SIGTERM/SIGKILL/restart、holder survival、reopen input/output、
  status/hook、hibernate/wake、resume 和 shutdown E2E。
- test fixture 必须记录自己创建的 daemon/holder/root child PID 和 socket；cleanup 只处理
  fixture-owned resource，并验证最终计数为零。
- 每个真实进程 suite 必须在运行前记录 holder PID+start-time 基线，在成功、assertion
  failure、panic 和 timeout 后重新采样；测试新增 holder 的集合差必须为空。该门禁只按
  进程名观测，不得按进程名 kill。
- evidence 状态只使用 `pass|blocked|not_run|partial|fail`。
- parity lock 只有在 implementation evidence 完成后由 master owner 更新；本规格阶段不改。

## 4. 方案设计

### 4.1 组件关系

```text
app / CLI / MCP
       |
  homie-client
       |
runtime daemon connection hub
       |
 RuntimeActor (single owner)
   |       |          |
reconcile status   resource governor
   |       |          |
holder IPC + output log + screen checkpoint
       |
real PTY + manifest agent process tree
```

边界保持：

- connection hub 不读写 holder/storage/live registry；
- RuntimeActor 拥有 session mutation、resolved effective-config contract、reducer 和 live
  registry；T-103 repository 独占 durable config/schema ownership；
- bounded lane 执行 output scan/process sampling；
- holder 拥有 PTY、child tree 和 output writer；
- `homie-agents` 提供 manifest、launch-plan building rules、screen detection 和 reducer；
- storage 保存 durable identity/projection，不能替代 holder liveness。

### 4.2 Startup Reconciliation Matrix

| Persisted state | Holder evidence | Reconciliation |
|-----------------|-----------------|----------------|
| created/starting/running/detached | live running | adopt；registry live；projection running |
| idle/needs_input | live running | adopt；registry live；保留 idle/needs_input |
| hibernated | live stopped | adopt；registry live；projection hibernated |
| archived | no live holder | 保持 archived |
| any live candidate | holder exited/status marker | projection exited；不加入 live registry |
| any live candidate | missing/unverifiable | projection detached；不加入 live registry |
| archived | unexpected live holder | fail startup/reconciliation evidence；不得静默标 running |

reconciliation 必须按 session 产生明确 outcome，再更新 storage 和 registry。禁止全局
`mark_interrupted_sessions_detached()` 作为 adoption 前置步骤。

### 4.3 Manifest Launch Pipeline

```text
SessionSpawnRequest
  -> load enabled profile
  -> resolve bundled manifest
  -> readiness resolve absolute binary
  -> build sanitized env/injection
  -> freeze EffectiveAgentConfig
  -> call T-103 repository to atomically freeze/bind session + effective config
  -> launch holder with structured plan
  -> wait holder Stat/live readiness
  -> register reducer/live session
  -> commit running + publish spawned
```

任一步失败执行逆序补偿：

```text
stop fixture/session holder if launched
  -> remove live registry entry
  -> invoke T-103 repository rollback/transaction semantics
  -> retain only safe error
```

holder launch plan 不包含 provider raw key。后续 T-402 可提供 scoped virtual key 和 local
proxy URL；T-102 只定义可选受控输入，不实现 provider credential issuance。

### 4.4 Canonical Status Flow

```text
holder stat / output / screen / hook / notify / input / tick / exit
                              |
                    per-session StatusReducer
                              |
              SessionStatus + NeedsInput + TurnComplete
                              |
                    storage commit -> event publish
```

holder live 证明 liveness；manifest reducer 证明 live session 的行为状态。两者不可互相
替代。

### 4.5 Resource Lifecycle

```text
running/idle
  -> sample tree + footprint
  -> eligible idle session
  -> holder STOP tree
  -> verify stopped
  -> hibernated
  -> holder CONT leaves-first
  -> verify live
  -> running/idle
```

archive/kill 使用 terminate tree；hibernate 不销毁 holder，不新建 shell。

### 4.6 Resume 与 Migration 边界

本 change 的 resume/relaunch 保持：

- 同一 Homie session id；
- 同一 output log 的新 epoch；
- frozen manifest/profile/permission identity；
- screen checkpoint 和 output offset；
- manifest 声明的 resume argv。

本 change 的 migration 只是上述同机基础和失败安全 checkpoint，不创建或公开 remote
handoff handler。远端传输、host 切换和 lease 仍由 T-401 规格决定。

### 4.7 Timeout 与 Cleanup

| 操作 | 上限 |
|------|------|
| holder IPC request | 350 ms |
| holder/agent readiness | 3 s |
| stop/continue verification | 2 s |
| terminate graceful phase | 500 ms |
| terminate total cleanup | 3 s |
| status/output/process sample | 10 s |
| one real daemon E2E phase | 15 s |
| one complete process E2E case | 60 s |

测试 cleanup 必须：

1. 测试前记录全部 holder PID+start-time 基线，但不操作 baseline holder；
2. 记录 fixture data dir、daemon PID、holder socket/PID 和 child sample；
3. 用 panic-safe guard 保证 assertion failure、panic 和 timeout 也进入 cleanup；
4. 优先走 session kill/holder terminate/daemon shutdown；
5. bounded wait；
6. 只对仍存活且 start-time 匹配的 fixture PID 使用 SIGKILL；
7. reap child；
8. 断言 fixture daemon/holder/socket/pid file 为零；
9. 重新采样并断言相对基线的新增 holder 集合为空；
10. 不处理用户真实 data dir 或 baseline 中的任何进程。

## 5. 边界情况

| 场景 | 处理 |
|------|------|
| storage running 但无 holder | detached，input fail closed |
| holder running 但 storage 曾被标 detached | adoption 后 running，修复 projection |
| holder running 且 persisted idle/needs_input | 保留行为状态，同时标记 live registry |
| holder exited marker 与 stale socket 同时存在 | exited；清理 stale control files |
| profile/manifest 在 session 运行中变化 | running session 继续使用 frozen config |
| agent binary 在 spawn 前消失 | spawn rollback，stable unavailable |
| hook 乱序或来自 subagent | reducer 去重/隔离，不覆盖 parent |
| resource sample 失败 | memory unknown；不 kill、不伪造 exited |
| hibernated session 收到 input | stable session_hibernated；不丢到 output log |
| resume agent 没有 id 且不是 latest | not_resumable |
| resume holder 已存活 | adopt 现有 holder，不启动第二个 incarnation |
| prepare-shutdown 与 governor tick 竞争 | 停止新 tick，已接受 mutation bounded drain |
| remote migrate 请求 | capability absent/method_not_found；不返回虚假成功 |

## 6. 影响范围

### 6.1 实施阶段预计产品文件

| Owner | 文件范围 | 责任 |
|-------|----------|------|
| runtime reconciliation | `crates/homie-runtime/src/lib.rs`, new focused reconciliation module | startup facts、adoption、projection |
| holder protocol/process | `crates/homie-runtime/src/holder.rs`, `src/bin/homie-runtime-holder.rs`, `src/process_tree.rs` | structured spawn、stat/signal/sample/cleanup |
| agent adapter | `crates/homie-agents/src/lib.rs`, `src/status.rs`, `src/detect/**` | launch plan、manifest authority、reducer |
| runtime actor/status | `crates/homie-runtime/src/runtime_actor.rs`, dispatcher/status modules | spawn/status/hook lifecycle |
| resource governor | new focused runtime governor module and daemon wiring | bounded sample/hibernate/wake |
| protocol/client | `crates/homie-proto`, `crates/homie-client` focused DTO/method additions | typed spawn/resume/lifecycle |
| durable handoff | T-103 `S103-GREEN-02` output, read-only dependency from T-102 | v4 effective-config freeze/readback and atomic session binding |
| tests | runtime/agents/client/CLI process suites | RED/GREEN/E2E/cleanup |

不计划修改 Wave 1A frame、connection、event stream 或 terminal stream wire 实现，除非
RED 证明 T-102 必需；出现这种情况必须先修订规格。

### 6.2 长期组件规格

| Component spec | 影响 |
|----------------|------|
| `specs/runtime-supervisor/README.md` | 修正当前 adoption 事实；定义 reconciliation、holder/resource/resume/shutdown |
| `specs/agent-adapter-contract/README.md` | 定义 manifest launch plan、effective config 和 reducer runtime wiring |

其他长期 spec 本轮不修改。Storage schema/repository/effective-config persistence 已明确由
T-103 `homie-t3u.2` 独占；T-102 不编辑 `homie-storage`。若实施发现必须改变 credential
custody、transport wire、UI 或 remote contract，先阻塞并创建对应 owner 的规格修订。

## 7. 测试与 Evidence 计划

### 7.1 RED

- 复现两个当前 adoption/PTY RED，保留完整 failure output。
- 添加 startup reconciliation table tests。
- 添加 manifest fake executable direct spawn RED。
- 添加 stateful reducer + hook/notify runtime RED。
- 添加 stop/continue/memory/hibernate continuity RED。
- 添加 direct manifest resume RED。
- 添加 shutdown/restart adoption RED。

### 7.2 GREEN

- 现有 `session_lifecycle` 14/14；
- `runtime_holder_stat_tracks_resize_and_log_offsets` 持续通过；
- `homie-agents` manifest/reducer/hook suites；
- runtime actor/dispatcher/resource focused suites；
- client/proto typed lifecycle contracts；
- real daemon + real holder + real PTY + fake executable process E2E。

### 7.3 REFACTOR

- 删除固定 shell production spawn 和 history shell-command injection；
- 删除 agent-agnostic runtime screen classifier；
- 删除 startup bulk-detach-before-adopt 路径；
- 删除 terminate-and-respawn hibernate；
- 扫描并禁止 production fallback/test mode/env override。

### 7.4 EVIDENCE

实施阶段在 `docs/verification/diri-agent-session-runtime/` 记录：

- RED baseline；
- functional cases/results；
- real daemon/holder E2E；
- process cleanup；
- security review；
- two-round code review；
- test report；
- release-readiness report。

本 S102 规格任务受文件 ownership 限制，不在本轮创建 evidence 目录或修改 parity lock。

## 8. 验收标准

1. Bead `homie-t3u.1`、change id、master T-102、checkpoint 和 PRD 路径一致。
2. OpenSpec proposal/design/4 capability specs/plan/tasks/alignment 完整且 strict valid。
3. 16 维 spec review 无阻断项。
4. 当前两个 RED 在实施后不修改断言地转 GREEN。
5. `runtime_holder_stat_tracks_resize_and_log_offsets` 始终保持 GREEN。
6. `session_lifecycle` 14/14，并至少连续 5 次 serial 运行无 flake。
7. fake agent 经真实 daemon/holder/PTY 按 manifest binary/argv/env 启动。
8. holder survival、reopen input/output、hook/status、stop/continue、hibernate/wake、
   resume 和 graceful/hard restart E2E 通过。
9. test-owned daemon/holder/child/socket cleanup 为零，不影响用户进程。
10. 每个真实进程 suite 的测试前/后 holder PID+start-time 集合差为空，且 RED
    failure/panic/timeout 路径也通过 panic-safe cleanup。
11. 无 production shell fallback、embedded runtime、test mode、env override 或 remote
    placeholder。
12. T-102 scoped evidence 可以通过，但 RT-010 remote migration、后续 UI/remote 仍明确
    `partial`/deferred。
13. parity lock 和 Bead 只在 implementation evidence 与 delivered state 一致后由 owner
    更新。

## 9. Beads 追踪

| 字段 | 值 |
|------|----|
| Bead | `homie-t3u.1` |
| Parent | `homie-t3u` |
| Dependency | `homie-nep` closed |
| Change ID | `diri-agent-session-runtime` |
| Master task | `T-102` |
| Baseline | Diri `7ba3407` |
| Checkpoint | `48f522b` |

Bead 当前 acceptance text 仍提到 3 个旧 failure。本轮受文件 ownership 限制不更新 Bead；
实施入口必须以本 PRD 的 2 RED + 1 GREEN 基线为准，并在允许更新 issue state 时同步。
