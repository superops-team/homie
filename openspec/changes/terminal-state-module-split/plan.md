# OpenSpec Plan — terminal-state-module-split

## 概述

将 `homie/crates/homie-terminal-state/src/lib.rs`（约 1,168 行）机械拆分为 `mirror` / `screen` / `wire` 三个聚焦子模块，`lib.rs` 只做 re-export。公共 API 与运行时行为完全不变。

## 模块划分与依赖

```text
homie-terminal-state/src/
├── lib.rs      mod 声明 + pub use re-export（GridMirror/MirrorError/HeadlessScreen/ScreenSnapshot）
├── mirror.rs   GridMirror + MirrorError + validate_grid
├── screen.rs   HeadlessScreen + ScreenSnapshot + Geometry + Collector + history
├── wire.rs     alacritty <-> wire 单元映射（wire_cell/sgr/emulator_cell/find 等）
└── tests.rs    旧内联测试迁出（use super::*）
```

依赖方向：`screen → wire`（`wire_cell`/`sgr`/`emulator_cell`/`find`）；`lib → {mirror, screen}`；`mirror`/`wire` 无逆向依赖。

## 任务清单

| Task | 描述 | 验收 | 关联验证 Case |
|------|------|------|---------------|
| S1 | 抽取 `mirror.rs`（GridMirror + MirrorError + validate_grid） | cargo check 全绿 | C1 |
| S2 | 抽取 `wire.rs`（单元映射辅助） | cargo check 全绿 | C2 |
| S3 | 抽取 `screen.rs`（HeadlessScreen 状态机 + 投影） | cargo check 全绿 | C3 |
| S4 | 抽取 `tests.rs` + `lib.rs` re-export 收尾 | cargo test 全绿 + fmt/check | C4 |

## 验证口径

- `cargo check --workspace`（0 error / 0 warning）
- `cargo fmt --all --check`
- `cargo test -p homie-terminal-state`（14 passed / 0 failed）
- 旧内联测试原样迁移至 `tests.rs` 且全部通过
