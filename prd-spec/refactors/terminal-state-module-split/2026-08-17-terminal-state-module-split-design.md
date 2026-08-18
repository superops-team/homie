# Terminal State 单文件拆分设计文档

## 1. 概述

### 1.1 问题/动机

`homie/crates/homie-terminal-state/src/lib.rs` 约 1,168 行，整个 crate 是单文件，同时承载
grid mirror（接收端快照/增量权威）、headless screen（VT 模拟 + 进度扫描 + grid 投影）、wire 单元映射
（alacritty cell <-> wire cell/color/style 互转）与内联测试。2026-08 审计 finding **F10（Suggestion）**。

### 1.2 目标

按职责拆成多模块（mirror / screen / wire），`lib.rs` 只做 re-export，保持 public API 与运行时行为不变。

### 1.3 非目标

不改变 crate 对外 API；不改变 grid/增量/恢复语义；不引入新依赖；不把仍绑定 `HeadlessScreen` 上下文的
方法强拆成纯函数。

### 1.4 基线

- branch `main`，commit `b96bbd1`；目标 `homie-terminal-state/src/lib.rs`（1,168 行）。

## 2. 方案设计

### 2.1 拆分原则

纯机械拆分：只移动代码、调整 `pub(crate)` 可见性、保留注释与实现；不重写逻辑。

- `mirror.rs`：`GridMirror`（接收端快照/增量权威）+ `MirrorError` + `validate_grid` 校验。
- `screen.rs`：`HeadlessScreen`（VT 模拟 + OSC 9;4 进度扫描 + grid 投影）+ `ScreenSnapshot` +
  `Geometry`/`Collector` + history 预算常量 + `find` 字节检索辅助。
- `wire.rs`：alacritty cell <-> wire cell/color/style 互转（`wire_cell`/`wire_color`/`wire_style`/
  `sgr`/`append_color`/`emulator_cell`/`emulator_color`）。
- `tests.rs`：旧 `#[cfg(test)] mod tests` 内联测试整体迁出。
- `lib.rs`：模块声明 + `pub use` re-export（`GridMirror`/`MirrorError`/`HeadlessScreen`/`ScreenSnapshot`）。

### 2.2 目标模块拓扑

```text
homie/crates/homie-terminal-state/src/
├── lib.rs      # mod 声明 + pub use re-export（< 50 行）
├── mirror.rs   # GridMirror + MirrorError + validate_grid
├── screen.rs   # HeadlessScreen + ScreenSnapshot + Geometry + Collector + history
├── wire.rs     # alacritty <-> wire 单元映射
└── tests.rs    # 旧内联测试迁出
```

依赖方向：`screen → wire`（`wire_cell`/`sgr`/`emulator_cell`/`find`）；`lib → {mirror, screen}`；
`mirror`/`wire` 无逆向依赖。

### 2.3 可见性规则

- 跨子模块访问的辅助函数（`wire_cell`/`sgr`/`emulator_cell`/`find`）统一 `pub(crate)`。
- 对外公开 API 保持不变：`GridMirror`/`MirrorError`/`HeadlessScreen`/`ScreenSnapshot`。
- 子模块内部辅助（`wire_color`/`wire_style`/`append_color`/`emulator_color`/`validate_grid`）保持私有。

### 2.4 实施顺序

- S1 抽出 `mirror.rs`（GridMirror + MirrorError + validate_grid）。
- S2 抽出 `wire.rs`（单元映射辅助）。
- S3 抽出 `screen.rs`（HeadlessScreen 状态机 + 投影）。
- S4 抽出 `tests.rs` + `lib.rs` re-export 收尾。
每步之后 `cargo check -p homie-terminal-state` 与 `cargo test -p homie-terminal-state` 全绿。

## 3. 测试与验收

- 验收：
  - `cargo check --workspace` 无 error/warning。
  - `cargo fmt --all --check` 通过。
  - `cargo test -p homie-terminal-state` 全绿（14 passed / 0 failed）。
  - 旧内联测试原样迁至 `tests.rs` 且全部通过。
  - `lib.rs` 仅 re-export（< 50 行），mirror/screen/wire 各职责落入对应子模块。
  - 对外 public API 与行为完全不变。
- 证据目录：`docs/verification/terminal-state-module-split/`

## 4. Beads 追踪

- change_id `terminal-state-module-split`；parent `homie-ubu`；child `homie-ubu.8`；P2。
