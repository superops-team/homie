# OpenSpec Tasks — app-launcher-mod-split

## T1 抽取渲染方法 + Render impl render.rs

- [x] `render.rs`：下沉 `render_harness_picker`/`render_project_picker`/`render_panel`/`floating`
  （作为 `impl super::LauncherOverlay` 扩展块）与 `impl Render for super::LauncherOverlay`。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`mod render;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T2 测试原样下沉 tests.rs

- [x] `tests.rs`：既有测试原样下沉为 `#[cfg(test)]` 文件，剥离 `mod tests {}` 外壳。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`#[cfg(test)] mod tests;`。
- 验收：`cargo test -p homie-app --offline launcher` 1/1 通过。关联 C2/C4。

## T3 全量验证 + code review + release readiness

- [x] `cargo fmt --all --check`
- [x] `cargo check -p homie-app --offline`
- [x] `cargo clippy -p homie-app --all-targets --offline`
- [x] `cargo test -p homie-app --offline`
- [x] code review（拆分边界清晰、无行为变更、无可见性泄漏、引用方零改动）
- [x] release readiness 证据写入 `docs/verification/app-launcher-mod-split/`
- 验收：全部通过。关联 C3/C4/C6。
