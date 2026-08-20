# OpenSpec Tasks — app-terminal-pane-view-split

## T1 方法边界扫描

- [x] 以缩进 + 闭合括号定位 8 个渲染方法 + 4 个自由辅助函数边界，跳过 doc 注释、多行签名、
  嵌套闭包。
- 验收：8 方法 + 4 函数全部解析，无缺失/重复。关联 C2/C5。

## T2 生成子模块

- [x] `buttons.rs`（4 函数）、`chrome.rs`（2 方法）、`grid.rs`（1 方法）、`find_bar.rs`（1 方法）、
  `status.rs`（4 方法）。
- 验收：方法/函数体逐字迁移，8 方法 + 4 函数统一提升为 `pub(crate)`。关联 C1/C5。

## T3 重建 mod.rs + 编译验证

- [x] 移除旧 `view.rs`，新增 `view/mod.rs`（保留 `impl Render` + 子模块声明）。
- [x] `cargo check -p homie-app --offline` 全绿。
- 验收：通过。关联 C2/C4。

## T4 全量验证

- [x] `cargo fmt --all --check`
- [x] `cargo check -p homie-app --offline`
- [x] `cargo clippy -p homie-app --all-targets --offline`
- [x] `cargo build -p homie-app --offline`
- [x] `cargo test -p homie-app --offline`（303 passed / 0 failed）
- 验收：全部通过。关联 C3/C4。

## T5 code review + release readiness

- [x] code review：拆分边界清晰、方法逐字迁移、无行为变更、可见性正确、单文件 < 800 行。
- [x] release readiness 证据写入 `docs/verification/app-terminal-pane-view-split/`。
- 验收：通过。关联 C6。
