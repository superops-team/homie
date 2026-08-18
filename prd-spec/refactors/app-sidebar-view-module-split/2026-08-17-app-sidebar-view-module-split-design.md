# App Sidebar View 模块拆分设计文档

## 1. 概述

### 1.1 问题/动机

`homie/crates/homie-app/src/sidebar/view.rs` 约 4,310 行，混合承担 sidebar 核心实体与 Render、
各 section 渲染（project / session / archived）、popover、directory picker、drag/drop、排序、
快捷键、操作 dispatch 与旧内联测试。2026-08 架构审计 finding **F5（Critical）**：单文件过大、
职责混杂，难以定位、复用与单测。

### 1.2 目标

把 `view.rs` 按职责机械拆分为聚焦子模块，每个子模块只承载一类职责，便于定位、复用与单测。
保持公开 API、视觉与运行时行为完全不变。

### 1.3 非目标

不重做 sidebar 设计/系统；不改变 GPUI 层级；不新增抽象层、兼容层或向后兼容代码；
不把仍绑定 `impl Sidebar` 上下文的渲染方法强行拆成纯函数。

### 1.4 基线

- branch `main`，commit `e4c7454`；目标 `sidebar/view.rs`（4,310 行）。
- 复用 `gpui-large-module-test-boundaries` 已抽出的 `sidebar/picker_logic.rs`。

## 2. 方案设计

### 2.1 拆分原则

纯机械拆分：只移动代码、调整 `pub(crate)` 可见性、保留原有注释与实现；不重写逻辑。

- `view.rs`：保留 `Sidebar` 结构体、`EventEmitter`/`Focusable` 实现、核心 `impl Sidebar` 方法与
  `impl Render`（对应原文件 ~126–510 与 ~3385–3448）。
- `sections.rs`：`impl Sidebar` 的各 section 渲染方法（原 ~512–1698）。
- `popover.rs`：`impl Sidebar` 的 popover 相关方法（原 ~1699–2942）。
- `commands.rs`：`impl Sidebar` 的命令/选择/拖拽 dispatch 方法（原 ~2943–3384）。
- `render_helpers.rs`：游离渲染辅助函数（原 ~3449–3646 与 ~3655–3723）。
- `projection.rs`：纯投影/计算函数（原 ~3647–3654 与 ~3724–3930）。
- `tests.rs`：旧 `#[cfg(test)] mod tests` 内联测试整体迁出（原 ~3934–4309）。
- `mod.rs`：导入、`pub use` 再导出与 `const PREVIEW_USAGE`。
- `state.rs`、`picker_logic.rs`、`fixture.rs`：保持不变。

### 2.2 目标模块拓扑

```text
homie/crates/homie-app/src/sidebar/
├── mod.rs             # 导入/再导出 + const
├── view.rs            # Sidebar 实体 + 核心 impl + Render（~540 行）
├── sections.rs        # project/session/archived 渲染（~1200 行）
├── popover.rs         # popover 相关方法（~1250 行）
├── commands.rs        # 命令/选择/拖拽 dispatch（~420 行）
├── render_helpers.rs  # 游离渲染辅助函数（~285 行）
├── projection.rs      # 纯投影函数（~217 行）
├── tests.rs           # 旧内联测试迁出（~376 行）
├── state.rs           # 已有（不变）
├── picker_logic.rs    # 已有（不变）
└── fixture.rs         # 已有（不变）
```

### 2.3 可见性规则

- 跨子模块互相访问的结构体字段、方法与函数统一改为 `pub(crate)`。
- 对外公开 API 保持不变：`Sidebar`、`SidebarEvent`、`SidebarUiState`、`Popover`、`DragItem`、
  `PreviewScenario`、`SidebarPreviewFixture`、`move_before`、`move_to_end`。
- `DraggedSidebarItem`、`DragPreview` 为 `pub(crate)`（仅模块内使用）。

### 2.4 实施顺序

- S1 抽出 `projection.rs` 纯投影函数。
- S2 抽出 `render_helpers.rs` 渲染辅助函数。
- S3 抽出 `sections.rs` section 渲染方法。
- S4 抽出 `popover.rs` popover 方法。
- S5 抽出 `commands.rs` dispatch 方法 + `tests.rs` + `mod.rs` 门面。
每步之后 `cargo check -p homie-app` 与 `cargo test -p homie-app` 全绿。

## 3. 测试与验收

- 验收：
  - `cargo check -p homie-app` 无 error/warning。
  - `cargo fmt --check` 通过。
  - `cargo test -p homie-app` 全绿（303 passed / 0 failed / 1 ignored）。
  - 旧 sidebar 内联测试（16 个）原样迁至 `tests.rs` 且全部通过。
  - 单文件不再 4,310 行；核心 `view.rs` 聚焦实体 + Render，各职责落入对应子模块。
  - 公开 API 与视觉/行为完全不变（纯机械拆分，无逻辑改写）。
- 证据目录：`docs/verification/app-sidebar-view-module-split/`

## 4. Beads 追踪

- change_id `app-sidebar-view-module-split`；parent `homie-ubu`；child `homie-ubu.6`；P1。
