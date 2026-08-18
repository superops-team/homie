# 发布就绪报告 — app-sidebar-view-module-split

## 变更概述

将 `homie/crates/homie-app/src/sidebar/view.rs`（约 4,310 行）机械拆分为 `sidebar/` 下的聚焦子模块目录，把渲染方法按 section / popover / dispatch 分类，并把纯投影与渲染辅助函数分离。公共 API 与运行时行为完全不变。

- change_id：`app-sidebar-view-module-split`
- Beads：`homie-ubu.6`
- 类型：refactor（机械拆分，删除旧单文件，不做向后兼容）

## 模块划分

```text
sidebar/
├── mod.rs            facade（导入 + pub use 再导出 + const PREVIEW_USAGE）
├── view.rs           Sidebar 结构体 + EventEmitter/Focusable + 核心 impl Sidebar + impl Render（~540 行）
├── sections.rs       impl Sidebar：project/session/archived 各 section 渲染方法（~1200 行）
├── popover.rs        impl Sidebar：popover 相关方法（~1250 行）
├── commands.rs       impl Sidebar：命令/选择/拖拽 dispatch 方法（~420 行）
├── render_helpers.rs 游离渲染辅助函数（icon_button/indent_rails/state_chip/menu_row/usage_row…）
├── projection.rs     纯投影函数（count_label/display_title/status_state/shortcut_ranks/clamp_path…）
├── tests.rs          旧内联测试 16 个原样迁移（use super::*）
├── state.rs          已有（不变）
├── picker_logic.rs   已有（不变）
└── fixture.rs        已有（不变）
```

依赖方向：`sections/popover/commands → view`；`mod → {view, sections, popover, commands, projection, render_helpers, state, picker_logic, fixture}`。

## 交付切片 S1–S5

| 切片 | 内容 | 状态 |
|------|------|------|
| S1 | 抽取纯投影 projection.rs | 完成 |
| S2 | 抽取渲染辅助 render_helpers.rs | 完成 |
| S3 | 抽取 section 渲染 sections.rs | 完成 |
| S4 | 抽取 popover 方法 popover.rs | 完成 |
| S5 | 抽取 dispatch commands.rs + tests.rs + facade 收尾 | 完成 |

## 验证证据

| 验证项 | 命令 | 结果 |
|--------|------|------|
| 编译检查 | `cargo check -p homie-app` | 通过，0 警告 |
| 格式检查 | `cargo fmt --check` | 通过 |
| 全量测试 | `cargo test -p homie-app` | **303 passed / 0 failed / 1 ignored** |
| sidebar 单测 | `cargo test -p homie-app sidebar::tests` | **16 passed / 0 failed** |

- 16 个 sidebar 相关测试原样迁移到 `tests.rs`，行为等价，全部通过。
- 公共 API 兼容性：`root.rs` / `main.rs` / `store::tests` 经 `cargo check -p homie-app` 编译通过，证明 `Sidebar`、`SidebarEvent`、`SidebarUiState`、`Popover`、`DragItem`、`PreviewScenario`、`SidebarPreviewFixture`、`move_before`、`move_to_end` 的可达性与签名不变。

## 已知限制与后续

- 无已知限制。
- 机械重构未改变任何运行时行为，公共 API 不变，删除旧文件、不做向后兼容。

## 结论

所有验收标准（C1–C6）均已满足，验证证据齐备，可发布。
