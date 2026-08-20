# OpenSpec Tasks — app-code-viewer-mod-split

## T1 抽取词法高亮 highlight.rs

- [x] `highlight.rs`：下沉 `source_row`/`highlighted_source`/`lexical_highlights`/`is_ident` 与
  `RUST_KEYWORDS`/`SWIFT_KEYWORDS`/`PYTHON_KEYWORDS`/`JS_KEYWORDS`/`COMMON_KEYWORDS` 常量表。
- [x] `source_row` 升 `pub(super)`（render.rs 调用）；`lexical_highlights` 升 `pub(super)`
  （tests.rs 调用）；其余保持私有。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`mod highlight;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T2 抽取渲染方法 + Render impl render.rs

- [x] `render.rs`：下沉 `render_toolbar`/`render_source`/`render_picker`/`render_message`（作为
  `impl super::CodeViewer` 扩展块）与 `impl Render for super::CodeViewer`。
- [x] 头：`use super::*;` + `use super::highlight::source_row;`。
- [x] `mod.rs`：`mod render;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T3 测试原样下沉 tests.rs

- [x] `tests.rs`：既有测试原样下沉为 `#[cfg(test)]` 文件，剥离 `mod tests {}` 外壳。
- [x] 头：`use super::*;` + `use super::highlight::lexical_highlights;`。
- [x] `mod.rs`：`#[cfg(test)] mod tests;`。
- 验收：`cargo test -p homie-app --offline code_viewer` 3/3 通过。关联 C2/C4。

## T4 全量验证 + code review + release readiness

- [x] `cargo fmt --all --check`
- [x] `cargo check -p homie-app --offline`
- [x] `cargo clippy -p homie-app --all-targets --offline`
- [x] `cargo test -p homie-app --offline`
- [x] code review（拆分边界清晰、无行为变更、无可见性泄漏、引用方零改动）
- [x] release readiness 证据写入 `docs/verification/app-code-viewer-mod-split/`
- 验收：全部通过。关联 C3/C4/C6。
