# OpenSpec Tasks — remote-manager-module-split

## T1 类型/函数/常量边界扫描

- [x] 定位制品目录关注点（`ArtifactCatalog` + `from_manifest`/`from_native_helper`/`artifact` +
  `verify_required_helper_probe` + `ArtifactManifest`/`ArtifactEntry`）、远程管理器关注点
  （`RemoteManager` + `CurrentHelper` + 单 `impl` 20 方法 + `InstalledHelper`）、控制目录关注点
  （`validate_control_dir_if_present`/`normalized_control_dir`/`effective_uid`）、工具关注点
  （`parse_json_line`/`random_hex`/`classify_persistence`/`persistence_key`）、6 常量、8 测试 +
  `hex_sha256` 边界。
- 验收：全部解析，无缺失/重复。关联 C2/C5。

## T2 生成子模块

- [x] `catalog.rs`（ArtifactCatalog + 3 方法 + verify_required_helper_probe + ArtifactManifest/
  ArtifactEntry）、`runtime.rs`（RemoteManager + CurrentHelper + 单 impl 20 方法 + InstalledHelper）、
  `control_dir.rs`（3 函数）、`util.rs`（4 函数）。
- 验收：函数体逐字迁移，`pub` 保持 `pub`，跨模块私有辅助提升 `pub(crate)`。关联 C1/C5。

## T3 重建 mod.rs + 编译验证

- [x] 移除旧 `manager.rs`，新增 `remote/manager/mod.rs`（文档 + 声明 + `pub use` 再导出 + 6 常量 +
  `#[cfg(test)] mod tests`）。
- [x] `cargo check -p homie-engine --offline` 全绿（0 警告）。
- 验收：通过。关联 C2/C4。

## T4 全量验证

- [x] `cargo fmt --all --check`
- [x] `cargo check -p homie-engine --offline`
- [x] `cargo clippy -p homie-engine --all-targets --offline`
- [x] `cargo build -p homie-engine --offline`
- [x] `cargo check --workspace --offline`
- [x] `cargo test -p homie-engine --offline`（unit 303 passed + 集成测试全绿；4 个 socket/transport
  用例沙箱内权限失败，非沙箱全绿）
- 验收：全部通过。关联 C3/C4。

## T5 code review + release readiness

- [x] code review：拆分边界清晰、类型/函数逐字迁移、无行为变更、可见性正确、单文件 < 800 行。
- [x] release readiness 证据写入 `docs/verification/remote-manager-module-split/`。
- 验收：通过。关联 C6。
