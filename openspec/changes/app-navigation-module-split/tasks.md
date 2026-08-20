# OpenSpec Tasks — app-navigation-module-split

## T1 抽取渲染 render.rs

- [x] `navigation.rs` → `navigation/mod.rs`（目录化）。
- [x] `render.rs`：下沉 `render_overlay`/`render_search`/`render_command_palette`/
  `render_action_row`/`render_session_row`/`render_quick_open`/`render_quick_row`。
- [x] `render.rs`：下沉自由函数 `section_header`/`empty_label`/`palette_row`/`chip`/
  `attention_color`/`kind_label`/`relative_parent`。
- [x] `render_overlay`/`relative_parent` 升为 `pub(super)`；其余 `render_*` 保持私有。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`mod render;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C1/C2/C5。

## T2 抽取目录索引 index.rs

- [x] `index.rs`：下沉 `index_roots`/`load_cached_index`/`refresh_directory_index`/
  `snapshot_inputs`/`project_roots`。
- [x] `load_cached_index`/`refresh_directory_index` 升为 `pub(super)`；其余保持私有。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`mod index;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C5。

## T3 抽取测试 tests.rs + mod.rs 收尾 facade

- [x] `tests.rs`：移动 `#[cfg(test)] mod tests` 全部内容。
- [x] 头：`use super::*;` + `use super::render::relative_parent;`。
- [x] `mod.rs`：`#[cfg(test)] mod tests;`，删除内联 test 模块。
- [x] `mod.rs` 保留类型/常量 + 生命周期/排序/命令方法 + trait 实现 + 共享查询字段工具。
- 验收：每文件 < 800 行。关联 C3。

## T4 全量验证 + code review + release readiness

- [x] `cargo fmt --check`
- [x] `cargo check -p homie-app`
- [x] `cargo clippy -p homie-app --all-targets`
- [x] `cargo test -p homie-app`
- [x] code review（拆分边界清晰、无行为变更、无可见性泄漏、引用方零改动）
- [x] release readiness 证据写入 `docs/verification/app-navigation-module-split/`
- 验收：全部通过。关联 C4/C6。
