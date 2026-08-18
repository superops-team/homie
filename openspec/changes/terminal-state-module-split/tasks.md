# OpenSpec Tasks — terminal-state-module-split

## S1 抽取 mirror.rs（GridMirror + MirrorError + validate_grid）

- [x] `mirror.rs`：移动 `GridMirror` 结构体 + `impl GridMirror` + `MirrorError` + `validate_grid`，公共类型保持 `pub`，`validate_grid` 保持私有。
- [x] `lib.rs`：`mod mirror;` + `pub use mirror::{GridMirror, MirrorError};`。
- 验收：`cargo check -p homie-terminal-state` 全绿。关联 C1。

## S2 抽取 wire.rs（单元映射辅助）

- [x] `wire.rs`：移动 `wire_cell`/`wire_color`/`wire_style`/`sgr`/`append_color`/`find`/`emulator_cell`/`emulator_color`；跨模块用的 `wire_cell`/`sgr`/`emulator_cell`/`find` 改 `pub(crate)`，其余保持私有。
- [x] `lib.rs`：`mod wire;`。
- 验收：`cargo check -p homie-terminal-state` 全绿。关联 C2。

## S3 抽取 screen.rs（HeadlessScreen 状态机 + 投影）

- [x] `screen.rs`：移动 `HeadlessScreen` + `ScreenSnapshot` + `Geometry` + `Collector` + `HISTORY_CELL_BUDGET_BYTES`/`history_line_limit`/`EVENT_QUEUE_CAPACITY`，并通过 `use super::wire::{...}` 引入 wire 辅助。
- [x] `lib.rs`：`mod screen;` + `pub use screen::{HeadlessScreen, ScreenSnapshot};`。
- 验收：`cargo check -p homie-terminal-state` 全绿。关联 C3。

## S4 抽取 tests.rs + lib.rs re-export 收尾

- [x] `tests.rs`：移动旧内联 `#[cfg(test)] mod tests`（14 个测试），`use super::*;`。
- [x] `lib.rs`：删除内联 `mod tests`，加 `#[cfg(test)] mod tests;`。
- [x] 全量验证：`cargo check --workspace` + `cargo fmt --all --check` + `cargo test -p homie-terminal-state`。
- 验收：全部通过。关联 C4。
