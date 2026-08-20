# OpenSpec Tasks — app-sidebar-popover-split

## T1 编写括号扫描器解析边界

- [x] Rust 括号扫描器：跳过字符串/字符/注释/raw string，逐字解析 10 个方法边界。
- 验收：10 个方法全部解析，无缺失/重复。关联 C2/C5。

## T2 生成 popover/ 目录

- [x] `mod.rs`：`use super::*;` + 模块声明。
- [x] `shell.rs`（4）、`new_agent.rs`（1）、`directory_picker.rs`（1）、`update_menu.rs`（1）、
  `account.rs`（1）、`actions.rs`（2）。
- 验收：方法体逐字迁移，`pub(crate)` 保留。关联 C1/C5。

## T3 删除旧文件 + 编译验证

- [x] 删除 `sidebar/popover.rs`。
- [x] `cargo check -p homie-app --offline` 全绿。
- 验收：通过。关联 C2/C4。

## T4 全量验证

- [x] `cargo fmt --all --check`
- [x] `cargo check -p homie-app --offline`
- [x] `cargo clippy -p homie-app --all-targets --offline`
- [x] `cargo build -p homie-app --offline`
- [x] `cargo test -p homie-app --offline`
- 验收：全部通过（2 个 `daemon_launch` EPERM 属预期）。关联 C3/C4。

## T5 code review + release readiness

- [x] code review：拆分边界清晰、方法逐字迁移、无行为变更、`pub(crate)` 完整、单文件 < 800 行。
- [x] release readiness 证据写入 `docs/verification/app-sidebar-popover-split/`。
- 验收：通过。关联 C6。
