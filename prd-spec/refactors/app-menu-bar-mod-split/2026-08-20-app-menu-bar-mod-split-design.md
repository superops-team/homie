# PRD — app-menu-bar-mod-split

## 背景

`homie/crates/homie-app/src/macos/menu_bar.rs`（962 行）是 macOS 状态栏弹出面板的 God Module：
一个文件同时承载状态栏原生控件（`NativeMenuBar` 结构 + `new`/`update`/`set_attention`/
`update_activity`/`rebuild_body` 生命周期）、body 行渲染逻辑（`add_project_header`/
`add_session_row`/`add_more_row`/`add_empty_state`）、纯数据/样式模型（`MenuBodyRow`/
`panel_fingerprint`/`menu_rows`/`active_session_count`/`TrailingStatus`/`trailing_status`/
`agent_symbol`/`session_color`/`display_title`/`FontStyle`/`system_font`/`label`/`symbol_image`/
`symbol_view`/`separator`/`style_footer_button`/`rect`）、以及 objc2 事件目标类（`MenuBarTargetIvars`
+ `define_class! MenuBarTarget` + `impl MenuBarTarget`），单文件超过 800 行阈值，阅读与变更成本高，
违背仓库「组件模块化、关注点清晰」原则。

## 目标

将 `menu_bar.rs` 机械拆分为目录化聚焦子模块，公共 API 与运行时行为零变更，引用方零改动，
单文件 < 800 行。

## 非目标

- 不改变任何状态栏渲染、面板布局、fingerprint 计算、事件响应行为。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动 `NativeMenuBar::new`/`update` 公共签名与 `NativeMenuBar` 结构字段语义。
- 不改动 `MenuBarTarget` 的 objc2 类名、方法选择子（`toggleHomieMenu:`/`openHomie:`/
  `selectSession:`/`quitHomie:`）与事件行为。

## 用户场景

1. 开发者定位「状态栏控件生命周期与面板更新」时，直接进入 `mod.rs`。
2. 开发者定位「body 行渲染（项目头/会话行/更多行/空状态）」时，聚焦在 `render.rs`。
3. 开发者定位「数据模型与视觉样式（指纹、菜单行、字体、符号、颜色）」时，聚焦在 `model.rs`。
4. 开发者定位「objc2 事件目标类」时，聚焦在 `target.rs`。

## 模块划分方案

```text
menu_bar/
├── mod.rs     facade：doc + imports + 常量 + NativeMenuBar 结构 + impl（new/update/
│              set_attention/update_activity/rebuild_body）+ 模块声明
├── render.rs  body 渲染：add_project_header/add_session_row/add_more_row/add_empty_state
│              （作为 impl super::NativeMenuBar 扩展块）
├── model.rs   数据/样式模型：MenuBodyRow/panel_fingerprint/menu_rows/active_session_count/
│              TrailingStatus/trailing_status/agent_symbol/session_color/display_title/
│              FontStyle/system_font/label/symbol_image/symbol_view/separator/
│              style_footer_button/rect
└── target.rs  objc2 事件目标：MenuBarTargetIvars + define_class! MenuBarTarget + impl MenuBarTarget
```

职责边界：
- `mod.rs` 保留：`NativeMenuBar` 结构 + `new`/`update`（pub）+ `set_attention`/`update_activity`/
  `rebuild_body`（私有生命周期方法）+ 常量 `POPUP_WIDTH`/`HEADER_HEIGHT`/`FOOTER_HEIGHT`/
  `ROW_HEIGHT`/`BODY_PADDING`/`EMPTY_BODY_HEIGHT`/`MAX_BODY_ROWS`。
- `render.rs` 下沉：`add_project_header`/`add_session_row`/`add_more_row`/`add_empty_state`
  （`impl super::NativeMenuBar` 扩展块，方法 `pub(super)` 供 `rebuild_body` 调用）。
- `model.rs` 下沉：`MenuBodyRow`/`panel_fingerprint`/`menu_rows`/`active_session_count`/
  `TrailingStatus`/`trailing_status`/`agent_symbol`/`session_color`/`display_title`/`FontStyle`/
  `system_font`/`label`/`symbol_image`/`symbol_view`/`separator`/`style_footer_button`/`rect`。
- `target.rs` 下沉：`MenuBarTargetIvars` + `define_class! MenuBarTarget` + `impl MenuBarTarget`
  （`new`/`set_session_ids`/`show_main_window` 标记 `pub(super)`）。

## 可见性设计

- 公共 API（`NativeMenuBar::new`/`update`）路径不变，仍通过 `crate::macos::menu_bar::NativeMenuBar`
  可达（`root/mod.rs` 两处引用零改动）。
- 跨子模块互调采用 `pub(super)`（仅对父模块 `menu_bar` 可见，不泄漏到 crate 外）：
  - `mod.rs` → `model.rs`：`MenuBodyRow`/`menu_rows`/`active_session_count`/`panel_fingerprint`/
    `symbol_image`。
  - `mod.rs` → `target.rs`：`MenuBarTarget`。
  - `mod.rs`（`rebuild_body`）→ `render.rs`：`add_project_header`/`add_session_row`/`add_more_row`/
    `add_empty_state`。
  - `render.rs` → `model.rs`：`symbol_view`/`label`/`rect`/`system_font`/`symbol_image`/
    `session_color`/`agent_symbol`/`trailing_status`/`display_title`/`FontStyle` + 常量
    `POPUP_WIDTH`/`ROW_HEIGHT`/`BODY_PADDING`。
- `MenuBarTarget` 定义在 `target.rs` 子模块，父模块通过 `use target::MenuBarTarget;` 引用；
  `NativeMenuBar` 字段 `_target: Retained<target::MenuBarTarget>`。
- 无 `pub` 可见性泄漏到 crate 外，所有跨模块符号均为 `pub(super)` 或保持私有。

## 影响面

- 引用方：仅 `root/mod.rs` 依赖 `crate::macos::menu_bar::NativeMenuBar`，零改动。
- `MenuBarTarget`/`set_session_ids`/`show_main_window` 仅供 `menu_bar` 内部使用，无外部引用。
- 无持久化/schema/协议变更；objc2 类名与选择子不变。

## 测试计划

- `cargo check -p homie-app --offline` 全绿。
- `cargo fmt --all --check` 通过。
- `cargo clippy -p homie-app --all-targets --offline` 0 警告。
- `cargo build -p homie-app --offline` 通过（objc2 `define_class!` 跨模块引用编译验证）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件删除。
- C2：公共 API 不变，引用方零改动（`git status` 确认仅改 `macos/menu_bar/` 目录）。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy 全绿。
- C5：无 `pub` 可见性泄漏到 crate 外（跨模块符号均为 `pub(super)` 或私有）。
- C6：release readiness 证据写入 `docs/verification/app-menu-bar-mod-split/`。

## Beads

- `homie-v0t`
