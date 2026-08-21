# PRD — remote-manager-module-split

## 背景

`homie/crates/homie-engine/src/remote/manager.rs`（1099 行）是远程 Helper 引导与管理 RPC 的单文件模块，
同时承载五类关注点：制品目录与清单（`ArtifactCatalog`/`ArtifactManifest`/`ArtifactEntry`/`verify_required_helper_probe`）、
远程管理器本体（`RemoteManager` 结构体 + 单 `impl` 20 方法 + `CurrentHelper` + `InstalledHelper`）、
控制目录校验与规范化（`validate_control_dir_if_present`/`normalized_control_dir`/`effective_uid`）、
RPC/工具辅助（`parse_json_line`/`random_hex`/`classify_persistence`/`persistence_key`），以及 8 个测试 + `hex_sha256` 辅助。
单文件超过 800 行阈值，且五类关注点彼此独立，阅读与变更成本高，违背仓库「组件模块化、关注点清晰」原则。

## 目标

将 `manager.rs` 按关注点拆分为目录化聚焦子模块，公共 API 与运行时行为零变更，引用方零改动，单文件 < 800 行。

## 非目标

- 不改变任何远程 Helper 引导/管理 RPC 的运行时行为。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动任何函数/结构体/常量签名语义。
- 不合并或重命名函数、结构体或常量。

## 用户场景

1. 开发者定位「制品目录与清单解析」时，聚焦在 `catalog.rs`。
2. 开发者定位「远程管理器 + Helper 引导/会话管理」时，聚焦在 `runtime.rs`。
3. 开发者定位「SSH 控制目录校验与规范化」时，聚焦在 `control_dir.rs`。
4. 开发者定位「JSON 行解析 / 随机十六进制 / 持久化分类」时，聚焦在 `util.rs`。

## 模块划分方案

```text
remote/manager/
├── mod.rs         facade：模块文档 + 子模块声明 + `pub use` 再导出 + 6 个常量
├── catalog.rs     verify_required_helper_probe + ArtifactCatalog + ArtifactManifest + ArtifactEntry
├── runtime.rs     RemoteManager 结构体 + CurrentHelper + 单 impl（20 方法）+ InstalledHelper
├── control_dir.rs validate_control_dir_if_present + normalized_control_dir + effective_uid
└── util.rs        parse_json_line + random_hex + classify_persistence + persistence_key
```

测试模块与 `hex_sha256` 辅助保留在 `mod.rs` 内的 `#[cfg(test)] mod tests`，通过 facade 的
`#[cfg(test)] pub(crate) use` 与显式 `use super::*` 引入所需符号。

## 可见性设计

- `ArtifactCatalog`/`RemoteManager`/`InstalledHelper` 为公共类型，经 `mod.rs` 的 `pub use` 再导出，
  保持 `remote::manager::*` 路径不变，公共 API 不变（`remote/mod.rs` 的 `pub mod manager;` 无需改动）。
- 6 个常量（`PROBE_TIMEOUT`/`UPLOAD_TIMEOUT`/`RPC_TIMEOUT`/`MAX_RPC_OUTPUT`/`MAX_ARTIFACT_BYTES`/
  `MAX_CONTROL_DIRECTORY_BYTES`）保留在 `mod.rs`，因被子模块跨文件使用，保持私有但对子模块可见
  （子模块可访问父模块私有项）。
- `verify_required_helper_probe`（catalog）、`normalized_control_dir`/`validate_control_dir_if_present`
  （control_dir）、`parse_json_line`/`random_hex`/`classify_persistence`/`persistence_key`（util）因被
  跨模块调用（runtime.rs 生产代码 + tests），从私有提升为 `pub(crate)`。
- `ArtifactManifest`/`ArtifactEntry` 仅 `catalog.rs` 内部使用，保持私有。
- `ArtifactCatalog` 的 `artifacts` 字段因 tests 直接构造结构体字面量，从私有提升为 `pub(crate)`。
- `effective_uid` 仅 `control_dir.rs` 内部使用，保持私有。
- 各子模块以显式 `use` 引入所需依赖。

## 影响面

- 仅 `remote/manager.rs` 的类型/函数/常量拆分为 4 个聚焦子模块 + facade，生产代码与其它模块零改动。
- `remote/mod.rs` 已使用 `pub mod manager;`，`manager.rs` 变目录 `manager/mod.rs` 后无需改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo fmt --all --check` 通过。
- `cargo check -p homie-engine --offline` 全绿。
- `cargo clippy -p homie-engine --all-targets --offline` 0 警告。
- `cargo build -p homie-engine --offline` 通过。
- `cargo check --workspace --offline` 全绿。
- `cargo test -p homie-engine --offline` 全绿（unit + 集成测试全绿；4 个 socket/transport 用例沙箱内
  权限失败，非沙箱环境全绿）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件（1099 行）拆为 4 子模块 + facade。
- C2：公共 API 不变，引用方零改动。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：类型/函数/常量逐字迁移，公共可见性不变，私有辅助仅内部提升为 `pub(crate)`。
- C6：release readiness 证据写入 `docs/verification/remote-manager-module-split/`。

## Beads

- `homie-ysa`
