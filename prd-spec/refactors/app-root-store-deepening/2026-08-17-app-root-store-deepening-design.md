# App Root/Store 下沉与投影单点化设计文档

## 1. 概述

### 1.1 问题/动机

`homie/crates/homie-app/src/root.rs`（2,130 行）仍不是足够窄的 shell 组合根：同时拥有 child entity
组合、全局键盘路由、panel seam、窗口持久化、surface 协调，违反 `specs/gpui-shell.md`「RootView
不堆积业务逻辑」合同。同时 `store/mod.rs`（2,434 行）与 engine `registry` 存在 session/project
概念双重投影，造成知识重复。2026-08 审计 finding **F8 + F9（Warning）**。

### 1.2 目标

把 root 的 shortcut 策略、seam 几何、窗口 placement debounce 下沉为纯逻辑；store 的 session/project
投影收敛为单点，消除与 engine registry 的重复投影。保持视觉/行为不变。

### 1.3 非目标

不重做 store 架构；不改变 `specs/gpui-shell.md` 合同本身；不迁移到全局单一 store。

### 1.4 基线

- branch `main`，commit `e4c7454`；目标 `root.rs`（2,130）、`store/mod.rs`（2,434）。
- 依赖至少一个 P1 container 拆分完成后启动（复用 seam 经验）。

## 2. 方案设计

### 2.1 拆分原则

- root：抽 shortcut 策略、seam 几何、窗口 placement debounce 纯函数到 `root/` 子模块。
- store：session/project 投影收敛为单一 source of truth，指向 engine 投影或 store 投影其一，消除重复。

### 2.2 目标模块拓扑

```text
homie/crates/homie-app/src/
├── root.rs                 # 组合根（< 800 行）
├── root/
│   ├── shortcuts.rs
│   ├── seams.rs
│   └── placement.rs
└── store/
    ├── mod.rs              # 投影单点化（< 800 行）
    └── projection.rs       # session/project 单一投影
```

### 2.3 实施顺序

S1 `root/` 纯逻辑下沉；S2 `store/projection.rs` 投影单点化；每步 `cargo test -p homie-app` 全绿。

## 3. 测试与验收

- 验收：`root.rs`/`store/mod.rs` < 800 行；session/project 投影单一来源；测试全绿；视觉/行为不变；
  同步评估 `specs/gpui-shell.md` 是否更新。
- 证据目录：`docs/verification/app-root-store-deepening/`

## 4. Beads 追踪

- change_id `app-root-store-deepening`；parent `homie-ubu`；child `homie-ubu.7`；P1。
