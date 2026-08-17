# App Inspector 模块拆分设计文档

## 1. 概述

### 1.1 问题/动机

`homie/crates/homie-app/src/inspector.rs` 约 4,692 行，是 `homie-app` 组合根最大的单文件，同时
承担 GPUI render 树、tab 状态机、PR/review/ask 工作流、artifact 投影、code/change 视图、store
effect dispatch 与测试 fixture。2026-08 审计 finding **F3（Critical）**：Cognitive Overload。

### 1.2 目标

把 inspector 拆成「纯逻辑 / render / effect dispatch」分层子模块，render 结构留在宿主文件，
纯状态机与投影函数下沉为可独立单测的模块。保持视觉与行为不变。

### 1.3 非目标

不重做 inspector 设计/视觉；不整体重写；不改变 GPUI component 层级；不迁移到全局 store。

### 1.4 基线

- branch `main`，commit `e4c7454`；目标 `inspector.rs`（4,692 行）。
- 复用 `gpui-large-module-test-boundaries` 的「纯逻辑优先、行为不变」原则。

## 2. 方案设计

### 2.1 拆分原则

先抽无 `Window`/`Context`/`Entity` 依赖的纯函数（tab 状态转移、review action 策略、artifact 分组、
error 压缩、文案映射），再抽 effect 组装；render 只负责调用与展示。

### 2.2 目标模块拓扑

```text
homie/crates/homie-app/src/inspector/
├── view.rs                 # GPUI render + click handler（< 800 行）
├── state.rs                # tab 状态机、review/ask 工作流状态
├── projection.rs           # artifact grouping、code/change 投影、error compaction
├── policy.rs               # review action 策略、快捷键决策
└── tests.rs                # 纯逻辑单测
```

### 2.3 实施顺序

S1 抽 `state.rs`；S2 抽 `projection.rs`；S3 抽 `policy.rs`；S4 收尾 render 留在 `view.rs`。
每步 `cargo test -p homie-app` 全绿。

## 3. 测试与验收

- 验收：`inspector/view.rs` < 800 行；子模块无 GPUI 依赖可独立单测；`cargo test -p homie-app` 全绿；
  视觉/行为不变。
- 证据目录：`docs/verification/app-inspector-module-split/`

## 4. Beads 追踪

- change_id `app-inspector-module-split`；parent `homie-ubu`；child `homie-ubu.3`；P1。
