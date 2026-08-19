# Release Readiness — mcp-http-transport-integration-test

> change_id: `mcp-http-transport-integration-test` · Beads: `homie-kpw` · tag: `v0.8.2`

## 目标

落实 `mcp-http-transport-unified`（v0.8.0）延后的 T4.2，为 daemon 内嵌 MCP `streamable-http`
传输层补真实监听 + 真实 HTTP 请求的集成测试。

## 变更范围

- 新增 `homie/crates/homie-engine/tests/mcp_http.rs`（`#[tokio::test]` 集成测试）。
- `homie/crates/homie-engine/Cargo.toml` 新增 dev-dependency `reqwest`（0.12，json）。
- `specs/mcp-transport.md` 增补「验证」小节记录集成测试覆盖。

## 验证证据

### 测试执行

```text
$ cargo test -p homie-engine --test mcp_http --offline
running 1 test
test auth_and_rpc_surface ... ok
test result: ok. 1 passed; 0 failed

$ cargo test -p homie-engine --offline
test result: ok. 301 passed; 0 failed; 3 ignored
```

### 断言覆盖（FR-2）

| 断言 | 结果 |
|------|------|
| 无 token → `401` | ✅ |
| 错误 token → `401` | ✅ |
| `initialize` 回显 `protocolVersion` + `serverInfo.name == "homie"` | ✅ |
| `tools/list` 返回非空 `tools` | ✅ |
| `ping` 返回 `{}` | ✅ |
| 事实文件 `mcp-http.json` 只含 `url`、全文不含 `runtime.token` | ✅ |

### 质量门

- `cargo fmt --all --check`：干净（`exit 0`）。
- `cargo clippy -p homie-engine --all-targets --offline`：新增测试文件无 clippy 告警；lib 侧
  告警为历史遗留（`registry/store.rs` 等），非本次变更引入。

## 已知限制 / 延后

- T4.3 端到端（真实 Codex/Claude 经 HTTP MCP 编排）仍依赖真实 agent 运行时，未在本 change 覆盖。
- 传输层实现（`http.rs`）未做改动；若后续发现真实缺陷，另开 PRD 修复。

## 结论

MCP HTTP 传输层已具备真实监听 + 真实 HTTP 请求的集成测试，覆盖鉴权、核心 JSON-RPC 方法与
bearer secret 不落盘保密性。验收标准全部满足，可发布。
