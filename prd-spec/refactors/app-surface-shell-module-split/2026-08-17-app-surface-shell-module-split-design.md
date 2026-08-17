# App SurfaceShell 模块拆分设计文档

## 1. 概述

### 1.1 问题/动机

`homie/crates/homie-app/src/surface_shell.rs` 约 4,362 行，承担 utility surfaces 与 session surface
组合、history/worktrees/settings 子域、view-state 映射。2026-08 审计 finding **F4（Critical）**。

### 1.2 目标

把 history/worktrees/settings 等子域的 view-state 映射抽为纯逻辑模块，render 留在宿主文件。
保持视觉/行为不变。

### 1.3 非目标

不重做 surface shell 组合/设计；不改变 GPUI 层级。

### 1.4 基线

- branch `main`，commit `e4c7454`；目标 `surface_shell.rs`（4,362 行）。

## 2. 方案设计

### 2.1 拆分原则

先抽无 GPUI 依赖的 view-state 映射纯函数（history 列表映射、worktree 投影、settings 投影）。

### 2.2 目标模块拓扑

```text
homie/crates/homie-app/src/surface_shell/
├── view.rs                 # GPUI render（< 800 行）
├── history.rs              # history 子域映射
├── worktrees.rs            # worktree 子域映射
├── settings.rs             # settings 子域映射
└── tests.rs
```

### 2.3 实施顺序

S1 `history.rs`；S2 `worktrees.rs`；S3 `settings.rs`；每步 `cargo test -p homie-app` 全绿。

## 3. 测试与验收

- 验收：`surface_shell/view.rs` < 800 行；子模块无 GPUI 依赖；测试全绿；视觉/行为不变。
- 证据目录：`docs/verification/app-surface-shell-module-split/`

## 4. Beads 追踪

- change_id `app-surface-shell-module-split`；parent `homie-ubu`；child `homie-ubu.5`；P1。
