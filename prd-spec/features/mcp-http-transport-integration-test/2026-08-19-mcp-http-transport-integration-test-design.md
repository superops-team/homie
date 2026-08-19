# MCP HTTP transport 集成测试补全设计文档

## 1. 背景

`mcp-http-transport-unified`（v0.8.0，Beads `homie-gyj`）将 MCP 工具 transport 统一为
daemon 内嵌 `streamable-http`（`POST /mcp` @ `127.0.0.1`），并新增
`homie-engine/src/mcp/http.rs` 作为传输层（鉴权、`X-Homie-Session-Id` 解析、监听生命周期）。

release 报告 §3 明确将 **T4.2 集成测试**（daemon 起 endpoint，POST
initialize/tools/list/tools/call + 401）列为延后项：当时只以单测覆盖了 JSON-RPC 核心
（`mod.rs`）与 schema（`tools.rs`），**`http.rs` 传输层零测试**。

本 PRD 落实 T4.2，为 HTTP 传输层补上真实监听 + 真实 HTTP 请求的集成测试。

## 2. 目标

- 为 `homie-engine/src/mcp/http.rs` 补集成测试，覆盖：
  1. 无 / 错误 bearer token → `401`。
  2. 有效 token → `initialize` 返回 `serverInfo` 并回显协议版本。
  3. 有效 token → `tools/list` 返回非空工具数组。
  4. 有效 token → `ping` 返回 `{}`。
  5. 注入事实文件 `mcp-http.json` 只含 URL、不含 bearer token（内存态 secret 不落盘）。

## 3. 非目标

- 不改 `http.rs` 传输层实现（仅补测试；实现若有真实缺陷再另行 PRD 修复）。
- 不做 T4.3 端到端（真实 Codex/Claude 经 HTTP MCP 编排），仍依赖真实 agent 运行时。
- 不新增对 MCP 协议的协议级变更。

## 4. 需求

### FR-1 集成测试 harness

- 新增 `homie/crates/homie-engine/tests/mcp_http.rs`（`#![cfg(unix)]`，`#[tokio::test]`）。
- 使用 `mcp::http::start(registry, logs_dir, None, kinds, &inject_dir)` 起真实 listener，
  从返回的 `McpRuntime { base_url, token }` 取地址与 in-memory token。
- 用真实 HTTP 客户端（`reqwest` dev-dependency）POST `/mcp`，并带重试等待 listener 就绪。

### FR-2 断言

- 无 token / 错误 token → `401`。
- `initialize`（带 `protocolVersion`）→ `200`，`result.serverInfo.name == "homie"`，
  回显 `protocolVersion`。
- `tools/list` → `200`，`result.tools` 非空。
- `ping` → `200`，`result == {}`。
- 事实文件 `mcp-http.json` 解析后 `url` 为字符串，且全文不含 `runtime.token`。

## 5. 受影响 Specs

- `specs/mcp-transport.md`：若其「验证」小节列出 T4.2 为延后，更新为「已有集成测试」。
- `docs/verification/mcp-http-transport-unified/release-readiness-report.md`：不动历史报告，
  本 change 独立成新证据目录。

## 6. 测试计划

- `cargo test -p homie-engine --test mcp_http --offline` 全绿。
- `cargo test -p homie-engine --offline` 全量绿。

## 7. 验收标准

- `mcp_http.rs` 集成测试全绿，覆盖上述 5 类断言。
- 无 flake（listener 就绪重试 + 短超时）。
- 证据齐全：`docs/verification/mcp-http-transport-integration-test/`。

## 8. Beads 追踪

- change_id: `mcp-http-transport-integration-test`
- 类型: feature（verification hardening）
- 优先级: P2（patch 级）
