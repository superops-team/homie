# session_surfaces.rs 模块拆分设计文档

## 1. 背景

2026-08 架构审计（`architecture-audit-governance-2026-08`）的模块降熵序列已先后拆分了
`surface_shell/view.rs`、`terminal_pane/mod.rs`、`sidebar`、`inspector` 等 God Module。
当前 homie-app 前几大单文件：

| 文件 | 行数 |
|------|------|
| `store/mod.rs` | 2,434 |
| `session_surfaces.rs` | 1,657 |
| `terminal_pane/mod.rs` | 1,486 |
| `git_review.rs` | 1,427 |
| `code_intelligence.rs` | 1,426 |

`session_surfaces.rs`（1,657 行）是 T13 导航界面（switcher + overview）的组合视图，内部混装：

1. `SessionSurfaces` 结构体 + 非渲染状态维护（`new`/`colors`/resident buffer 同步/`open_overview`）；
2. 事件路由（`handle_key_down`/`handle_key_up`/`handle_modifiers_changed`）；
3. switcher 渲染（`render_switcher`，约 198 行）；
4. overview 渲染（`render_overview` 及 mode/filter/empty/board/list/card/row/bulk-close 等子方法，
   约 860 行）；
5. 自由投影函数（`switcher_key`/`ui_agent_kind`/`ui_status_state`/`status_color`/`state_badge`/
   `state_badge_color`/`clamp_branch`，约 106 行）；
6. 内联测试（`mod tests`，约 178 行）。

相比 `store/mod.rs`（依赖密、风险高），`session_surfaces.rs` 是单文件、无子目录、渲染方法内聚
边界清晰，是最适合作为下一个机械拆分切片的候选。

## 2. 目标

- 把 `session_surfaces.rs`（1,657 行）拆为目录化聚焦子模块，**单文件 < 800 行**。
- overview 渲染、switcher 渲染、投影自由函数、测试各自下沉到独立子模块，
  `mod.rs` 只保留 facade（结构体 + 非渲染 impl + 事件路由 + `impl Render` + 共享渲染辅助）。
- 公共 API 与运行时渲染行为完全不变。

## 3. 非目标

- 不重设计任何 UI/交互，不改 GPUI 渲染路径语义。
- 不改 `SessionSurfaces` 对外结构体字段与 `Render` 契约。
- 不合并/删除任何既有方法；纯职责搬迁。
- 不触及任何 `specs/` 合同（本次不涉及长生命周期组件接口变更）。

## 4. 需求

### FR-1: overview 渲染下沉

`render_overview` / `mode_button` / `filter_chip` / `overview_empty_state` /
`overview_board` / `overview_list` 下沉到 `session_surfaces/overview.rs`。

### FR-2: overview 卡片/行渲染下沉

`overview_card` / `overview_list_row` / `bulk_close_bar` 下沉到
`session_surfaces/overview_card.rs`。

### FR-3: switcher 渲染下沉

`render_switcher` 下沉到 `session_surfaces/switcher.rs`。

### FR-4: 投影自由函数下沉

`switcher_key` / `ui_agent_kind` / `ui_status_state` / `status_color` / `state_badge` /
`state_badge_color` / `clamp_branch` 下沉到 `session_surfaces/projection.rs`；
`switcher_key` 保持 `pub(crate)` 并通过 `mod.rs` re-export，使
`crate::session_surfaces::switcher_key` 引用路径不变。

### FR-5: 测试下沉

内联 `mod tests` 下沉到 `session_surfaces/tests.rs`，`#[cfg(test)]` 保持。

### FR-6: mod.rs 收尾为 facade

`session_surfaces/mod.rs` 保留 `SessionSurfaces` 结构体 + 非渲染 impl（`new`/`colors`/
`set_resident_buffer`/`remove_resident_buffer`/`sync_resident_buffers`/`open_overview`）+
事件路由（三个 `handle_*`）+ `impl Render for SessionSurfaces` + 共享渲染辅助
（`render_grid_or_logo`/`status_glyph`），行数 < 800。

### FR-7: 行为不变

拆分后 `cargo check -p homie-app`、`cargo test -p homie-app`、`cargo fmt --check` 全绿，
渲染行为与拆分前等价。

## 5. 涉及文件

- `homie/crates/homie-app/src/session_surfaces.rs`（拆分源，转为目录）
- `homie/crates/homie-app/src/session_surfaces/mod.rs`（新增，facade）
- `homie/crates/homie-app/src/session_surfaces/overview.rs`（新增，overview chrome 渲染）
- `homie/crates/homie-app/src/session_surfaces/overview_card.rs`（新增，卡片/行/批量关闭渲染）
- `homie/crates/homie-app/src/session_surfaces/switcher.rs`（新增，switcher 渲染）
- `homie/crates/homie-app/src/session_surfaces/projection.rs`（新增，投影自由函数）
- `homie/crates/homie-app/src/session_surfaces/tests.rs`（新增，测试）

## 6. 验证计划

```bash
cargo fmt --check
cargo check -p homie-app
cargo clippy -p homie-app --all-targets
cargo test -p homie-app
```

人工验收：

1. switcher / overview 渲染与拆分前等价（既有 overview 滚动测试原样通过）。
2. 所有既有 session_surfaces 测试通过。
3. 每个新子模块与 `mod.rs` 均 < 800 行。

## 7. Beads

- change_id: `app-session-surfaces-split`
- 类型: task（机械拆分，行为不变）
- 优先级: P1（homie-app 组合根降熵）
- 上游: `architecture-audit-governance-2026-08`（模块降熵序列延续）
