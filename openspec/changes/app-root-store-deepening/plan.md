# OpenSpec Plan — app-root-store-deepening

## 概述

将 `homie/crates/homie-app/src/root.rs`（约 2,130 行）机械拆分为 `root/` 下的聚焦子模块目录，把 shortcut 策略、seam 几何、auxiliary terminal 编排、render 方法下沉为职责单一的子模块；同时评估 store session/project 投影是否已收敛为单一 source of truth。公共 API 与运行时行为完全不变。

## 模块划分与依赖

```text
root/
├── mod.rs          facade（RootView 结构体 + Focusable + 核心编排 + Render + 子模块 use）
├── shortcuts.rs    NewSessionShortcut 枚举 + new_session_shortcut + session_navigation_delta 纯策略
├── seams.rs        advance_seam 纯函数 + DraggedSidebarEdge/DraggedTerminalEdge/DraggedInspectorEdge marker
├── auxiliary.rs    impl RootView：open_auxiliary_terminal / sync_auxiliary_terminal 编排
├── view.rs         impl RootView render 方法 + 游离渲染辅助（preview_control / preview_hint）
└── tests.rs        旧 #[cfg(test)] mod tests 内联测试迁出（use super::*）
```

依赖方向：`auxiliary/view → mod（RootView 实体 + pub(crate) 字段/方法）`；`shortcuts/seams` 为纯策略无逆向依赖；`mod → {shortcuts, seams, auxiliary, view}`。

store 投影保持单点于 `store/projection.rs`，`store/mod.rs` 只做 `SessionStore/StoreRuntime` 编排。

## 任务清单

| Task | 描述 | 验收 | 关联验证 Case |
|------|------|------|---------------|
| S1 | 抽取 `root/shortcuts.rs` 纯策略 | cargo check 全绿 | C1 |
| S2 | 抽取 `root/seams.rs` seam 动画 + 拖拽边缘 marker | cargo check 全绿 | C2 |
| S3 | 抽取 `root/auxiliary.rs` auxiliary terminal 编排 | cargo check 全绿 | C3 |
| S4 | 抽取 `root/view.rs` render 方法 + 渲染辅助 | cargo check 全绿 | C4 |
| S5 | 抽取 `root/tests.rs` + facade 收尾 | cargo test 全绿 + fmt/check | C5 |
| S6 | 评估 store session/project 投影单点化（F8） | 记录评估结论，无重复投影 | C6 |

## 验证口径

- `cargo check -p homie-app`（0 error / 0 warning）
- `cargo fmt --check`
- `cargo test -p homie-app`（303 passed / 0 failed / 1 ignored）
- 旧 root 内联测试原样迁移至 `root/tests.rs` 且全部通过
