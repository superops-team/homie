# Engine Session 运行时拆分设计文档

## 1. 概述

### 1.1 问题/动机

`homie/crates/homie-engine/src/session.rs` 当前约 2,888 行，单个 `Session` 类型及其
`impl` 同时承担四类职责，且 `control/handlers.rs` 中仍有约 800 行 spawn/resume/migrate
业务逻辑未下沉到领域层（`engine-control-wire-runtime-split` 的 documented residual）：

1. **会话生命周期**：`spawn`/`spawn_remote`/`adopt_remote`/`adopt`/`attach`/`spawn_direct`/
   `spawn_held`/`spawn_held_deferred`，以及 `SessionSpec`/`HolderConfig`/`RemoteSessionSpec`/
   `RemoteAdoptSpec`/`RemoteLaunchCleanup`/`DeferredLaunch`/`LaunchHandoff`/`DeferredState`；
2. **屏幕/网格渲染**：`SessionView`/`PromptInputState`/`Shared`/`RemoteGridState`/`GridSignature`/
   `GridWake`/`grid_update_if_changed`/`screen_lines`/`read_scrollback*`/`scroll`；
3. **PTY I/O**：`Transport`/`write_raw`/`send_text`/`paste_text`/`submit_input`/`write_input`/
   `resize`/`read_output`/`screen_size`/`child_pid`；
4. **状态 reducer**：`feed_signal`/`claude_hook`/`observe_prompt_input`/`capture_prompt_title`/`status`。

这四类职责变化原因不同（会话生命周期 vs 屏幕渲染 vs 终端 I/O vs 状态机），混在一个
`Session` 里导致：任何 PTY 环境、屏幕检测、生命周期或状态信号的改动都要加载 2,888 行
上下文，回归半径被放大；同时 spawn/resume/migrate 的 handler 逻辑仍停留在 transport
层（`control/handlers.rs`），未归位到 session/registry 领域，违反「handler 只做 decode/
副作用调度，领域逻辑在 domain 模块」的既定分层。

这是 `engine-registry-session-split`（已关闭，registry 持久化已分离）明确 defer 的
「session 内部深化」，也是 `engine-control-wire-runtime-split`（已关闭）明确记录的
「spawn/resume/migrate sinking 残余」。

### 1.2 目标

1. 把 `session.rs` 拆成内聚的子模块（生命周期、屏幕/网格、PTY I/O、状态 reducer），
   单文件 < 800 行。
2. 把 `control/handlers.rs` 中 spawn/resume/migrate 的业务逻辑下沉到 `session`（生命周期）
   与 `registry`（协调）域，handler 退化为薄适配层（decode → 调用 domain → encode）。
3. 保持 wire 协议、`Session` 对外公开 API、磁盘持久化语义、PTY 环境合同完全不变。
4. 拆出的子模块能脱离 daemon socket 环境独立单测。
5. 遵守 `specs/engine-session-runtime.md`：runtime authority 与 PTY 环境合同不变。

### 1.3 非目标

- 不改变 `ControlMessage` wire shape、method 名或 JSON 语义。
- 不改变 `Session` 的公开方法签名与行为（纯职责搬迁 + 下沉，无新行为）。
- 不引入新的持久化后端（SQLite/rocksdb，属 `persistence-incremental-state` 阶段 2）。
- 不重写会话状态机逻辑（status reducer 只搬迁，不改语义）。
- 不接入真实 provider typed driver（属后续 `typed-agent-driver-capabilities` child）。
- 不触及 `homie-proto/src/control.rs`（协议定义不是本 PRD 治理对象）。

### 1.4 基线快照

- branch: `main`
- baseline commit: 记录于 Beads 启动时 HEAD
- 目标文件：`homie/crates/homie-engine/src/session.rs`（2,888 行）、
  `homie/crates/homie-engine/src/control/handlers.rs`（1,607 行，其中 spawn/resume/migrate
  相关约 800 行）
- 相关测试：`homie/crates/homie-engine/tests/`（session/registry/control/holder/pty 相关）
- 相关 spec：`specs/engine-session-runtime.md`

### 1.5 与存量 PRD 的关系

| 存量文档 | 关系 |
|----------|------|
| `architecture-audit-governance-2026-08` | 其 F7 的 session 内部深化落点（原 homie-ubu.2 defer 部分） |
| `engine-registry-session-split` | 本 PRD 是其「session 内部深化」后续 child |
| `engine-control-wire-runtime-split` | 本 PRD 消化其「spawn/resume/migrate sinking」残余 |
| `specs/engine-session-runtime.md` | 拆分后 runtime authority / PTY 环境合同保持不变 |

## 2. 现状分析

### 2.1 session.rs 职责映射

| 职责 | 主要符号 | 目标子模块 |
|------|----------|-----------|
| 会话生命周期 | `SessionSpec`/`HolderConfig`/`RemoteSessionSpec`/`RemoteAdoptSpec`/`RemoteLaunchCleanup`/`DeferredLaunch`/`LaunchHandoff`/`DeferredState` + `spawn*`/`adopt*`/`attach` | `session/lifecycle.rs` |
| 屏幕/网格渲染 | `SessionView`/`PromptInputState`/`Shared`/`RemoteGridState`/`GridSignature`/`GridWake` + `grid_update_if_changed`/`screen_lines`/`read_scrollback*`/`scroll` | `session/screen.rs` |
| PTY I/O | `Transport` + `write_raw`/`send_text`/`paste_text`/`submit_input`/`write_input`/`resize`/`read_output`/`screen_size`/`child_pid` | `session/pty.rs` |
| 状态 reducer | `feed_signal`/`claude_hook`/`observe_prompt_input`/`capture_prompt_title`/`status` | `session/status.rs` |

### 2.2 control/handlers.rs spawn/resume/migrate 现状

`session_spawn`(72–288)、`session_spawn_remote`(289–479)、`session_migrate`(522–680)、
`session_resume`(1064–1140)、`remote_resume_spec`(1141–1245)、`session_resume_from_history`
(1246–1278)、`resume_spec`(1279–1352)、`session_reopen_last`(1353–1364)、`session_hibernate`
(1455–1468)、`session_wake`(1469–1483) 中仍含参数解析、会话构造、恢复决策等业务逻辑，
应下沉到 `session/lifecycle` 与 `registry` 的协调入口，handler 只保留 decode/encode 与
副作用触发。

## 3. 方案设计

### 3.1 拆分拓扑

```text
session/
├── mod.rs        # Session 公开类型 + SessionView 再导出 + 内部 mod 装配（< 800 行）
├── lifecycle.rs  # 生命周期：spawn/adopt/attach/resume/migrate + 相关 spec 结构
├── screen.rs     # 屏幕/网格：SessionView/Grid 相关 + scrollback
├── pty.rs        # PTY I/O：Transport + 读写/resize
└── status.rs     # 状态 reducer：feed_signal/hook/prompt
```

拆分子模块通过 `pub(crate)`/`pub(super)` 共享 `Session` 内部状态，`Session` 保持单一
公开类型；`SessionView` 继续作为对外快照类型，其构造下沉到 `screen.rs`。

### 3.2 下沉拓扑

`control/handlers.rs` 的 spawn/resume/migrate handler 退化为：

```text
decode params → session::lifecycle::spawn(...) / registry::resume(...) / registry::migrate(...)
             → encode result
```

领域入口以 `Registry` 与 `Session` 已有公开方法为主，避免 handler 直接操作私有字段；
确需跨模块协调的逻辑落在 `registry`（live session 协调）而非 handler。

### 3.3 行为保持约束

- wire shape、method 名、JSON 参数/返回完全不变；
- `Session` 公开方法签名不变（纯搬迁，不改签名、不改语义）；
- PTY 环境合同（`shell_pty_environment`）与 spawn/adopt 语义不变；
- 磁盘持久化与恢复语义不变（不触碰 `registry` 已分离的持久化模块）。

## 4. 验收标准

1. `session.rs` 拆分为 `session/{lifecycle,screen,pty,status}.rs`，各单文件 < 800 行。
2. spawn/resume/migrate 业务逻辑从 `control/handlers.rs` 下沉，handler 仅做 decode/encode。
3. `cargo test -p homie-engine` 全绿，无新增失败；`cargo fmt --check`、`cargo clippy -D warnings` 通过。
4. 行为不变：新增/既有 session/registry/control/holder/pty 测试均通过，无断言弱化。
5. 拆分前后 wire 协议兼容（`protocol_contract` / golden fixture 不变）。
6. 触及 `specs/engine-session-runtime.md` 边界的，同步更新 spec 并记录。

## 5. 测试计划

- **RED→GREEN→REFACTOR**：先为每个拆分 seam 建立「行为保持」回归锚（既有测试全绿作为
  refactor 基线，逐步搬迁，每步保持绿）；对明确下沉的 handler，先写 decode→domain→encode
  的单元测试证明薄适配行为，再搬迁。
- **Tier 2（normal refactor）**：纯职责搬迁，无新行为、无并发/凭据/数据丢失边界改动；
  涉及 PTY 进程生命周期（spawn/adopt/terminate）的搬迁按 Tier 3 加一条失败模型（进程残留、
  部分启动、恢复竞争）并补 stress/rehearsal 证据。
- 证据目录：`docs/verification/engine-session-runtime-split/`。

## 6. Beads 追踪

- change_id: `engine-session-runtime-split`
- 类型: refactor（Tier 2，PTY 生命周期部分 Tier 3）
- 优先级: P0
- 上游: `engine-registry-session-split`（session 内部深化 defer）、
  `engine-control-wire-runtime-split`（spawn/resume/migrate sinking residual）
