# OpenSpec Plan — app-menu-bar-mod-split

## 目标

将 `homie/crates/homie-app/src/macos/menu_bar.rs`（962 行）机械拆分为目录化聚焦子模块：
`mod.rs`（NativeMenuBar 结构 + 生命周期）、`render.rs`（body 行渲染）、`model.rs`（数据/样式模型）、
`target.rs`（objc2 事件目标类）。公共 API 与运行时行为零变更，引用方零改动，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（父）→ `render.rs`/`model.rs`/`target.rs`（子）；`render.rs` → `model.rs`
  （复用样式/模型函数与常量）；`target.rs` 无向下依赖（仅 objc2 + store）。
- 跨子模块互调采用 `pub(super)`：`MenuBodyRow`/`menu_rows`/`active_session_count`/
  `panel_fingerprint`/`symbol_image`（model），`add_project_header`/`add_session_row`/
  `add_more_row`/`add_empty_state`（render），`MenuBarTarget`（target），以及常量 `POPUP_WIDTH`/
  `ROW_HEIGHT`/`BODY_PADDING`。全部仅对父模块 `menu_bar` 可见，无 `pub` 泄漏。
- `MenuBarTarget` 的 `define_class!` 宏定义在 `target.rs` 子模块，父模块 `mod.rs` 通过
  `use target::MenuBarTarget;` 引用，`NativeMenuBar._target` 字段类型改为
  `Retained<target::MenuBarTarget>`。objc2 类名与选择子不变，运行时注册行为不变。

## 交付切片

- T1：抽取数据/样式模型 → `model.rs`。
- T2：抽取 body 行渲染 → `render.rs`。
- T3：抽取 objc2 事件目标类 → `target.rs`。
- T4：`mod.rs` facade 收尾（结构 + 生命周期 + 模块声明）。
- T5：全量验证 + code review + release readiness。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/app-menu-bar-mod-split/`。
