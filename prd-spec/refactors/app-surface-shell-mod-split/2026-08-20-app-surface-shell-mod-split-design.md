# PRD — app-surface-shell-mod-split

## 背景

`homie/crates/homie-app/src/surface_shell/mod.rs`（904 行）是 UtilitySurfaces 外壳的 God Module：
一个文件同时承载外壳结构体定义与构造（`UtilitySurfaces`/`new`）、主题颜色切换（`colors`/
`settings_colors`）、代际计数（`next_history_generation`/`next_worktrees_generation`）、偏好持久化
（`persist_prefs`）、外壳开合（`close_surface`/`is_open`/`open_settings`/`open_add_remote_host`/
`toggle_history`/`key_down`）、历史面板逻辑（`open_history`/`finish_history_load`/
`finish_history_resume`/`resume_history`/`visible_history`/`move_history`/`activate_history`）、
worktree 面板逻辑（`open_worktrees`/`refresh_worktrees`/`finish_worktrees_refresh`/
`confirm_cleanup`）、主机管理逻辑（`reload_hosts`/`begin_adding_host`/`begin_editing_host`/
`select_host_field`/`save_host`/`initialize_host`/`reinstall_host`/`prepare_host`/
`retry_host_initialization`/`request_remove_host`/`persist_hosts`/`edit_host_field`/
`handle_host_editor_key`）以及 `Focusable` impl，单文件超过 800 行阈值，阅读与变更成本高，
违背仓库「组件模块化、关注点清晰」原则。

## 目标

将 `surface_shell/mod.rs` 机械拆分为目录化聚焦子模块，公共 API 与运行时行为零变更，
引用方零改动，单文件 < 800 行。

## 非目标

- 不改变任何历史加载、worktree 刷新、主机增删改、偏好持久化、外壳开合行为。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动 `UtilitySurfaces` 结构体字段、`Surface` 枚举、`new`/`is_open`/`open_settings`/
  `open_add_remote_host`/`toggle_history`/`key_down` 等公开函数签名。

## 用户场景

1. 开发者定位「历史面板加载/恢复/移动/激活」时，直接进入 `history.rs`。
2. 开发者定位「worktree 面板刷新/确认清理」时，聚焦在 `worktrees.rs`。
3. 开发者定位「主机增删改/初始化/重装/字段编辑」时，聚焦在 `hosts.rs`。
4. 开发者定位「外壳结构体/主题颜色/偏好持久化/开合」时，聚焦在 `mod.rs`。

## 模块划分方案

```text
surface_shell/
├── mod.rs         facade：结构体定义 + new + 主题颜色 + 代际计数 + persist_prefs +
│                  close_surface/is_open/open_settings/open_add_remote_host/toggle_history/
│                  key_down + Focusable + 模块声明
├── history.rs     历史面板：open_history/finish_history_load/finish_history_resume/
│                  resume_history/visible_history/move_history/activate_history
├── worktrees.rs   worktree 面板：open_worktrees/refresh_worktrees/finish_worktrees_refresh/
│                  confirm_cleanup
└── hosts.rs       主机管理：reload_hosts/begin_adding_host/begin_editing_host/select_host_field/
                   save_host/initialize_host/reinstall_host/prepare_host/
                   retry_host_initialization/request_remove_host/persist_hosts/edit_host_field/
                   handle_host_editor_key
```

职责边界：
- `mod.rs` 保留：`UtilitySurfaces` 结构体 + `new` 构造 + `colors`/`settings_colors` 主题颜色 +
  `next_history_generation`/`next_worktrees_generation` 代际计数 + `persist_prefs` +
  `close_surface`/`is_open`/`open_settings`/`open_add_remote_host`/`toggle_history`/`key_down` +
  `Focusable` impl + 全部 `mod` 声明 + `tests` 声明。
- `history.rs` 下沉：`open_history`/`finish_history_load`/`finish_history_resume`/
  `resume_history`/`visible_history`/`move_history`/`activate_history`。
- `worktrees.rs` 下沉：`open_worktrees`/`refresh_worktrees`/`finish_worktrees_refresh`/
  `confirm_cleanup`。
- `hosts.rs` 下沉：`reload_hosts`/`begin_adding_host`/`begin_editing_host`/`select_host_field`/
  `save_host`/`initialize_host`/`reinstall_host`/`prepare_host`/`retry_host_initialization`/
  `request_remove_host`/`persist_hosts`/`edit_host_field`/`handle_host_editor_key`。

## 可见性设计

- 公共 API（`UtilitySurfaces::new`、`is_open`、`open_settings`、`open_add_remote_host`、
  `toggle_history`、`key_down`、`open_history`、`open_worktrees` 等）路径不变，仍通过
  `crate::surface_shell::UtilitySurfaces` 可达。
- 跨子模块互调采用 `pub(super)`（仅对父模块 `surface_shell` 可见，不泄漏到 crate 外）：
  - `history.rs`：`finish_history_load`/`resume_history`/`visible_history`/`move_history`/
    `activate_history` → `pub(super)`；`finish_history_resume` 保持私有（仅 `resume_history` 调用）。
  - `worktrees.rs`：`refresh_worktrees`/`finish_worktrees_refresh`/`confirm_cleanup` →
    `pub(super)`（其中 `finish_worktrees_refresh`/`finish_worktrees_resume` 被 tests.rs 调用）。
  - `hosts.rs`：`reload_hosts`/`begin_adding_host`/`begin_editing_host`/`select_host_field`/
    `save_host`/`reinstall_host`/`retry_host_initialization`/`request_remove_host`/
    `handle_host_editor_key` → `pub(super)`；`initialize_host`/`prepare_host`/`persist_hosts`/
    `edit_host_field` 保持私有（仅同文件调用）。
  - `tests.rs`：显式导入 `super::host_init::expire_completed_reinstall`，不再依赖 `super::*` 的
    私有 `use` 泄漏。
- 无 `pub` 可见性泄漏到 crate 外，所有跨模块符号均为 `pub(super)` 或保持私有。

## 影响面

- 引用方：`sidebar/view.rs` 调用 `reload_hosts`（已 `pub(super)` 保持可达）；`view.rs`/
  `settings_view.rs`/`hosts_view.rs` 通过 `use super::*` 自动拿到 mod.rs 可见方法，零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo check -p homie-app --offline` 全绿。
- `cargo fmt --all --check` 通过。
- `cargo clippy -p homie-app --all-targets --offline` 0 警告。
- `cargo build -p homie-app --offline` 通过。
- `cargo test -p homie-app --offline` 全绿（沙箱内 2 个 `daemon_launch` EPERM 属预期）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件瘦身为 facade。
- C2：公共 API 不变，引用方零改动（`git status` 确认仅改 `surface_shell/` 目录 + 文档）。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：无 `pub` 可见性泄漏到 crate 外（跨模块符号均为 `pub(super)` 或私有）。
- C6：release readiness 证据写入 `docs/verification/app-surface-shell-mod-split/`。

## Beads

- `homie-9uw`
