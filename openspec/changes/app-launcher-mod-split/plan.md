# OpenSpec Plan — app-launcher-mod-split

## 目标

将 `homie/crates/homie-app/src/launcher.rs`（1,046 行）机械拆分为目录化聚焦子模块：
`render.rs`、`tests.rs`，`mod.rs` 保留 facade（布局常量 + 状态模型 + 生命周期/事件/提交 +
Focusable + 纯函数辅助）。公共 API 与运行时行为零变更，引用方零改动，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（父）→ `render.rs`（子）；`tests.rs` → `mod.rs`（`title_case_id`）。
- 渲染方法 + `Render` impl 下沉 `render.rs`（`impl super::LauncherOverlay` + `impl Render for
  super::LauncherOverlay`），渲染职责内聚；`render.rs` 通过 `use super::*` 访问父模块私有字段/
  方法与 free function，零 `pub(super)` 提升。
- 既有测试原样下沉 `tests.rs`。

## 交付切片

- T1：抽取渲染方法 + Render impl → `render.rs`。
- T2：测试原样下沉 → `tests.rs`。
- T3：全量验证 + code review + release readiness。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/app-launcher-mod-split/`。
