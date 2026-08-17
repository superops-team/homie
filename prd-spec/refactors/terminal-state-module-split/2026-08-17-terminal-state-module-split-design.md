# Terminal State 单文件拆分设计文档

## 1. 概述

### 1.1 问题/动机

`homie/crates/homie-terminal-state/src/lib.rs` 约 1,168 行，整个 crate 是单文件，承载 terminal 状态机、
grid 状态、增量更新与测试。2026-08 审计 finding **F10（Suggestion）**。

### 1.2 目标

按职责拆成多模块（grid state / delta / 状态机），保持 public API 与行为不变。

### 1.3 非目标

不改变 crate 对外 API；不改变 grid/增量语义；不引入新依赖。

### 1.4 基线

- branch `main`，commit `e4c7454`；目标 `homie-terminal-state/src/lib.rs`（1,168 行）。

## 2. 方案设计

### 2.1 拆分原则

按状态机 / grid / delta / projection 切分，`lib.rs` 只做 re-export。

### 2.2 目标模块拓扑

```text
homie/crates/homie-terminal-state/src/
├── lib.rs                 # re-export（< 200 行）
├── grid.rs
├── delta.rs
├── state.rs
└── tests.rs
```

### 2.3 实施顺序

S1 `grid.rs`；S2 `delta.rs`；S3 `state.rs`；每步 `cargo test -p homie-terminal-state` 全绿。

## 3. 测试与验收

- 验收：`lib.rs` 仅 re-export；各模块内聚；测试全绿；对外 API/行为不变。
- 证据目录：`docs/verification/terminal-state-module-split/`

## 4. Beads 追踪

- change_id `terminal-state-module-split`；parent `homie-ubu`；child `homie-ubu.8`；P2。
