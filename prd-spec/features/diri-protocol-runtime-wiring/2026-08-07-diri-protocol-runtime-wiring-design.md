# Diri 协议运行时接线设计文档

```yaml
change_id: diri-protocol-runtime-wiring
beads: homie-qci
parent_bead: homie-h7n.1
source_lock: docs/research/diri-parity-lock.md
target_rows:
  - API-002
  - API-003
```

## 1. 概述

### 1.1 问题/背景

Homie 已经具备 `homie-client` 的 in-process runtime facade，并且 app 的部分终端数据源已切换到 `HomieClient`。但 Diri 的 client/protocol 能力不仅是进程内函数调用，还包含外部控制面可通过协议 request/response/event frame 连接 runtime、恢复事件游标、订阅 runtime events。

当前 parity lock 中 `API-002` 仍标记为 `partial`，原因是缺少外部 subscription transport；`API-003` 仍标记为 `partial`，原因是 CLI 仍有部分 session 路径绕过 runtime，只写 storage，容易产生“存在 session 记录但没有真实 PTY”的假状态。

### 1.2 目标

- 为 `homie-client` 增加基于 `homie-proto::ControlMessage` 的 NDJSON transport dispatcher。
- `events.subscribe` 必须输出真实 runtime event frame，并返回可继续 resume 的 cursor。
- `events.wait` 必须按 timeout 等待真实 runtime event，而不是只做一次静态读取。
- `homie-cli session create/list/snapshot` 必须通过 `HomieClient` 访问 runtime state。
- 增加 `homie control-stdio`，为外部进程提供最小可验证的 protocol transport 入口。
- 通过测试与证据将 `API-002` 从 `partial` 推进到 `implemented`；`API-003` 仅更新证据，保持 `partial`，直到 worktree/ports/MCP bridge 完整完成。

## 2. 用户场景

### 场景 1: 外部 client 通过 NDJSON 控制 runtime

**Given** Homie 已经有一个 runtime session 并产生了事件  
**When** 外部进程向 `homie control-stdio` 写入 `ControlMessage::Request(events.subscribe)`  
**Then** Homie 输出匹配的 `ControlMessage::Event` frame，并返回带 cursor 的 success response。

### 场景 2: CLI 创建真实 session

**Given** 用户通过 CLI 创建 session  
**When** 执行 `homie session create --workspace <path>`  
**Then** CLI 必须启动真实 holder-owned PTY，并且后续 `session snapshot` 能读取 runtime snapshot，而不是只返回 storage-only 记录。

### 场景 3: runtime event wait

**Given** 外部 client 传入 `afterSeq` 和 event filter  
**When** 执行 `events.wait` 且暂时没有新事件  
**Then** Homie 在 timeout 内轮询 runtime event ring；命中后返回事件与 cursor，超时则返回空事件和原 cursor。

## 3. 功能需求

### FR-1: ControlMessage transport dispatcher

`homie-client` 必须提供可复用 dispatcher，输入 NDJSON `ControlMessage`，输出 NDJSON `ControlMessage`。request 必须映射到现有 runtime 方法，错误必须用 safe `ErrorEnvelope` 返回，不能 panic 或静默吞掉。

### FR-2: 外部 events.subscribe

`events.subscribe` 必须：

- 解析 `EventsSubscribeRequest`；
- 读取 persisted runtime event ring；
- 根据 `afterSeq` 和 `eventFilter` 输出一个或多个 `ControlMessage::Event`；
- 返回 success response，包含 `cursor.nextSeq` 和事件数量。

### FR-3: events.wait timeout 语义

`events.wait` 必须按 `timeoutMs` 等待新事件；命中后返回当前 cursor 后的真实事件，超时返回空集合和 `timedOut=true`。

### FR-4: CLI runtime-backed session path

`homie session create/list/snapshot` 必须通过 `HomieClient`，不允许再由 CLI 直接写 storage 伪造 session lifecycle。

### FR-5: 外部入口

CLI 必须提供 `homie control-stdio --data-dir <dir>`，从 stdin 读 NDJSON control messages，向 stdout 写 NDJSON control messages。

## 4. 实现方案

### 4.1 homie-client

- 增加 `handle_control_message`，把单个 `ControlMessage::Request` 转换为 response/event frames。
- 增加 `serve_control_stream`，用 `BufRead`/`Write` 实现 NDJSON transport。
- 将 `events.wait` 的 timeout 逻辑下沉到 `HomieClient::handle_request`，让 CLI 与 transport 共用语义。

### 4.2 homie-cli

- 新增 `control-stdio` subcommand。
- `session create/list/snapshot` 改为通过 `HomieClient`。
- 保留 `doctor/runtime status` 的 storage health 逻辑，因为它们不是 session lifecycle 操作。

### 4.3 证据与锁表

- 新增 client transport integration tests。
- 新增 CLI control-stdio/session tests。
- 更新 `docs/research/diri-parity-lock.md`：`API-002` 标记为 `implemented`，`API-003` 仍为 `partial` 并补充 transport/session evidence。

## 5. 边界情况

| 场景 | 处理方式 |
|------|---------|
| stdin 空行 | 忽略，不输出 frame |
| JSON 非法 | 返回 protocol error，transport 函数返回错误，CLI 非零退出 |
| 收到 response/event 入站 frame | 返回 unsupported incoming message 的 failure response，不执行 runtime side effect |
| runtime session 不存在 | 返回 `runtime_error` failure response |
| events.wait 超时 | 返回 success response，`events=[]`、`timedOut=true` |

## 6. 涉及文件

- `crates/homie-client/src/lib.rs`
- `crates/homie-client/tests/runtime_client.rs`
- `crates/homie-cli/src/main.rs`
- `crates/homie-cli/tests/*`
- `docs/research/diri-parity-lock.md`
- `docs/verification/diri-protocol-runtime-wiring/*`
- `openspec/changes/diri-protocol-runtime-wiring/*`

## 7. 测试计划

- `cargo test -p homie-client --tests -- --nocapture`
- `cargo test -p homie-cli --test control_stdio_cli -- --nocapture`
- `cargo test -p homie-cli --test session_snapshot_cli -- --nocapture`
- `cargo test -p homie-proto --tests -- --nocapture`
- `cargo clippy -p homie-proto -p homie-client -p homie-cli --all-targets -- -D warnings`
- `make parity-lock`

## 8. 验收标准

- `API-002` 有真实 external transport 证据，不再仅依赖 in-process facade。
- CLI `session create` 产生 runtime-backed live session。
- `events.subscribe` 输出 `ControlMessage::Event` frame 和 cursor response。
- `events.wait` 覆盖命中与 timeout 行为。
- 所有证据写入 `docs/verification/diri-protocol-runtime-wiring/`。

