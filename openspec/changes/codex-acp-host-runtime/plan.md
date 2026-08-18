# OpenSpec Plan — codex-acp-host-runtime

## 1. 变更概述

本变更是 **Codex ACP host harness 的首个真实代码纵向切片**。在 `homie-engine` 中落地
一个**通用 ACP host（stdio JSON-RPC 2.0）**，覆盖协议 DTO、newline-delimited framing、
host 循环（spawn 子进程 + `initialize` 握手 + request/response id 关联 + `session/update`
通知派发）、approval 四态记忆，以及实现 `AgentDriverControl` 的 `AcpDriver`。

本阶段**不依赖 `codex-acp` crate 作为库**：`codex-acp` 作为可配置 ACP server 二进制路径被
`AcpHost::spawn` 启动，harness 通过 stdio JSON-RPC 与其通信。这统一支撑任意 ACP server，
且不引入需要网络拉取的库依赖。

## 2. 模块划分与依赖

```text
homie/crates/homie-engine/src/acp/
├── mod.rs        # 模块根，re-export
├── protocol.rs   # JSON-RPC 2.0 DTO + 方法/kind 常量 + classify_inbound
├── frame.rs      # newline-delimited JSON framing（encode/decode/read_line）
├── host.rs       # AcpHost + AcpClient trait + 后台 reader 线程
├── approval.rs   # PermissionDecision 四态 + ApprovalMemory
└── driver.rs     # AcpDriver: AgentDriverControl（capabilities 由 initialize 填充）
```

依赖（已存在，无需新增 crate）：

- `serde` / `serde_json`（workspace 依赖，已有）；
- `homie-proto::DriverCapabilities`（`typed-agent-driver-capabilities` 已定义）；
- `crate::driver::{AgentDriverControl, DriverError, DriverResult, ModelOption}`（已存在）。

新增集成测试 `homie/crates/homie-engine/tests/acp_host.rs`（`harness = false`，spawn 自身
二进制以 `--acp-fake-server` 模式扮演 agent 侧）。

## 3. 层级关系

| 层 | 产物 |
|----|------|
| 需求 | `prd-spec/features/codex-acp-host-runtime/2026-08-18-codex-acp-host-runtime-design.md` |
| 规范 | `specs/engine-session-runtime.md`（记录 ACP 与 PTY authority 边界，不改变 runtime 合同） |
| 执行 | 本 OpenSpec plan/tasks/alignment + `homie-engine/src/acp/*` |
| 证据 | `docs/verification/codex-acp-host-runtime/` |

## 4. 与既有权威的关系

- PTY/holder 仍是 session 生命周期、环境、screen state、子进程监督的**权威**（见
  `specs/engine-session-runtime.md` §2/§3）。
- ACP harness 是**附加的结构化控制面**：把 typed control（cancel/steer/respond_permission）
  映射为 ACP 方法，不接管 PTY/holder 的既有职责。
- `typed-agent-driver-capabilities` 的 `driver.rs` 行为对非 ACP agent **不变**；`AcpDriver`
  是新增的 provider 实现，不在本阶段接入 `session.spawn` 路径。

## 5. 后续 child Bead（本变更只声明，不实现）

- session-driver 集成：把 `AcpDriver` 接入 `session.spawn`（属后续 Bead）。
- `fs/read_text_file` / `fs/update_text_file` 文件代理。
- GPUI chat canvas / composer / transcript（`chat-surface-gpui`）。
- 真实 provider（Claude/OpenCode ACP）接入与模型发现（`available_commands_update`）。
