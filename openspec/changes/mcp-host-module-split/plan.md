# OpenSpec Plan — mcp-host-module-split

## 目标

将 `homie/crates/homie-engine/src/mcp/host.rs`（863 行）按关注点拆分为 3 个聚焦子模块：
`lineage.rs`（调用方会话谱系与关系判定）、`registry.rs`（注册表宿主 + 工具分发 + 辅助方法）。
`mod.rs` 保留模块文档 + 子模块声明 + `pub use` 再导出 + `SESSION_ID_ENV` 常量。所有类型/函数/
常量逐字迁移，公共 API 不变，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（facade，`pub use` 再导出）→ `registry.rs` → `lineage.rs`。
- `registry.rs` 依赖 `lineage.rs`（`Lineage`/`Relation`/`relation_word`）与 `SESSION_ID_ENV`，
  并以显式 `use` 引入。
- `RegistryHost` 保持 `pub` 并经 `pub use` 再导出；`Relation`/`relation_word`/`Lineage` 因跨模块
  共享提升为 `pub(crate)`。
- `SESSION_ID_ENV` 保持 `pub const`，保留在 `mod.rs`。
- 无生产代码语义变更，无外部 API 泄漏。

## 交付切片

- T1：类型/函数/常量边界扫描，定位 `Relation`/`relation_word`/`Lineage`（7 方法）、
  `RegistryHost`（结构体 + 9 方法 + `impl ToolHost` 20 工具）、`required_str`/`opt_strings`/
  `render_report`/`status_word` + 4 常量的闭合边界。
- T2：生成 `lineage.rs`/`registry.rs` 子模块。
- T3：重建 `mod.rs`（文档 + 声明 + 再导出 + 常量），删除旧 `host.rs`，编译验证。
- T4：全量验证（fmt/check/clippy/build/workspace-check/test）。
- T5：code review + release readiness 证据。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/mcp-host-module-split/`。
