# Release Readiness — remote-manager-module-split

## 变更摘要

将 `homie/crates/homie-engine/src/remote/manager.rs`（1099 行）按关注点拆分为 4 个聚焦子模块。
`ArtifactCatalog`（3 方法 + `verify_required_helper_probe` + `ArtifactManifest`/`ArtifactEntry`）、
`RemoteManager`（结构体 + `CurrentHelper` + 单 `impl` 20 方法 + `InstalledHelper`）、控制目录 3 函数
（`validate_control_dir_if_present`/`normalized_control_dir`/`effective_uid`）、工具 4 函数
（`parse_json_line`/`random_hex`/`classify_persistence`/`persistence_key`）全部逐字迁移。公共
`ArtifactCatalog`/`RemoteManager`/`InstalledHelper` 保持 `pub` 并经 `mod.rs` 的 `pub use` 再导出
（公共 API 不变）。跨模块共享的私有辅助提升为 `pub(crate)`；仅本模块使用的辅助保持私有。生产代码
语义零变更。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 36 | facade：模块文档 + 子模块声明 + `pub use` 再导出 + 6 常量 + `#[cfg(test)] mod tests` |
| `catalog.rs` | 144 | verify_required_helper_probe + ArtifactCatalog + ArtifactManifest + ArtifactEntry |
| `runtime.rs` | 617 | RemoteManager 结构体 + CurrentHelper + 单 impl（20 方法）+ InstalledHelper |
| `control_dir.rs` | 48 | validate_control_dir_if_present + normalized_control_dir + effective_uid |
| `util.rs` | 45 | parse_json_line + random_hex + classify_persistence + persistence_key |
| `tests.rs` | 260 | 8 测试 + hex_sha256 |

全部单文件 < 800 行（最大 `runtime.rs` 617 行）。

## 类型/函数逐字迁移

- `ArtifactCatalog` + 3 方法、`RemoteManager` + 单 `impl` 20 方法、`InstalledHelper`、控制目录 3 函数、
  工具 4 函数、6 常量全部逐字迁移，远程 Helper 引导/会话管理 RPC 语义零变更。
- 公共 `ArtifactCatalog`/`RemoteManager`/`InstalledHelper` 保持 `pub`，经 `pub use` 再导出。
- `verify_required_helper_probe`（catalog）、`normalized_control_dir`/`validate_control_dir_if_present`
  （control_dir）、`parse_json_line`/`random_hex`/`classify_persistence`/`persistence_key`（util）因被
  `runtime.rs` 跨模块调用，提升为 `pub(crate)`。
- `ArtifactCatalog::artifact` 因被 `runtime.rs` 调用，提升为 `pub(crate)`。
- `ArtifactCatalog.artifacts` 字段因 tests 直接构造结构体字面量，提升为 `pub(crate)`。
- `ArtifactManifest`/`ArtifactEntry` 仅 `catalog.rs` 内部使用，保持私有。
- `effective_uid` 仅 `control_dir.rs` 内部使用，保持私有。
- `CurrentHelper` 仅 `runtime.rs` 内部使用，保持私有。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-engine --offline` | ✅ 通过（0 警告） |
| `cargo clippy -p homie-engine --all-targets --offline` | ✅ 0 警告 |
| `cargo build -p homie-engine --offline` | ✅ 通过 |
| `cargo check --workspace --offline` | ✅ 通过 |
| `cargo test -p homie-engine --offline` | ✅ 全绿（unit 303 passed / 0 failed / 3 ignored + 集成测试全绿；4 个 socket/transport 用例沙箱内权限失败，非沙箱全绿） |
| 引用方零改动 | ✅ 仅 `remote/manager.rs` → `remote/manager/` 目录 + 文档 |

## 已知限制 / 延期

- 无。后续候选（按行数排序）：`remote/client.rs`（约 14k 行以下，无需拆分）等，当前无超过 800 行的
  单文件候选。
