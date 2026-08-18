# OpenSpec Plan — app-sidebar-view-module-split

## 概述

将 `homie/crates/homie-app/src/sidebar/view.rs`（约 4,310 行）机械拆分为 `sidebar/` 下的聚焦子模块目录，把渲染方法按 section / popover / dispatch 分类，并把纯投影与渲染辅助函数分离。公共 API 与运行时行为完全不变。

## 模块划分与依赖

```text
sidebar/
├── mod.rs           facade（导入 + pub use 再导出 + const PREVIEW_USAGE），依赖全部子模块
├── view.rs          Sidebar 结构体 + EventEmitter/Focusable + 核心 impl Sidebar + impl Render
├── sections.rs      impl Sidebar：project/session/archived 各 section 渲染方法
├── popover.rs       impl Sidebar：popover 相关方法
├── commands.rs      impl Sidebar：命令/选择/拖拽 dispatch 方法
├── render_helpers.rs 游离渲染辅助函数（icon_button/indent_rails/state_chip/menu_row/usage_row…）
├── projection.rs    纯投影函数（count_label/display_title/status_state/shortcut_ranks/clamp_path…）
├── tests.rs         旧内联测试 16 个原样迁移（use super::*）
├── state.rs         已有（不变）
├── picker_logic.rs  已有（不变）
└── fixture.rs       已有（不变）
```

依赖方向：`sections/popover/commands → view（Sidebar 实体 + pub(crate) 方法）`；`mod → {view, sections, popover, commands, projection, render_helpers, state, picker_logic, fixture}`。

## 任务清单

| Task | 描述 | 验收 | 关联验证 Case |
|------|------|------|---------------|
| S1 | 抽取纯投影 projection.rs | cargo check 全绿 | C1 |
| S2 | 抽取渲染辅助 render_helpers.rs | cargo check 全绿 | C2 |
| S3 | 抽取 section 渲染 sections.rs | cargo check 全绿 | C3 |
| S4 | 抽取 popover 方法 popover.rs | cargo check 全绿 | C4 |
| S5 | 抽取 dispatch commands.rs + tests.rs + facade 收尾 | cargo test 全绿 + fmt/check | C5 |

## 验证口径

- `cargo check -p homie-app`（0 警告）
- `cargo fmt --check`
- `cargo test -p homie-app`（16 个 sidebar 相关测试原样迁移，行为等价）
