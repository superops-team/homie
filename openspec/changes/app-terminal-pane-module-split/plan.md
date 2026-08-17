# OpenSpec Plan — app-terminal-pane-module-split

## 概述

将 `homie/crates/homie-app/src/terminal_pane.rs`（约 3,495 行）机械拆分为 `terminal_pane/` 子模块目录，纯逻辑与 GPUI 渲染分离，公共 API 与行为完全不变。

## 模块划分与依赖

```text
terminal_pane/
├── mod.rs         facade（结构定义 + 事件分发），依赖 chip/attachment/projection/policy/keys/view
├── chip.rs        纯投影（依赖 homie_proto、homie_ui、projection），无 GPUI 渲染
├── attachment.rs  attachment 生命周期（依赖 homie_client、mod::PaneEvent、常量 REATTACH_DELAY）
├── projection.rs  纯投影（依赖 homie_proto、homie_ui）
├── policy.rs      纯决策（依赖常量、CellMetrics）
├── keys.rs        键位适配（依赖 homie_term、gpui KeyDownEvent）
├── view.rs        渲染（依赖 mod::*、chip、projection、常量）
└── tests.rs       24 个测试原样迁移（use super::*）
```

依赖方向：`view → mod → {chip, attachment, projection, policy, keys}`；`chip → projection`；`attachment → mod(PaneEvent)`。

## 任务清单

| Task | 描述 | 验收 | 关联验证 Case |
|------|------|------|---------------|
| S1 | 抽取纯函数 keys/policy/projection | cargo test 全绿 | C1 |
| S2 | 抽取 chip 投影 | cargo test 全绿 | C2 |
| S3 | 抽取 attachment 生命周期 | cargo test 全绿 | C3 |
| S4 | 抽取渲染到 view.rs | cargo test 全绿 | C4 |
| S5 | 迁移测试到 tests.rs + facade 收尾 | cargo test 全绿 + fmt/check | C5 |

## 验证口径

- `cargo check -p homie-app`（0 警告）
- `cargo fmt --check`
- `cargo test -p homie-app`（303 passed / 0 failed / 1 ignored，行为等价）
