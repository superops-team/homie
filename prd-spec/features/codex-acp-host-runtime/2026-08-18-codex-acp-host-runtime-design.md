# Codex ACP Host Harness（backend）首个纵向切片 设计文档

## 1. 概述

### 1.1 问题/背景

上游 `codex-acp-harness-runtime`（homie-sc6）已锁定协议选型：**ACP + pinned `codex-acp`**，
并定义了 backend harness 的模块边界、ACP 数据模型、event 投影契约、composer/transcript/approval
交互契约，以及 Apple/design 规范。但这些都是**设计与规范**，尚未落地代码。

`typed-agent-driver-capabilities`（homie-kcq）已实现 `AgentDriverControl` trait、
`DriverCapabilities`、`UnsupportedDriver`、`FakeDriver`，但**未接入任何真实 provider**。

本 PRD 是 ACP 的第一个**真实代码纵向切片**：在 `homie-engine` 中实现 ACP host harness 的
核心——JSON-RPC 2.0 协议 DTO、newline-delimited JSON framing、host 循环（spawn 子进程、
`initialize` 握手、`session/new`/`session/prompt`/`session/stop`/`session/cancel`、
接收 `session/update` 通知）、approval 四态记忆、以及 `AcpDriver`（实现
`AgentDriverControl`）。

本阶段**不依赖 `codex-acp` crate 作为库**，而是把 harness 实现为**通用 ACP host（stdio
子进程）**：`codex-acp` 作为可配置的 ACP server 二进制路径被 spawn，harness 通过 JSON-RPC
over stdio 与它通信。这使 harness 能统一支撑任意 ACP server，且不引入需要网络的库依赖。

### 1.2 目标

1. 实现 ACP JSON-RPC 2.0 协议 DTO（serde），覆盖 initialize / session/new / session/prompt /
   session/stop / session/cancel / session/update 及常见 update kinds。
2. 实现 newline-delimited JSON framing（ACP 的 stdio 帧格式）。
3. 实现 `AcpHost`：spawn ACP server 子进程、`initialize` 握手、request/response 按 id 关联、
   `session/update` 通知派发。
4. 实现 `AcpDriver`（实现 `AgentDriverControl`）：capabilities 由 `initialize` 协商结果填充，
   cancel_turn→`session/stop`，steer_message→`session/prompt`，respond_permission→permission 响应，
   model_options→可用模型。
5. 实现 approval 四态记忆（allow/deny once + always allow/deny for session）。
6. 用 **fake ACP server** 证明 host 循环端到端可用（不依赖真实 codex-acp）。

### 1.3 非目标

- 不引入 `codex-acp` crate 作为库依赖（避免网络拉取与版本耦合）；只把它作为可配置二进制路径。
- 不实现 `fs/read_text_file` / `fs/update_text_file` 文件代理（属后续 child Bead）。
- 不接入 GPUI chat canvas / composer / transcript（属 `chat-surface-gpui` child Bead）。
- 不在本阶段把 `AcpDriver` 接入 `Session` spawn 路径（session-driver-handle 集成属后续）。
- 不支持 `session/load` 恢复、rollback/fork（协议 DTO 可预留，但不实现语义）。
- 不实现认证/浏览器登录代理（`authenticate` 预留 DTO，不实现语义）。

## 2. 用户场景

### 场景 1: harness 与 ACP server 握手

**Given** 配置了 ACP server 二进制路径（如 `codex-acp`）。  
**When** `AcpHost::spawn(path)` 被调用。  
**Then** harness spawn 子进程、发送 `initialize`、收到能力协商结果，`AcpDriver` 据此暴露
capabilities。

### 场景 2: 发送 prompt

**Given** host 已 initialize，session 已 `session/new`。  
**When** 调用 `session/prompt`。  
**Then** harness 发送 JSON-RPC request 并收到响应；agent 产生的 `session/update` 通知被派发。

### 场景 3: 取消 turn / steer

**Given** turn 正在运行。  
**When** `AcpDriver::cancel_turn()` 或 `steer_message(text)` 被调用。  
**Then** 映射为 `session/stop`（保持 session）与 `session/prompt`（steer）。

### 场景 4: 审批四态

**Given** 收到 permission 请求。  
**When** 用户选择 allow/deny once 或 always allow/deny for session。  
**Then** `ApprovalMemory` 按 kind 记忆 always 决策，后续同类请求自动应用。

## 3. 功能需求

### FR-1: ACP JSON-RPC 2.0 协议 DTO

- `JsonRpcRequest { jsonrpc, id, method, params }` / `JsonRpcResponse { id, result|error }` /
  `JsonRpcNotification { jsonrpc, method, params }`。
- 方法常量：`initialize`、`session/new`、`session/prompt`、`session/stop`、`session/cancel`。
- `session/update` 通知的 `sessionUpdate` kind：agent_message_changed / agent_thought_changed /
  plan / tool_call / available_commands_update / current_mode_update / session_status_update。
- serde 反序列化未知 kind 不 panic（记录/丢弃，不崩溃 host）。

### FR-2: framing

- newline-delimited JSON：每帧一行 JSON，`\n` 分隔。
- 编码/解码往返一致；空行跳过；非法 JSON 返回明确错误而非 panic。

### FR-3: AcpHost

- `Transport` trait 抽象 stdio 读写（`read_line` / `write_line`），便于测试注入。
- `StdioTransport` 包装子进程 stdin/stdout。
- request/response 按唯一 id 关联（`AtomicU64` 递增 + pending map）。
- 后台 reader 线程持续读帧：response 路由到对应请求，notification 派发到 channel。
- `spawn` 后自动 `initialize` 握手。

### FR-4: AcpDriver（AgentDriverControl）

- capabilities 由 initialize 结果填充。
- `cancel_turn()` → `session/stop`。
- `steer_message(text)` → `session/prompt`。
- `respond_permission(request_id, option_id)` → permission 响应。
- `model_options()` → 可用模型列表。
- 不支持的操作返回 `DriverError::unsupported`。

### FR-5: approval 四态

- `PermissionDecision::AllowOnce / DenyOnce / AlwaysAllow / AlwaysDeny`。
- `ApprovalMemory` 按 permission kind 记忆 always 决策，once 决策不记忆。

### FR-6: 安全边界

- 不记录 secret / Authorization / 完整敏感 prompt payload。
- host 错误短消息化，详细诊断只进内部日志。

## 4. 实现方案

### 4.1 模块边界

```text
homie/crates/homie-engine/src/acp/
├── mod.rs        # 模块根，re-export
├── protocol.rs   # JSON-RPC 2.0 DTO + 方法/kind 常量
├── frame.rs      # newline-delimited JSON framing
├── host.rs       # AcpHost + Transport + StdioTransport + reader 线程
├── approval.rs   # PermissionDecision + ApprovalMemory
└── driver.rs     # AcpDriver: AgentDriverControl
```

### 4.2 数据模型

```rust
// protocol.rs
struct JsonRpcRequest { jsonrpc: String, id: i64, method: String, params: Value }
struct JsonRpcResponse { jsonrpc: String, id: i64, result: Option<Value>, error: Option<RpcError> }
struct JsonRpcNotification { jsonrpc: String, method: String, params: Value }

// host.rs
trait Transport: Send {
    fn read_line(&mut self) -> io::Result<Option<String>>;
    fn write_line(&mut self, line: &str) -> io::Result<()>;
}
struct AcpHost { /* child, reader thread, pending map, notification channel */ }

// approval.rs
enum PermissionDecision { AllowOnce, DenyOnce, AlwaysAllow, AlwaysDeny }
struct ApprovalMemory { always: HashMap<String, PermissionDecision> }

// driver.rs
struct AcpDriver { host: AcpHost, capabilities: DriverCapabilities }
```

### 4.3 I/O 模型

- 同步 std 模型，与 engine 现有同步设计一致（不引入 tokio 到 engine）。
- `AcpHost` 内部起一个 reader 线程（`std::thread`），持续 `read_line` 解析帧并路由。
- 请求写入 `StdioTransport` 的 stdin；响应/通知由 reader 线程读 stdout。

### 4.4 Fake ACP server（测试用）

- 集成测试 spawn 当前测试二进制（`current_exe`）作为 fake ACP server：一个 `--acp-fake-server`
  模式，在 stdin 上读 JSON-RPC，响应 `initialize`/`session/*`，并可选推送 `session/update`
  通知，证明 host 循环端到端可用。

### 4.5 首阶段关闭口径

- protocol DTO + framing + host + driver + approval 均可编译、可单测。
- fake ACP server 集成测试证明 spawn→initialize→session/new→prompt→notification 全链路。
- 不接入 Session spawn、不接 GPUI、不引 codex-acp crate。

## 5. 边界情况

| 场景 | 处理 |
|------|------|
| 子进程 spawn 失败 | 返回明确 `io::Error`，不 panic |
| initialize 失败 | 返回 `DriverError`，capabilities 为空 |
| 响应 id 不匹配 | 记录 diagnostic，不 panic |
| 未知 update kind | 反序列化为通用 value，不崩溃 |
| 非法 JSON 帧 | framing 返回错误，host 记录 diagnostic |
| approval 未决 once | 不记忆，仅本次 |
| always 已记忆 | 后续同类请求自动应用 |

## 6. 涉及文件

- `homie/crates/homie-engine/src/acp/*`（新增）
- `homie/crates/homie-engine/src/lib.rs`（注册 `pub mod acp;`）
- `specs/engine-session-runtime.md`（记录 ACP 与 PTY authority 边界，不改变 runtime 合同）

## 7. 验证计划

### 7.1 单元测试

- protocol DTO serde 往返、未知 kind 容忍。
- framing 编解码往返、空行/非法 JSON。
- approval 四态与 always 记忆。
- AcpDriver capabilities 映射、cancel/steer/respond_permission/model_options 方法映射。

### 7.2 集成测试

- fake ACP server：spawn→initialize→session/new→session/prompt→session/stop→notification 全链路。
- host request/response id 关联正确。

### 7.3 门禁

- `cargo check --workspace`
- `cargo fmt --all --check`
- `cargo test -p homie-engine acp`

## 8. 验收标准

1. `homie-engine/src/acp/` 五模块可编译，单测全绿。
2. fake ACP server 集成测试端到端通过。
3. `AcpDriver` 正确实现 `AgentDriverControl`，能力由 initialize 协商填充。
4. approval 四态语义正确。
5. 现有 engine 行为不回退（无 ACP 的 session 不受影响）。
6. OpenSpec alignment 对齐本 PRD，Beads `homie-skh` 关闭。

## 9. Beads 追踪

- Beads: `homie-skh`
- change_id: `codex-acp-host-runtime`
- 类型: feature
- 优先级: P0
- 上游: `homie-sc6`（codex-acp-harness-runtime 设计）、`homie-kcq`（typed-agent-driver-capabilities）
