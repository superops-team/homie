# GPUI task/subscription 生命周期所有权硬化设计文档

## 1. 概述

### 1.1 问题/动机

`gpui-architecture-hardening` 要求重要 entity subscription 必须由 owner 显式持有，不应默认 `.detach()`。当前 `RootView` 已经有 `_subscriptions: Vec<Subscription>` 字段，但 terminal、sidebar、launcher、navigation、inspector 的 child event subscriptions 仍在构造阶段直接 `.detach()`。

### 1.2 目标

1. 将 `RootView::new` 中的 child entity subscriptions 显式存入 `_subscriptions`。
2. 保持现有事件处理逻辑和 UI 行为不变。
3. 不处理 service lifetime tasks、hover delay、host initialization 等非 subscription 工作。
4. 增加静态验证，防止 `root.rs` 中 `cx.subscribe*().detach()` 回流。

### 1.3 非目标

- 不重构 RootView 字段结构。
- 不修改 UtilitySurfaces history/worktrees 逻辑，该部分已由 `homie-yon` 覆盖。
- 不处理非 subscription 的 `.detach()`。

## 2. 方案

- 在 `RootView::new` 中把 terminal/sidebar/launcher/navigation/inspector 的 `Subscription` 返回值存到局部 `subscriptions`。
- 构造 `RootView` 时将 activation、bounds observer 和 child subscriptions 一起放入 `_subscriptions`。
- 保持 service tasks 仍使用 `_service_events`、`_surface_sync`、`_workbench_sync` 字段。

## 3. 验证计划

```bash
rg -n "cx\\.subscribe(_in)?\\([\\s\\S]{0,200}\\.detach\\(\\)" homie/crates/homie-app/src/root.rs
cargo test --manifest-path homie/Cargo.toml -p homie-app root -- --nocapture
cargo test --manifest-path homie/Cargo.toml -p homie-app surface_shell::tests -- --nocapture
(cd homie && cargo fmt --check)
git diff --check
```

第一条命令预期无命中。

## 4. 验收标准

1. `root.rs` 中 child entity subscriptions 不再直接 `.detach()`。
2. `_subscriptions` 包含 activation、bounds observer 和 child subscriptions。
3. RootView 相关测试和 surface_shell 回归测试通过。
4. 不修改 `homie-ui`、daemon/client API 或视觉输出。

## 5. Beads

- Beads: `homie-9w2`
- change_id: `gpui-lifecycle-task-ownership`
- parent: `homie-4lu`
