# Tasks — mcp-http-transport-integration-test

## T1 — 测试 harness + 依赖
- [x] `homie-engine` dev-dependencies 增 `reqwest`（0.12，json）。
- [x] 新增 `tests/mcp_http.rs`，用 `mcp::http::start` 起真实 listener + reqwest POST。

## T2 — 断言
- [x] 无 / 错误 token → 401。
- [x] initialize 回显 protocolVersion + serverInfo.name == homie。
- [x] tools/list 返回非空。
- [x] ping 返回 {}。
- [x] 事实文件只含 url、不含 token。

## T3 — 验证 + 证据
- [x] `cargo test -p homie-engine --test mcp_http --offline` 绿。
- [x] `cargo test -p homie-engine --offline` 全绿。
- [x] release-readiness 报告。

## T4 — 提交
- [ ] commit + tag `v0.8.2` + 关闭 Beads `homie-kpw` + push。
