# 发布就绪报告 — app-navigation-module-split

## 变更概述

将 `homie/crates/homie-app/src/navigation.rs`（1,376 行，GPUI 导航覆盖层 God Module）
机械拆分为目录化聚焦子模块：目录索引/扫描、覆盖层渲染、测试各自下沉，`mod.rs` 收尾为
facade。公共 API 与运行时行为完全不变，引用方零改动，单文件 < 800 行。

- change_id：`app-navigation-module-split`
- Beads：`homie-itl`
- 类型：task（机械拆分，行为不变）
- 上游：`architecture-audit-governance-2026-08`（模块降熵序列延续）

## 模块划分

```text
navigation/
├── mod.rs      facade（类型/常量 + 生命周期/排序/命令执行 + trait 实现 + 共享查询字段工具），646 行
├── index.rs    目录索引/扫描 + Quick Open 快照构建，126 行
├── render.rs   覆盖层行/区块/浮层渲染 + 渲染辅助自由函数，519 行
└── tests.rs    既有测试（原样下沉，#[cfg(test)]），109 行
```

依赖方向：公共类型/常量全部在 `mod.rs` 定义；`index.rs` 与 `render.rs` 读取 facade 类型与
私有字段；`mod.rs` 反向调用 `index.rs`/`render.rs` 的 `pub(super)` 方法。引用方
（`terminal_pane`、`code_viewer`、`inspector`、`sidebar`、`surface_shell`、`launcher`）
仅依赖 `crate::navigation::{...}` 公共 API，路径不变。

## 交付切片 T1–T4

| 切片 | 内容 | 状态 |
|------|------|------|
| T1 | 抽取渲染 render.rs | 完成 |
| T2 | 抽取目录索引 index.rs | 完成 |
| T3 | 抽取测试 tests.rs + mod.rs 收尾 facade | 完成 |
| T4 | 全量验证 + code review + release readiness | 完成 |

## 验证证据

| 验证项 | 命令 | 结果 |
|--------|------|------|
| 编译检查 | `cargo check -p homie-app --offline` | 通过，0 警告 |
| 格式检查 | `cargo fmt --all --check` | 通过 |
| 静态检查 | `cargo clippy -p homie-app --all-targets --offline` | 通过，0 警告 |
| 全量测试 | `cargo test -p homie-app --offline` | **301 passed / 2 failed / 1 ignored**（2 failed 为沙箱 socket bind EPERM，非本变更引入） |
| 沙箱外复测 | `cargo test -p homie-app --offline daemon_launch::tests` | 8 passed / 0 failed（确认 2 个失败纯属沙箱限制） |
| 单文件行数 | `wc -l` | mod 646 / index 126 / render 519 / tests 109，均 < 800 |

- `navigation` 相关 6 个测试（`cargo test -p homie-app --offline navigation::tests`）全部原样
  通过，布局/滚动/排序/命令行为与拆分前等价。
- 公共 API 兼容性：`NavigationOverlay`、`NavigationEvent`、`ToggleCommandPalette`、
  `ToggleQuickOpen`、`query_label`、`CARET` 路径不变；`cargo check -p homie-app` 编译通过
  证明可达性不变。
- 引用方零改动（`git status` 确认仅删旧单文件、新增目录）。
- 可见性管控：仅跨模块调用方法（`render_overlay`、`load_cached_index`、`refresh_directory_index`、
  `relative_parent`）升为 `pub(super)`，无 `pub` 泄漏到 crate 外。

## 已知限制与后续

- 无已知限制。
- 机械重构未改变任何运行时行为、公共 API 不变；纯职责搬迁，不做向后兼容。
- 后续候选（`store/mod.rs` 2,434 行、`terminal_pane/mod.rs` 1,486 行）依赖密、风险更高，
  建议作为更谨慎的独立切片处理。

## 结论

所有验收标准（C1–C6）均已满足，验证证据齐备，可发布。
