# 发布就绪报告 — app-session-surfaces-split

## 变更概述

将 `homie/crates/homie-app/src/session_surfaces.rs`（1,657 行，T13 导航界面组合视图 God Module）
机械拆分为目录化聚焦子模块：overview chrome 渲染、overview 卡片/行渲染、switcher 渲染、
投影自由函数、测试各自下沉，`mod.rs` 收尾为 facade。公共 API 与运行时渲染行为完全不变，
单文件 < 800 行。

- change_id：`app-session-surfaces-split`
- Beads：`homie-4ix`
- 类型：task（机械拆分，行为不变）
- 上游：`architecture-audit-governance-2026-08`（模块降熵序列延续）

## 模块划分

```text
session_surfaces/
├── mod.rs           facade（SessionSurfaces 结构 + 非渲染 impl + 事件路由 + impl Render +
│                    render_grid_or_logo/status_glyph 共享辅助 + mod 声明 + switcher_key re-export），324 行
├── overview.rs      overview chrome 渲染（render_overview/mode_button/filter_chip/
│                    overview_empty_state/overview_board/overview_list），463 行
├── overview_card.rs overview 卡片/行渲染（overview_card/overview_list_row/bulk_close_bar），407 行
├── switcher.rs      switcher 渲染（render_switcher），204 行
├── projection.rs    投影自由函数（switcher_key/ui_agent_kind/ui_status_state/status_color/
│                    state_badge/state_badge_color/clamp_branch），109 行
└── tests.rs         既有测试（原样下沉，#[cfg(test)]），177 行
```

依赖方向：投影函数在最底层，渲染模块读投影与 facade 字段；`terminal_pane` 经
`crate::session_surfaces::switcher_key` re-export 引用，路径不变。

## 交付切片 T1–T6

| 切片 | 内容 | 状态 |
|------|------|------|
| T1 | 抽取投影自由函数 projection.rs + switcher_key re-export | 完成 |
| T2 | 抽取 switcher 渲染 switcher.rs | 完成 |
| T3 | 抽取 overview chrome 渲染 overview.rs | 完成 |
| T4 | 抽取 overview 卡片/行渲染 overview_card.rs | 完成 |
| T5 | 抽取测试 tests.rs + mod.rs 收尾 facade | 完成 |
| T6 | 全量验证 + code review + release readiness | 完成 |

## 验证证据

| 验证项 | 命令 | 结果 |
|--------|------|------|
| 编译检查 | `cargo check -p homie-app` | 通过，0 警告 |
| 格式检查 | `cargo fmt --all --check` | 通过 |
| 静态检查 | `cargo clippy -p homie-app --all-targets` | 通过，0 警告 |
| 全量测试 | `cargo test -p homie-app` | **303 passed / 0 failed / 1 ignored** |
| 单文件行数 | `wc -l` | mod 324 / overview 463 / overview_card 407 / switcher 204 / projection 109 / tests 177，均 < 800 |

- `session_surfaces` 相关测试（`overflowing_overview_lane_scrolls_without_reaching_the_background`）
  原样通过，渲染行为与拆分前等价。
- 公共 API 兼容性：`SessionSurfaces` 结构体字段、`impl Render for SessionSurfaces` 入口签名、
  `pub(crate)` 方法（`set_resident_buffer`/`remove_resident_buffer`/`sync_resident_buffers`/
  `open_overview`/`handle_*`）均未改动；`cargo check -p homie-app` 编译通过证明可达性不变。
- `switcher_key` 保持 `pub(crate)` 并通过 `mod.rs` 的 `pub(crate) use projection::switcher_key;`
  re-export，`terminal_pane/mod.rs` 的 `crate::session_surfaces::switcher_key` 引用路径不变。
- 可见性管控：仅跨模块调用方法（`render_switcher`/`render_overview`/`overview_card`/
  `overview_list_row`/`bulk_close_bar`）与投影函数升为 `pub(super)`，无 `pub` 泄漏到 crate 外。

## 已知限制与后续

- 无已知限制。
- 机械重构未改变任何运行时行为、公共 API 不变；纯职责搬迁，不做向后兼容。
- 后续候选（`store/mod.rs` 2,434 行）依赖密、风险更高，建议作为更谨慎的独立切片处理。

## 结论

所有验收标准（C1–C7）均已满足，验证证据齐备，可发布。
