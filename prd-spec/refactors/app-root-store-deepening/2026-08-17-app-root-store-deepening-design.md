# App Root/Store 下沉与投影单点化设计文档

## 1. 概述

### 1.1 问题/动机

`homie/crates/homie-app/src/root.rs`（2,130 行）仍不是足够窄的 shell 组合根：同时拥有 child entity
组合、全局键盘路由、panel seam、窗口持久化、auxiliary terminal 编排、surface 协调，违反
`specs/gpui-shell.md`「RootView 不堆积业务逻辑」合同。同时 `store/mod.rs`（2,434 行）与 engine
`registry` 存在 session/project 概念双重投影，造成知识重复。2026-08 审计 finding **F8 + F9（Warning）**。

### 1.2 目标

把 root 的 shortcut 策略、seam 几何、auxiliary terminal 编排、render 方法下沉为聚焦子模块；
store 的 session/project 投影收敛为单一 source of truth，消除与 engine registry 的重复投影。
保持视觉/行为不变。

### 1.3 非目标

不重做 store 架构；不改变 `specs/gpui-shell.md` 合同本身；不迁移到全局单一 store；
不把仍绑定 `impl RootView` 上下文的编排方法强行拆成纯函数。

### 1.4 基线

- branch `main`，commit `b96bbd1`（sidebar 拆分后 HEAD）；目标 `root.rs`（2,130）、`store/mod.rs`（2,434）。
- 复用 P1 container 拆分的 seam 经验（`surface_shell/`、`sidebar/`）。

## 2. 方案设计

### 2.1 拆分原则

纯机械拆分：只移动代码、调整 `pub(crate)` 可见性、保留注释与实现；不重写逻辑。

- `root.rs`：保留 `RootView` 结构体、`Focusable`、核心 `impl RootView` 编排方法与 `impl Render`。
- `root/shortcuts.rs`：`NewSessionShortcut` 枚举 + `new_session_shortcut` + `session_navigation_delta` 纯策略。
- `root/seams.rs`：`advance_seam` 纯函数 + 三个拖拽边缘 marker 结构体（`DraggedSidebarEdge`/`DraggedTerminalEdge`/`DraggedInspectorEdge`）。
- `root/auxiliary.rs`：`impl RootView` 的 auxiliary terminal 编排方法（`open_auxiliary_terminal`/`sync_auxiliary_terminal`）。
- `root/view.rs`：`impl RootView` 的 render 方法（`resize_handle`/`terminal_resize_handle`/`inspector_resize_handle`/`resize_shield`/`terminal_card`/`preview_workbench`/`close_confirmation`/`status_banner`）+ 游离渲染辅助（`preview_control`/`preview_hint`）。
- `root/tests.rs`：旧 `#[cfg(test)] mod tests` 内联测试整体迁出。
- `store/mod.rs`：保留 `SessionStore`/`StoreRuntime` 结构与编排；投影相关保持单点于 `store/projection.rs`。

### 2.2 目标模块拓扑

```text
homie/crates/homie-app/src/
├── root.rs             # RootView 组合根（实体 + 核心编排 + Render）
├── root/
│   ├── shortcuts.rs    # 快捷键/导航纯策略
│   ├── seams.rs        # seam 动画 + 拖拽边缘 marker
│   ├── auxiliary.rs    # auxiliary terminal 编排
│   ├── view.rs         # render 方法 + 渲染辅助
│   └── tests.rs        # 旧内联测试迁出
└── store/
    ├── mod.rs          # SessionStore/StoreRuntime（投影单点于 projection.rs）
    └── projection.rs   # session/project 单一投影（已有，保持单点）
```

### 2.3 可见性规则

- 跨子模块访问的字段/方法/函数统一 `pub(crate)`。
- 对外公开 API 保持不变：`RootView`、`SessionStore`、`StoreRuntime`、`SidebarProjection` 等。
- 子模块内部辅助标记为 `pub(crate)` 或保持私有（仅在 `root.rs` 内经 `use` 引入）。

### 2.4 实施顺序

- S1 抽出 `root/shortcuts.rs` 纯策略。
- S2 抽出 `root/seams.rs` seam 动画 + 拖拽边缘 marker。
- S3 抽出 `root/auxiliary.rs` auxiliary terminal 编排。
- S4 抽出 `root/view.rs` render 方法 + 渲染辅助。
- S5 抽出 `root/tests.rs` + `root.rs` 门面收尾。
- S6 评估 store 投影单点化（F8），确认 `projection.rs` 已为单一来源或消除实际重复。
每步之后 `cargo check -p homie-app` 与 `cargo test -p homie-app` 全绿。

## 3. 测试与验收

- 验收：
  - `cargo check -p homie-app` 无 error/warning。
  - `cargo fmt --check` 通过。
  - `cargo test -p homie-app` 全绿（303 passed / 0 failed / 1 ignored）。
  - 旧 root 内联测试原样迁至 `root/tests.rs` 且全部通过。
  - `root.rs` 行数显著下降，shortcut/seam/auxiliary/render 各职责落入对应子模块。
  - store session/project 投影单一来源（`store/projection.rs`），无明显重复投影。
  - 公开 API 与视觉/行为完全不变。
  - 评估 `specs/gpui-shell.md` 是否需要更新并记录。
- 证据目录：`docs/verification/app-root-store-deepening/`

## 4. Beads 追踪

- change_id `app-root-store-deepening`；parent `homie-ubu`；child `homie-ubu.7`；P1。
