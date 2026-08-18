# App SurfaceShell 模块拆分设计文档

## 1. 背景与动机

### 1.1 现状

`homie/crates/homie-app/src/surface_shell.rs` 约 4,362 行（165,780 字节），是 `homie-app` 中仅次于 `root.rs` 的第二大单文件。它同时承担了：

- GPUI 渲染（`impl Render` + 16 个 `render_*`/子面板方法 + 约 23 个自由渲染辅助函数）
- utility surface 组合与事件分发（`open_history`/`open_worktrees`/`open_settings`/`close_surface`/`key_down` 等）
- history / worktrees / settings / updates 子域的状态机与异步任务（`finish_history_load`、`finish_worktrees_refresh`、`prepare_host`、`persist_hosts` 等）
- host 表单编辑状态机（`HostFormField`、`HostEditor`）
- host 初始化生命周期（`HostPreparationKind`、`HostInitialization`、`HostInitializationCardModel`）
- 纯投影辅助（`ui_agent`、`ui_default_agent`、`folder_name`、`relative_parent`、`update_detail`、`relative_time`、`expire_completed_reinstall`）
- 16 个测试

该文件对应 2026-08 架构审计 finding **F4（Critical）**：单文件职责过载，纯逻辑与 GPUI 渲染耦合，无法独立单测。

### 1.2 治理依据

本拆分是 `homie-ubu`（Homie 架构治理总纲，2026-08 审计）下 `homie-ubu.5` 子任务的落地，沿用前序 `app-terminal-pane-module-split`（`homie-ubu.4`）已验证的拆分模式：纯逻辑子域（表单状态机 / 生命周期 / 投影）与 GPUI 渲染分离，facade 保留公共 API 与事件分发。

## 2. 目标

- 将纯逻辑子域抽取为可独立单测的 `pub(crate)` 模块，**无 `Window`/`Context`/`Entity`/渲染依赖**：
  - `host_editor.rs`：`HostFormField` + `HostEditor`（host 表单编辑状态机）
  - `host_init.rs`：`HostPreparationKind` + `HostInitialization` + `HostInitializationCardModel`（host 初始化生命周期）
  - `projection.rs`：`ui_agent` / `ui_default_agent` / `folder_name` / `relative_parent` / `update_detail` / `relative_time`
- 渲染逻辑（`impl Render` + 16 个渲染方法 + 自由渲染辅助函数）收敛到 `view.rs`。
- `mod.rs` 作为 facade，保留常量、`actions!`、`Surface`/`SettingsMenu` 枚举、`UtilitySurfaces` 结构 + 非渲染 `impl` + `impl Focusable`。
- 保持视觉与行为完全不变（纯机械重构）。

## 3. 非目标

- 不重做 surface shell 组合/视觉设计。
- 不改变 GPUI 层级、事件流或 history/worktrees/settings/hosts 的异步状态机行为。
- 不迁移全局 store、不改动 `homie-term`/`homie-client`/`homie-proto`。
- 不新增功能、不调整任何公开签名。

## 4. 目标模块拓扑

```text
homie/crates/homie-app/src/surface_shell/
├── mod.rs          # facade：常量、actions!、Surface/SettingsMenu 枚举、
│                   #        UtilitySurfaces 结构 + 非渲染 impl（new/colors/generation/
│                   #        history/worktrees/settings/hosts 状态与事件分发）、impl Focusable
├── host_editor.rs  # HostFormField + impl、HostEditor + impl、text_editor
├── host_init.rs    # HostPreparationKind、HostInitialization + impl、
│                   # HostInitializationCardModel、expire_completed_reinstall
├── projection.rs   # ui_agent、ui_default_agent、folder_name、relative_parent、
│                   # update_detail、relative_time
├── view.rs         # impl Render + 16 个渲染方法 + 自由渲染辅助函数
└── tests.rs        # 16 测试 + utility_surfaces_for_unit_tests/history_entry/worktree_entry 辅助
```

依赖方向：`view → mod → {host_editor, host_init, projection}`；`host_editor → mod(常量)`；`host_init/projection` 无反向依赖。

## 5. 公共 API 兼容性约束

`surface_shell` 模块在 `main.rs` 中以 `mod surface_shell;` 声明（私有模块）。唯一跨模块可达的公共类型是：

- `root.rs`：`use crate::surface_shell::UtilitySurfaces;` → `UtilitySurfaces` 必须保持 `pub`，其 `new` / `open_history` / `open_worktrees` / `open_settings` / `open_add_remote_host` / `toggle_history` / `key_down` / `is_open` 等 `pub(crate)` 方法签名不变。

拆分后 `UtilitySurfaces` 通过 `mod.rs` 的 `pub struct` + `impl` 保持可达，签名不变。

## 6. 实施切片（每片 `cargo test -p homie-app` 全绿）

- **S1 纯投影抽取**：`projection.rs`（`ui_agent`/`ui_default_agent`/`folder_name`/`relative_parent`/`update_detail`/`relative_time`）→ `projection.rs`，`pub(crate)`。
- **S2 host 表单状态机抽取**：`HostFormField` + `impl`、`HostEditor` + `impl`、`text_editor` → `host_editor.rs`，`pub(crate)`。
- **S3 host 初始化生命周期抽取**：`HostPreparationKind`、`HostInitialization` + `impl`、`HostInitializationCardModel`、`expire_completed_reinstall` → `host_init.rs`，`pub(crate)`。
- **S4 渲染抽取**：`impl Render` + 16 个渲染方法 + 23 个自由渲染辅助函数 → `view.rs`。
- **S5 测试迁移**：16 个测试 + 3 个测试辅助 → `tests.rs`，`mod.rs` 加 `#[cfg(test)] mod tests;`。

## 7. 验收标准

- `surface_shell/` 下每个子模块职责单一；`mod.rs` 收敛为 facade（保留结构 + 非渲染 impl + 事件分发）。
- `projection.rs`/`host_editor.rs`/`host_init.rs` 无 GPUI 渲染依赖（可纯单测；`HostInitializationCardModel.tone: Rgba` 为数据字段，非渲染）。
- `cargo check -p homie-app` 零警告；`cargo fmt --check` 干净。
- `cargo test -p homie-app` 全绿（16 个 surface_shell 相关测试原样迁移，行为不变）。
- 公共 API 名 `UtilitySurfaces` 及其 `pub(crate)` 方法签名与可达性不变（`root.rs` 编译通过即证）。

## 8. 测试计划

- 现有 16 个 `#[gpui::test]` 原样迁移，作为行为等价回归门禁。
- 新增无（纯机械重构，不改变行为）。

## 9. Beads 追踪

- change_id `app-surface-shell-module-split`
- Beads：`homie-ubu.5`（IN_PROGRESS），parent `homie-ubu`
- 证据目录：`docs/verification/app-surface-shell-module-split/`
- 版本 tag：`v0.1.12`（internal refactor patch）
