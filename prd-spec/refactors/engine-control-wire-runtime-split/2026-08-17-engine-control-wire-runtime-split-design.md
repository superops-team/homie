# Engine Control Wire/Runtime 分离设计文档

## 1. 概述

### 1.1 问题/动机

`homie/crates/homie-engine/src/control.rs` 当前约 3,802 行，`ControlServer` 单个 impl
块（第 56–2239 行，约 2,180 行）同时承担三类职责：

1. **wire 编解码与 method 路由**：`serve`/`handle_line`/`dispatch`/`events_subscribe`/
   `events_wait`，以及文件底部约 40 个 free function（`write_message`/`decode`/`encode`/
   `with_session`/`history_entry_to_wire`/`worktree_to_wire` 等）负责 JSON 编解码与 `ControlMessage`
   读写；
2. **业务 handler 下沉**：约 35 个 `session_*`/`host_*`/`worktree_*`/`daemon_*`/`browser_*`/
   `governor_*` 方法，各自 decode 参数、访问 `registry`、执行副作用、encode 返回；
3. **runtime 生命周期协调**：`bind`/`spawn_remote_restore`/`restore_remote_bindings`/
   `retire_legacy_remote_sessions`/`daemon_shutdown`/`daemon_shutdown_if_idle`/`Drop` 等，管理
   UnixListener、订阅句柄、连接守卫、空闲关停与远程恢复。

这三类职责变化原因不同：wire 形状变（协议改动）时不应触碰 handler 逻辑；session/host
语义变时不应触碰 transport 细节。当前混在单文件里，任何协议或生命周期变更都需要加载
整个 3,800 行上下文，回归半径被放大。

这是 2026-08 架构审计 finding **F2（Critical）**：`engine/control.rs` dispatcher + runtime
coordinator 双职责。

### 1.2 目标

1. 把 `control.rs` 拆成 wire 层（编解码 + 路由）与 runtime 层（生命周期协调），handler 逻辑下沉到
   registry/session/holder 各自职责。
2. 保持 wire 协议（`homie_proto::ControlMessage` shape、method 名、JSON 参数/返回）完全不变。
3. 纯逻辑（编解码、参数解析、返回组装）先抽，可独立单测，不要求 daemon socket 环境。
4. 降低后续协议/session/host 变更的 review 面，单文件行数 < 800。
5. 遵守 `specs/engine-session-runtime.md`：runtime authority 与 PTY 环境合同不变。

### 1.3 非目标

- 不改变 `ControlMessage` 的 wire shape、method 名或 JSON 语义。
- 不改变 `ControlServer::serve`/`bind` 对外行为或 socket 协议。
- 不重写 session/host 业务语义；只做职责搬迁，不引入新行为。
- 不合并或删除 handler，不新增协议能力。
- 不触及 `homie-proto/src/control.rs`（该文件是协议定义，不是本 PRD 的治理对象）。

### 1.4 基线快照

- branch: `main`
- baseline commit: `e4c7454`
- 目标文件：`homie/crates/homie-engine/src/control.rs`（3,802 行）
- 相关测试：`homie/crates/homie-engine/tests/`（session/registry/control 相关）
- 相关 spec：`specs/engine-session-runtime.md`

### 1.5 与存量 PRD 的关系

| 存量文档 | 关系 |
|----------|------|
| `architecture-audit-governance-2026-08` | 本 PRD 是其 child（homie-ubu.1），F2 |
| `architecture-audit-hardening` | Phase 3 已规划 ControlServer method-family 抽取原则，本 PRD 落地其 P0 切片 |
| `specs/engine-session-runtime.md` | runtime authority 合同，拆分后必须保持不变 |

## 2. 现状分析

`ControlServer` 当前职责拆解（行号基于基线快照）：

| 层 | 成员 | 变化原因 |
|----|------|----------|
| wire transport | `serve`/`handle_line`/`dispatch`/`events_subscribe`/`events_wait` | 协议/流式事件 |
| wire codec | `write_message`/`decode`/`encode`/`resolve_on_path`/`history_entry_to_wire`/`worktree_to_wire` | 序列化 |
| handler | `session_spawn`/`session_spawn_remote`/`session_list`/`session_send_text`/`session_resize`/`session_kill`/`session_remove`/`session_rename`/`session_archive`/`session_hibernate`/`session_wake`/`session_resume`/`session_reopen_last`/`host_*`/`worktree_*`/`daemon_*`/`browser_call`/`governor_configure`/`hook_report`/`project_add` 等约 35 个 | 业务语义 |
| runtime coordinator | `bind`/`spawn_remote_restore`/`restore_remote_bindings`/`retire_legacy_remote_sessions`/`daemon_shutdown`/`daemon_shutdown_if_idle`/`Drop` | 生命周期 |

关键观察：handler 方法普遍是「decode 参数 → 查/改 registry → encode 返回」三段式，其中
参数解析与返回组装是纯函数、registry 访问是业务逻辑。当前二者耦合在每个 handler 内。

## 3. 方案设计

### 3.1 拆分原则

- **wire 纯逻辑先抽**：编解码、参数解析、返回组装抽成无 socket / 无 registry 依赖的纯函数。
- **handler 下沉**：把 handler 的业务逻辑下沉为 `Registry`/`Session`/`RemoteManager` 的职责方法，
  `ControlServer` 只保留 method 路由表（method string → 下沉调用）。
- **runtime 生命周期单独成模块**：`bind` 循环、订阅句柄、连接守卫、空闲关停、远程恢复放
  `runtime.rs`。
- 每步行为不变，`cargo test` 全绿。

### 3.2 目标模块拓扑

```text
homie/crates/homie-engine/src/
├── control.rs                 # ControlServer：构造、路由表、serve/handle_line/dispatch（< 800 行）
├── control/
│   ├── wire.rs                # write_message/decode/encode/read_message + wire helper（纯函数）
│   ├── codec.rs               # history_entry_to_wire/worktree_to_wire 等 proto↔domain 投影
│   └── runtime.rs             # bind 循环、订阅句柄、连接守卫、空闲关停、远程恢复
├── registry.rs                # 承接 session_*/host_*/worktree_* 的业务 handler 逻辑
├── session.rs                 # 承接 session resume/hibernate/wake 生命周期细节
└── remote/manager.rs          # 承接 remote spawn/adopt 细节（已有，补 handler 下沉）
```

### 3.3 下沉映射（handler → 目标模块）

| handler 方法 | 下沉目标 |
|--------------|----------|
| `session_spawn`/`session_spawn_remote`/`session_resume`/`session_resume_from_history`/`session_reopen_last` | `registry`（spawn/resume 编排，复用现有 `Registry::spawn`/`respawn`/`reopen_last_closed`） |
| `session_list`/`session_rename`/`session_mark_seen`/`session_archive`/`session_unarchive`/`session_remove`/`session_kill` | `registry`（已有同名/近似方法） |
| `session_send_text`/`session_resize`/`session_read_screen`/`session_read_scrollback`/`session_read_scrollback_cells`/`session_read_diff` | `registry`（通过 `Session` 视图） |
| `session_hibernate`/`session_wake` | `registry`（已有 `hibernate`/`wake_session`） |
| `host_*`/`worktree_*`/`project_add`/`browser_call`/`governor_configure`/`hook_report` | 保持路由，抽纯逻辑到 `wire.rs`/`codec.rs`，副作用下沉到对应子模块 |
| `daemon_shutdown`/`daemon_shutdown_if_idle`/`daemon_prepare_shutdown` | `runtime.rs`（生命周期） |

### 3.4 实施顺序（每次一个可验证切片）

1. **S1**：抽 `wire.rs`（编解码 + 参数解析 + 返回组装纯函数），`control.rs` 引用它们。测试覆盖
   encode/decode round-trip。
2. **S2**：抽 `codec.rs`（proto↔domain 投影纯函数）。测试覆盖投影不变量。
3. **S3**：抽 `runtime.rs`（bind 循环、订阅句柄、连接守卫、空闲关停、远程恢复）。
4. **S4**：把 `session_*` 等 handler 逻辑下沉到 `registry.rs`/`session.rs`/`remote/manager.rs`，
   `ControlServer` 只保留路由表。

每步完成后 `cargo test -p homie-engine` 全绿，再做下一步。

## 4. 测试与验收

### 4.1 测试计划

- 纯函数单测（`wire.rs`/`codec.rs`）：encode/decode round-trip、参数缺省/非法值、投影不变量。
- 集成测试（现有 `homie-engine/tests/`）：daemon spawn/resume/send_text/read_screen 行为不变。
- 协议兼容回归：固定 method 名与返回 shape 的 golden 断言（若已存在复用，否则新增最小 golden）。

### 4.2 验收标准

1. `control.rs` < 800 行；`control/wire.rs`/`control/codec.rs`/`control/runtime.rs` 各自内聚单一职责。
2. `cargo test -p homie-engine` 全绿，无新增失败。
3. wire shape 不变：`ControlMessage` 的 method 名、参数、返回 JSON 与拆分前完全一致。
4. 拆出的 `wire.rs`/`codec.rs` 无 socket / GPUI / registry 依赖，可脱离 daemon 单测。
5. `specs/engine-session-runtime.md` 的 runtime authority 与 PTY 环境合同保持不变。

## 5. Beads 追踪

- change_id: `engine-control-wire-runtime-split`
- parent Beads: `homie-ubu`；child Beads: `homie-ubu.1`
- 类型: refactor
- 优先级: P0
- 验收证据目录: `docs/verification/engine-control-wire-runtime-split/`
