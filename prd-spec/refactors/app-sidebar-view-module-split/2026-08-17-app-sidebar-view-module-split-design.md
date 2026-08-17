# App Sidebar View 模块拆分设计文档

## 1. 概述

### 1.1 问题/动机

`homie/crates/homie-app/src/sidebar/view.rs` 约 4,310 行，承担 sidebar render、popover、directory
picker、drag/drop、排序、快捷键、操作 dispatch。2026-08 审计 finding **F5（Critical）**。

### 1.2 目标

抽 new-agent picker 选择逻辑、drag reorder、shortcut rank、popover state 等纯逻辑为可独立单测模块，
render 留在 `view.rs`。保持视觉/行为不变。

### 1.3 非目标

不重做 sidebar 设计/系统；不改变 GPUI 层级。

### 1.4 基线

- branch `main`，commit `e4c7454`；目标 `sidebar/view.rs`（4,310 行）。
- 复用 `gpui-large-module-test-boundaries` 已抽出的 `sidebar/picker_logic.rs`。

## 2. 方案设计

### 2.1 拆分原则

延续 `picker_logic.rs` 方向，抽 drag reorder、shortcut rank、popover state 等纯函数。

### 2.2 目标模块拓扑

```text
homie/crates/homie-app/src/sidebar/
├── view.rs                 # GPUI render（< 800 行）
├── picker_logic.rs         # 已有
├── reorder.rs              # drag reorder
├── shortcuts.rs            # shortcut rank
├── popover.rs              # popover state
└── tests.rs
```

### 2.3 实施顺序

S1 `reorder.rs`；S2 `shortcuts.rs`；S3 `popover.rs`；每步 `cargo test -p homie-app` 全绿。

## 3. 测试与验收

- 验收：`sidebar/view.rs` < 800 行；子模块无 GPUI 依赖；测试全绿；视觉/行为不变。
- 证据目录：`docs/verification/app-sidebar-view-module-split/`

## 4. Beads 追踪

- change_id `app-sidebar-view-module-split`；parent `homie-ubu`；child `homie-ubu.6`；P1。
