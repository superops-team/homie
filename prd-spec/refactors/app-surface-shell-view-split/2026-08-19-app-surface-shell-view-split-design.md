# surface_shell/view.rs 进一步拆分设计文档

## 1. 背景

2026-08 架构审计（`architecture-audit-governance-2026-08`）的 F4 已把 `app/surface_shell.rs`
（4,362 行）拆为 `surface_shell/{mod,host_editor,host_init,projection,tests,view}.rs`，但
`view.rs` 仍遗留 **2,475 行**，是当前 homie-app 中最大的单文件 God Module。

`view.rs` 内部混装三类职责：

1. 主界面 utility 面板渲染（history / worktrees，`render_history`/`render_worktrees`）；
2. 设置面板渲染（`render_settings` 及 general/terminal/resource/remote/host-editor 子面板，
   约 1,500 行，占 view.rs 主体）；
3. 一堆自由 UI 原语/小部件（`surface_button`/`settings_*_button`/`toggle_row`/`setting_row`/
   `settings_dropdown`/`theme_preview`/`chip`/`colored_badge`/`empty_label` 等，约 530 行）。

设置面板内部又可分为两块内聚子域：

- **应用自身设置**（general / default_agent / update / terminal / resource /
  terminal_theme / hibernate / memory，约 760 行）；
- **远端主机管理**（remote / remote_hosts_section / host_initialization_card /
  host_editor_panel / host_text_field，约 750 行）。

三者边界清晰、内聚性可切，是本次审计后最明确的下一个优化切片。

## 2. 目标

- 把 `surface_shell/view.rs`（2,475 行）拆为聚焦子模块，**单文件 < 800 行**。
- 设置面板渲染（应用设置 / 远端主机两块）与通用 UI 原语各自下沉到独立子模块，
  `view.rs` 只保留 facade 编排。
- 公共 API 与运行时渲染行为完全不变。

## 3. 非目标

- 不重设计任何 UI/交互，不改 GPUI 渲染路径语义。
- 不改 `UtilitySurfaces` 对外结构体字段与 `Render` 契约。
- 不合并/删除任何既有 widget；纯职责搬迁。
- 不触及 `specs/gpui-shell.md` 的 RootView/store 合同（本次不涉及）。

## 4. 需求

### FR-1: 应用自身设置下沉

`render_settings` 及应用自身设置子面板（`general_settings` / `default_agent_dropdown` /
`update_settings` / `terminal_settings` / `resource_settings` / `terminal_theme_dropdown` /
`hibernate_dropdown` / `memory_dropdown`）下沉到 `surface_shell/settings_view.rs`，
保持 `impl UtilitySurfaces` 方法原位迁移。

### FR-2: 远端主机管理下沉

`remote_settings` / `remote_hosts_section` / `host_initialization_card` /
`host_editor_panel` / `host_text_field` 下沉到 `surface_shell/hosts_view.rs`，
保持 `impl UtilitySurfaces` 方法原位迁移。

### FR-3: 通用 UI 原语下沉

自由 UI 原语（`surface_button`/`settings_primary_button`/`settings_danger_button`/
`danger_button`/`toggle_row`/`setting_section`/`setting_row`/`setting_text_stack`/
`wrappable_setting_copy`/`settings_note`/`settings_page`/`setting_divider`/
`settings_select_button`/`settings_dropdown`/`settings_choice_row`/`theme_preview`/`chip`/
`colored_badge`/`empty_label`/`host_field_value`/`text_offset_for_x`）下沉到
`surface_shell/widgets.rs`。

### FR-4: view.rs 收尾为 facade

`view.rs` 只保留 `render_history`/`render_worktrees` 与 `impl Render for UtilitySurfaces`
入口，行数 < 800。

### FR-5: 行为不变

拆分后 `cargo check -p homie-app`、`cargo test -p homie-app`、`cargo fmt --check` 全绿，
渲染行为与拆分前等价。

## 5. 涉及文件

- `homie/crates/homie-app/src/surface_shell/view.rs`（拆分源，收尾 facade）
- `homie/crates/homie-app/src/surface_shell/settings_view.rs`（新增，应用设置渲染）
- `homie/crates/homie-app/src/surface_shell/hosts_view.rs`（新增，远端主机管理渲染）
- `homie/crates/homie-app/src/surface_shell/widgets.rs`（新增，通用 UI 原语）
- `homie/crates/homie-app/src/surface_shell/mod.rs`（`mod` 声明）
- `homie/crates/homie-app/src/surface_shell/tests.rs`（`setting_row` 导入路径调整）

## 6. 验证计划

```bash
cargo fmt --check
cargo check -p homie-app
cargo test -p homie-app
```

人工验收：

1. 设置面板、utility 面板、history/worktrees 渲染正常。
2. 所有既有 surface_shell 测试通过。
3. `view.rs` / `settings_view.rs` / `hosts_view.rs` / `widgets.rs` 均 < 800 行。

## 7. Beads

- change_id: `app-surface-shell-view-split`
- 类型: task（机械拆分，行为不变）
- 优先级: P1（homie-app 组合根降熵）
- 上游: `architecture-audit-governance-2026-08`（F4 延续）
