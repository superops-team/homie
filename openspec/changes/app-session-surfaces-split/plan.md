# OpenSpec Plan — app-session-surfaces-split

## 概述

将 `homie/crates/homie-app/src/session_surfaces.rs`（1,657 行）机械拆分为目录化聚焦子模块：
overview 渲染、switcher 渲染、投影自由函数、测试各自下沉，`mod.rs` 收尾为 facade。
公共 API 与运行时渲染行为完全不变，单文件 < 800 行。

## 模块划分与依赖

```text
session_surfaces/
├── mod.rs           facade（SessionSurfaces 结构 + 非渲染 impl + 事件路由 + impl Render +
│                    render_grid_or_logo/status_glyph 共享辅助 + mod 声明 + switcher_key re-export），< 800
├── overview.rs      overview chrome 渲染（render_overview/mode_button/filter_chip/
│                    overview_empty_state/overview_board/overview_list），< 800
├── overview_card.rs overview 卡片/行渲染（overview_card/overview_list_row/bulk_close_bar），< 800
├── switcher.rs      switcher 渲染（render_switcher），< 800
├── projection.rs    投影自由函数（switcher_key/ui_agent_kind/ui_status_state/status_color/
│                    state_badge/state_badge_color/clamp_branch），< 800
└── tests.rs         既有测试（原样下沉，#[cfg(test)]）
```

依赖方向：

- `overview → mod`（`colors`/`render_grid_or_logo`/`status_glyph`/`store`/scroll handle/常量）
- `overview_card → mod`（`colors`/`render_grid_or_logo`/`status_glyph`/`store`/常量）
- `switcher → mod`（`colors`/`render_grid_or_logo`/`status_glyph`/`store`/常量）
- `overview_card → projection`（`status_color`/`state_badge`/`state_badge_color`）
- `switcher → projection`（`ui_agent_kind`/`status_color`）
- `mod → projection`（`switcher_key` re-export）
- `tests → mod`（`SessionSurfaces`/`StoreRuntime`）
- `terminal_pane → mod`（`switcher_key` 经 re-export，路径不变）

依赖方向单向：投影函数在最底层，渲染模块读投影与 facade 字段。

## 任务清单

| Task | 描述 | 验收 | 关联验证 Case |
|------|------|------|---------------|
| T1 | 抽取 projection.rs（投影自由函数 + switcher_key re-export） | cargo check 全绿 | C1 |
| T2 | 抽取 switcher.rs（render_switcher） | cargo check 全绿 | C2 |
| T3 | 抽取 overview.rs（overview chrome 渲染） | cargo check 全绿 | C3 |
| T4 | 抽取 overview_card.rs（卡片/行/批量关闭渲染） | cargo check 全绿 | C4 |
| T5 | 抽取 tests.rs + mod.rs 收尾为 facade | 单文件 < 800 | C5 |
| T6 | 全量验证 + code review + release readiness | fmt/check/clippy/test 全绿 | C6/C7 |

## 验证口径

- `cargo fmt --check`
- `cargo check -p homie-app`
- `cargo clippy -p homie-app --all-targets`
- `cargo test -p homie-app`（session_surfaces 相关测试原样通过，行为等价）
- `mod.rs` / `overview.rs` / `overview_card.rs` / `switcher.rs` / `projection.rs` / `tests.rs` 均 < 800 行
