# 功能验证 Case 清单 — codex-acp-host-runtime

本变更为 Rust 代码实现，验证 Case 验证协议/framing/host/approval/driver 的运行时行为。

## FC-1: ACP JSON-RPC 2.0 协议 DTO

- 断言：`JsonRpcRequest/Response/Notification/RpcError` serde 往返一致；
  `classify_inbound` 正确分类 response/notification/request；未知 kind 容忍；非法 JSON 返回
  `Err` 而非 panic。
- 预期：`protocol.rs` 单测全绿（request_round_trips / response_result_and_error_round_trip /
  classify_response_notification_and_request / unknown_session_update_kind_is_tolerated /
  malformed_frame_is_an_error_not_a_panic）。

## FC-2: newline-delimited framing

- 断言：`encode` 恰好追加一个 `\n`；`decode` 往返一致；`read_line` 跳过空行/空白行、容忍
  CRLF、EOF 返回 `None`。
- 预期：`frame.rs` 单测全绿。

## FC-3: ACP host 循环 + id 关联 + 通知派发

- 断言：`AcpHost` spawn/from_stream 可用；`request` 按 id 关联响应；`session/update` 通知
  通过 `try_recv_notification` 派发；`Drop` 先 kill child 再 join reader（不挂起）。
- 预期：`host.rs` 逻辑正确，E2E 集成测试（FC-6）覆盖 spawn 路径。

## FC-4: approval 四态记忆

- 断言：once 决策不记忆、always 决策按 kind 召回、后写覆盖、option_id 稳定。
- 预期：`approval.rs` 单测全绿。

## FC-5: AcpDriver 方法映射

- 断言：cancel→`session/stop`、steer→`session/prompt`（text block）、respond_permission→
  `session/respond_permission`、model_options→`unsupported`；capabilities 由构造参数提供。
- 预期：`driver.rs` 单测（RecordingClient）全绿。

## FC-6: fake ACP server 端到端

- 断言：spawn 自身二进制 `--acp-fake-server`，走真实 `AcpHost::spawn` 子进程路径，
  initialize→session/new→session/prompt→session/update 通知→session/stop 全链路成功。
- 预期：`tests/acp_host.rs`（harness=false）进程退出码 0，输出
  `acp_host: end-to-end host loop passed`。

## FC-7: 模块注册 + 规范记录 + 对齐 + 关闭

- 断言：`lib.rs` 注册 `pub mod acp;`；`specs/engine-session-runtime.md` 记录 ACP/PTY 边界；
  OpenSpec 三文件齐备；Beads `homie-skh` 关闭。
- 预期：`cargo check --workspace` / `cargo fmt --all --check` / `cargo test -p homie-engine` 全绿。

## 覆盖矩阵

| PRD 需求项 | 验证 Case |
|-----------|-----------|
| FR-1 / FR-6 | FC-1 |
| FR-2 | FC-2 |
| FR-3 | FC-3 / FC-6 |
| FR-4 | FC-5 |
| FR-5 | FC-4 |
| 验收 §8.2 | FC-6 |
| 验收 §8.6 | FC-7 |
