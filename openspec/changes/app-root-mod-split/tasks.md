# OpenSpec Tasks — app-root-mod-split

## T1 方法边界扫描

- [x] 以缩进 + 闭合括号定位 25 个方法边界，跳过 doc 注释、多行签名、嵌套闭包。
- 验收：25 个方法全部解析，无缺失/重复。关联 C2/C5。

## T2 生成子模块

- [x] `new.rs`（1）、`input.rs`（5）、`sessions.rs`（5）、`layout.rs`（11）、`inspector.rs`（3）。
- 验收：方法体逐字迁移，24 个私有方法提升为 `pub(crate)`，`new` 保持 `pub(crate)`。关联 C1/C5。

## T3 重建 mod.rs + 编译验证

- [x] 移除旧 `impl RootView` 块，新增子模块声明，保留结构体 + trait 实现。
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
- [x] release readiness 证据写入 `docs/verification/app-root-mod-split/`。
- 验收：通过。关联 C6。
