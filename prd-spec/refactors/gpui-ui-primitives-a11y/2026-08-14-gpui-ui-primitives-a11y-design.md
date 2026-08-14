# GPUI 语义 UI primitives 与 a11y 首个切片设计文档

## 1. 概述

### 1.1 问题

Homie GPUI app 中大量按钮类控件直接使用 `div().cursor_pointer().on_click(...)`，语义、尺寸、状态和可访问性行为分散在业务文件中。父级 `gpui-architecture-hardening` 要求先建立可复制的 semantic primitive。

### 1.2 目标

1. 在 `homie-ui` 中新增最小 `Button` primitive。
2. 支持 stable id、variant、size、disabled、icon/text child、点击回调。
3. 迁移一个小型真实控件：Settings dialog 的 `close-settings`。
4. 增加 focused tests 验证 disabled 和 variant 样式策略。

### 1.3 非目标

- 不一次性迁移所有裸 click 控件。
- 不实现完整 AccessKit role API，若 pinned GPUI 缺少本地稳定示例则在后续 child change 继续。
- 不改视觉设计。

## 2. 方案

- 在 `homie-ui/src/components.rs` 新增 `Button`、`ButtonVariant`、`ButtonSize`。
- `Button` 使用 `RenderOnce`，调用方传入 id、colors、variant、size 和内容。
- 支持 `disabled`，disabled 时不绑定 click handler。
- 在 `surface_shell.rs` 用 `Button` 替换 `close-settings`。

## 3. 验证

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-ui button -- --nocapture
cargo test --manifest-path homie/Cargo.toml -p homie-app close_settings -- --nocapture
(cd homie && cargo fmt --check)
git diff --check
```

## 4. 验收

1. `homie-ui` 导出 `Button` primitive。
2. `close-settings` 使用 `Button`。
3. Disabled button 不触发 click handler。
4. Targeted tests 通过。

## 5. Beads

- Beads: `homie-0aj`
- change_id: `gpui-ui-primitives-a11y`
