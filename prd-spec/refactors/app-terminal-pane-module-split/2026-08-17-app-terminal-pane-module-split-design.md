# App TerminalPane 模块拆分设计文档

## 1. 概述

### 1.1 问题/动机

`homie/crates/homie-app/src/terminal_pane.rs` 约 3,495 行，同时承担 terminal 渲染、resize、
input、clipboard、attachment、status chips 与测试。2026-08 审计 finding **F6（Critical）**。

### 1.2 目标

抽纯逻辑子域（key mapping、resize debounce、clipboard staging、status chip 投影、attachment 策略）
为可独立单测模块，render 留在宿主文件。保持视觉/行为不变。

### 1.3 非目标

不重做 terminal 渲染/设计；不改变 GPUI 层级；不迁移全局 store。

### 1.4 基线

- branch `main`，commit `e4c7454`；目标 `terminal_pane.rs`（3,495 行）。

## 2. 方案设计

### 2.1 拆分原则

先抽无 GPUI 依赖的纯函数（key mapping、resize debounce 阈值、clipboard staging、status chip 投影）。

### 2.2 目标模块拓扑

```text
homie/crates/homie-app/src/terminal_pane/
├── view.rs                 # GPUI render + 事件（< 800 行）
├── keymap.rs               # key 映射
├── resize.rs               # resize debounce
├── clipboard.rs            # clipboard staging
├── chips.rs                # status chip 投影
└── tests.rs
```

### 2.3 实施顺序

S1 `keymap.rs`；S2 `resize.rs`；S3 `clipboard.rs`；S4 `chips.rs`；每步 `cargo test -p homie-app` 全绿。

## 3. 测试与验收

- 验收：`terminal_pane/view.rs` < 800 行；子模块无 GPUI 依赖；测试全绿；视觉/行为不变。
- 证据目录：`docs/verification/app-terminal-pane-module-split/`

## 4. Beads 追踪

- change_id `app-terminal-pane-module-split`；parent `homie-ubu`；child `homie-ubu.4`；P1。
