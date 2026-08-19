# Tasks — remote-capability-mcpstdio-removal

## T1 — 删除死变体
- [x] 删除 `RemoteCapability::McpStdio` 变体。
- [x] 删除 `wire_name` 的 `McpStdio` 分支。

## T2 — 验证
- [x] `cargo test -p homie-proto -p homie-remote -p homie-engine --offline`（homie-proto/engine 绿；homie-remote 1 项既有环境 flake 与本次无关）。
- [x] `cargo clippy -p homie-proto --all-targets --offline` 无告警。
- [x] 确认 `McpStdio` / `mcp-stdio` 零引用。

## T3 — 证据 + 提交
- [x] release-readiness 报告。
- [ ] commit + tag `v0.8.1` + 关闭 Beads `homie-cqm` + push。
