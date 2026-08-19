# Plan — mcp-http-transport-integration-test

## 目标

为 `homie-engine/src/mcp/http.rs` 传输层补集成测试，覆盖鉴权（401）、initialize、
tools/list、ping 与事实文件安全，落实 `mcp-http-transport-unified` 的 T4.2 延后项。

## 影响范围

- `homie/crates/homie-engine/Cargo.toml`：dev-dependencies 增 `reqwest`（0.12，json）。
- 新增 `homie/crates/homie-engine/tests/mcp_http.rs`。

## 验证

- `cargo test -p homie-engine --test mcp_http --offline`
- `cargo test -p homie-engine --offline`
