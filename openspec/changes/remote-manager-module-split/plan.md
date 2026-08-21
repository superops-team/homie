# OpenSpec Plan — remote-manager-module-split

## 目标

将 `homie/crates/homie-engine/src/remote/manager.rs`（1099 行）按关注点拆分为 4 个聚焦子模块：
`catalog.rs`（制品目录与清单）、`runtime.rs`（远程管理器 + Helper 引导/会话管理）、
`control_dir.rs`（SSH 控制目录校验与规范化）、`util.rs`（JSON 行解析 / 随机十六进制 / 持久化分类）。
`mod.rs` 保留模块文档 + 子模块声明 + `pub use` 再导出 + 6 个常量。所有类型/函数/常量逐字迁移，
公共 API 不变，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（facade，`pub use` 再导出 + 常量）→ `runtime.rs` → `catalog.rs`/`control_dir.rs`/`util.rs`。
- `runtime.rs` 依赖 `catalog.rs`（`ArtifactCatalog`/`verify_required_helper_probe`）、
  `control_dir.rs`（`normalized_control_dir`/`validate_control_dir_if_present`）、`util.rs`
  （`parse_json_line`/`random_hex`/`classify_persistence`/`persistence_key`）与 6 个常量，并以显式 `use` 引入。
- 跨模块共享的私有辅助从私有提升为 `pub(crate)`；仅本模块使用的辅助保持私有。
- `ArtifactCatalog`/`RemoteManager`/`InstalledHelper` 保持 `pub` 并经 `pub use` 再导出，
  `remote::manager::*` 路径不变（`remote/mod.rs` 的 `pub mod manager;` 无需改动）。
- tests 模块与 `hex_sha256` 保留在 `tests.rs`，经 `mod.rs` 的 `#[cfg(test)] pub(crate) use` 再导出与
  显式 `use` 引入所需符号。
- 无生产代码语义变更，无外部 API 泄漏。

## 交付切片

- T1：类型/函数/常量边界扫描，定位 `ArtifactCatalog`（3 方法 + 2 serde 结构体）、`RemoteManager`
  （结构体 + `CurrentHelper` + 单 `impl` 20 方法 + `InstalledHelper`）、控制目录 3 函数、工具 4 函数、
  6 常量、8 测试 + `hex_sha256` 的闭合边界。
- T2：生成 `catalog.rs`/`runtime.rs`/`control_dir.rs`/`util.rs` 子模块。
- T3：重建 `mod.rs`（文档 + 声明 + 再导出 + 常量），删除旧 `manager.rs`，编译验证。
- T4：全量验证（fmt/check/clippy/build/workspace-check/test）。
- T5：code review + release readiness 证据。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/remote-manager-module-split/`。
