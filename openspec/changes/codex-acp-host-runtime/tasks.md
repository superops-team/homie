# OpenSpec Tasks — codex-acp-host-runtime

本变更为 Rust 代码实现，tasks 覆盖协议/framing/host/approval/driver 五模块与端到端验证。

## T1: ACP JSON-RPC 2.0 协议 DTO（protocol.rs）

- 交付：`JsonRpcRequest` / `JsonRpcResponse` / `JsonRpcNotification` / `RpcError` DTO，
  方法常量、`session/update` kind 常量、`classify_inbound` 帧分类。
- 验收：serde 往返测试 + 未知 kind 容忍 + 非法 JSON 不 panic。
- 关联验证 Case：FC-1。

## T2: newline-delimited framing（frame.rs）

- 交付：`encode` / `decode` / `read_line`，`\n` 分隔、空行/空白行跳过、CRLF 容忍。
- 验收：编解码往返、空行/空白行跳过、CRLF、EOF。
- 关联验证 Case：FC-2。

## T3: ACP host 循环（host.rs）

- 交付：`AcpHost`（spawn 子进程 + `initialize` 握手 + pending id 关联 + 后台 reader 线程 +
  通知派发）、`AcpClient` trait、`AcpError`、`Drop` 正确回收（先 kill child 再 join reader）。
- 验收：`from_stream` 单测可注入 transport；`Drop` 不挂起。
- 关联验证 Case：FC-3。

## T4: approval 四态记忆（approval.rs）

- 交付：`PermissionDecision`（AllowOnce/DenyOnce/AlwaysAllow/AlwaysDeny）+ `ApprovalMemory`
  按 kind 记忆 always、once 不记忆。
- 验收：once 不记忆、always 按 kind 召回、后写覆盖、option_id 稳定。
- 关联验证 Case：FC-4。

## T5: AcpDriver 实现 AgentDriverControl（driver.rs）

- 交付：`AcpDriver`，capabilities 由 initialize 结果填充；cancel→`session/stop`、
  steer→`session/prompt`、respond_permission→`session/respond_permission`、model_options→unsupported。
- 验收：方法映射单测（RecordingClient 断言 method/params）。
- 关联验证 Case：FC-5。

## T6: fake ACP server 端到端集成测试（tests/acp_host.rs）

- 交付：`harness = false` 集成测试，spawn 自身二进制 `--acp-fake-server` 扮演 agent 侧，
  走真实 `AcpHost::spawn` 子进程路径。
- 验收：spawn→initialize→session/new→session/prompt→session/update 通知→session/stop 全链路。
- 关联验证 Case：FC-6。

## T7: 注册模块 + 规范记录 + 证据 + 关闭

- 交付：`lib.rs` 注册 `pub mod acp;`；`specs/engine-session-runtime.md` 记录 ACP/PTY 边界；
  OpenSpec alignment + 证据齐备；Beads `homie-skh` 关闭。
- 关联验证 Case：FC-7。
