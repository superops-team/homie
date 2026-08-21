# Release Readiness — mcp-host-module-split

## 变更摘要

将 `homie/crates/homie-engine/src/mcp/host.rs`（863 行）按关注点拆分为 3 个聚焦子模块。
`Relation`/`relation_word`/`Lineage`（7 方法）、`RegistryHost`（结构体 + 首个 `impl` 4 方法 +
`impl ToolHost` 20 工具 + 第二个 `impl` 7 方法）、辅助函数（`required_str`/`opt_strings`/
`render_report`/`status_word`）+ 4 常量全部逐字迁移。公共 `RegistryHost` 保持 `pub` 并经
`mod.rs` 的 `pub use` 再导出（公共 API 不变）。`Relation`/`relation_word`/`Lineage` 因跨模块
共享提升为 `pub(crate)`；`delivers_verbatim`/`descendants_of`/`ancestors_of` 仅 `lineage.rs`
内部使用，保持私有。生产代码语义零变更。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 13 | facade：模块文档 + 子模块声明 + `pub use registry::RegistryHost` + `SESSION_ID_ENV` |
| `lineage.rs` | 145 | Relation / relation_word / Lineage + new / record / children_of / descendants_of / ancestors_of / relation_to / frame |
| `registry.rs` | 721 | RegistryHost 结构体 + 首个 impl（new/with_holder/with_caller/registry）+ impl ToolHost（20 工具）+ 第二个 impl（require_caller/browser_call/spawn_agent/spawn_agent_remote/wait_for/wait_for_children/logs_dir）+ required_str/opt_strings/render_report/status_word + 4 常量 |

全部单文件 < 800 行（最大 `registry.rs` 721 行）。

## 类型/函数逐字迁移

- `Relation`/`relation_word`/`Lineage` + 7 方法、`RegistryHost` + 11 方法 + `impl ToolHost` 20
  工具、4 个辅助函数 + 4 个常量全部逐字迁移，MCP 工具分发语义零变更。
- 公共 `RegistryHost` 保持 `pub`，经 `pub use` 再导出；`SESSION_ID_ENV` 保持 `pub const`。
- `Relation`/`relation_word`/`Lineage` 及其方法因 `registry.rs` 跨模块调用，提升为 `pub(crate)`。
- `delivers_verbatim`/`descendants_of`/`ancestors_of` 仅 `lineage.rs` 内部使用，保持私有。
- `required_str`/`opt_strings`/`render_report`/`status_word` 及 4 常量仅 `registry.rs` 使用，
  保持私有并随宿主下放。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-engine --offline` | ✅ 通过（0 警告） |
| `cargo clippy -p homie-engine --all-targets --offline` | ✅ 0 警告 |
| `cargo build -p homie-engine --offline` | ✅ 通过 |
| `cargo check --workspace --offline` | ✅ 通过 |
| `cargo test -p homie-engine --offline` | ✅ 全绿（unit 303 passed / 0 failed / 3 ignored + 集成测试全绿；4 个 socket/transport 用例沙箱内权限失败，非沙箱全绿） |
| 引用方零改动 | ✅ 仅 `mcp/host.rs` → `mcp/host/` 目录 + 文档 |

## 已知限制 / 延期

- 无。后续候选（按行数排序）：`remote/manager.rs`（1099 行）等。
