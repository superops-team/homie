# PRD — app-sidebar-popover-split

## 背景

`homie/crates/homie-app/src/sidebar/popover.rs`（1,249 行）是侧边栏弹出层的 God Module：单个
`impl Sidebar` 块同时承载 popover 入口分发（`popover`）、弹窗外壳（`popover_shell`/
`popover_shell_above_footer`/`popover_shell_at`）、新 agent 弹窗（`new_agent_popover`，423 行）、
目录选择器（`directory_picker`）、更新菜单行（`update_menu_row`）、账户弹窗（`account_popover`）、
项目操作菜单（`project_actions_popover`）与会话操作菜单（`session_actions_popover`），单文件远超
800 行阈值，阅读与变更成本极高，违背仓库「组件模块化、关注点清晰」原则。

## 目标

将 `popover.rs` 机械拆分为目录化聚焦子模块，按弹窗类型对齐关注点，公共 API 与运行时行为零变更，
引用方零改动，单文件 < 800 行。

## 非目标

- 不改变任何弹窗渲染逻辑、交互行为或视觉样式。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动 `Sidebar` 结构体、`Popover` 枚举、任何方法签名或可见性。
- 不合并或重命名方法。

## 用户场景

1. 开发者定位「新 agent 弹窗」渲染逻辑时，直接进入 `new_agent.rs`。
2. 开发者定位「弹窗外壳/入口分发」时，聚焦在 `shell.rs`。
3. 开发者定位「目录选择器」时，聚焦在 `directory_picker.rs`。
4. 开发者定位「账户弹窗」时，聚焦在 `account.rs`。
5. 开发者定位「项目/会话操作菜单」时，聚焦在 `actions.rs`。
6. 开发者定位「更新菜单行」时，聚焦在 `update_menu.rs`。

## 模块划分方案

```text
sidebar/popover/
├── mod.rs             facade：use super::*; + 模块声明
├── shell.rs           弹窗入口与外壳：popover/popover_shell/popover_shell_above_footer/popover_shell_at
├── new_agent.rs       新 agent 弹窗：new_agent_popover
├── directory_picker.rs 目录选择器：directory_picker
├── update_menu.rs     更新菜单行：update_menu_row
├── account.rs         账户弹窗：account_popover
└── actions.rs         项目/会话操作菜单：project_actions_popover/session_actions_popover
```

## 可见性设计

- `sidebar/mod.rs` 中 `mod popover;` 声明不变，`crate::sidebar::popover` 模块路径不变。
- 10 个方法均为 `pub(crate) fn`，迁移后保持 `pub(crate)` 可见性不变，经 `impl Sidebar` 在子模块
  实现，crate 内调用方（`crate::sidebar::view` 等）经 `Sidebar` 类型访问，路径零改动。
- 各子模块以 `use super::*;` 引入 `Sidebar` 与渲染依赖，`impl Sidebar` 跨子模块实现。
- 无 `pub` 可见性泄漏，无生产代码语义变更。

## 影响面

- 仅 `sidebar/popover.rs` → `sidebar/popover/` 目录化迁移，生产代码与其它模块零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo check -p homie-app --offline` 全绿。
- `cargo fmt --all --check` 通过。
- `cargo clippy -p homie-app --all-targets --offline` 0 警告。
- `cargo build -p homie-app --offline` 通过。
- `cargo test -p homie-app --offline` 全绿（沙箱内 2 个 `daemon_launch` socket bind EPERM 属预期）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件（1,249 行）拆为 6 子模块 + facade。
- C2：公共 API 不变，引用方零改动（`git status` 确认仅改 `popover.rs` → `popover/` 目录 + 文档）。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：10 个方法逐字迁移，`pub(crate)` 可见性保持不变。
- C6：release readiness 证据写入 `docs/verification/app-sidebar-popover-split/`。

## Beads

- `homie-156`
