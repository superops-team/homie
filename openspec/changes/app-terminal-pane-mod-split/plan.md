# OpenSpec Plan — app-terminal-pane-mod-split

## 目标

将 `terminal_pane/mod.rs`（1,486 行）机械拆分为聚焦子模块：`events.rs`、`input.rs`、`find.rs`、
`geometry.rs`、`clipboard.rs`、`scroll.rs`，`mod.rs` 保留 facade。公共 API 与行为零变更，
引用方零改动，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（父）→ 各子模块（子）。子模块间通过 `use super::*` 访问父模块私有
  字段/类型/方法；跨模块调用的方法升 `pub(super)`。
- 输入/查找/缩放/滚动/剪贴板/几何方法下沉对应子模块；事件分发/网格重排下沉 `events.rs`；
  生命周期/驻留/视图接口/状态字形/选择保留 `mod.rs`。
- 测试已独立为 `tests.rs`，无需改动。

## 交付切片

- T1：抽取事件分发 + 网格重排 → `events.rs`。
- T2：抽取键盘输入 → `input.rs`。
- T3：抽取查找 → `find.rs`。
- T4：抽取网格几何 + 缩放 → `geometry.rs`。
- T5：抽取剪贴板 → `clipboard.rs`。
- T6：抽取滚动回取 + 滚动 → `scroll.rs`。
- T7：全量验证 + code review + release readiness。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/app-terminal-pane-mod-split/`。
