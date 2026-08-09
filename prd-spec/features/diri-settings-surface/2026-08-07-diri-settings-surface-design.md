# Diri 设置界面对齐设计文档

```yaml
change_id: diri-settings-surface
beads: homie-s0w
parent_bead: homie-h7n.4
target_rows:
  - UI-006
```

## 1. 概述

### 1.1 问题/背景

`UI-006` 在 parity lock 中仍为 `missing`：Homie 没有 Settings General/Terminal/Resources/Remote tabs，也没有通过 storage `preferences` 表持久化偏好。当前 command palette 的 `OpenSettings` 只修改本地 notice，属于假交互。

### 1.2 目标

- 为 `homie-storage` 增加 typed preference get/set API。
- 为 `homie-app` 增加 settings surface，包含 General、Terminal、Resources、Remote 四个 tab。
- `OpenSettings` command 必须打开真实 settings surface。
- settings surface 必须显示并可切换持久化偏好：startup behavior、terminal font size、hibernate idle minutes、remote companion access。
- 通过测试证明 preferences 可持久化，app 不再只有 notice。

## 2. 用户场景

### 场景 1: 打开设置

**Given** 用户打开 command palette  
**When** 执行 Settings  
**Then** app 显示设置面板，而不是只在状态栏显示 `settings`。

### 场景 2: 修改终端字号

**Given** settings surface 打开  
**When** 用户选择 Terminal tab 并调整字号  
**Then** Homie 将 `terminal.font_size` 写入 `preferences`，重启后能读取。

### 场景 3: Remote companion access

**Given** 用户进入 Remote tab  
**When** 切换 companion access  
**Then** Homie 将偏好持久化，但不展示 token secret。

## 3. 功能需求

### FR-1: Storage Preferences API

`homie-storage` 必须提供 `get_preference_json`、`set_preference_json` 和 typed `load_settings_preferences`/`save_settings_preferences`。

### FR-2: Settings Surface

`homie-app` 必须维护 `settings_visible`、`settings_tab` 和 `settings_preferences` 状态，并渲染 General/Terminal/Resources/Remote tabs。

### FR-3: OpenSettings 真实交互

`PaletteCommand::OpenSettings` 必须调用 `open_settings`，打开 settings surface 并加载持久化偏好。

### FR-4: 偏好持久化

至少持久化：

- `general.startup_behavior`
- `terminal.font_size`
- `resources.hibernate_idle_minutes`
- `remote.companion_access`

## 4. 非目标

- 不实现完整 remote pairing/token 管理。
- 不实现系统设置窗口多页面复杂导航。
- 不把 `UI-006` 标为 implemented，直到有真实交互 E2E。

## 5. 验收标准

- `cargo test -p homie-storage --test storage_bootstrap -- --nocapture` 覆盖 preferences API。
- `cargo test -p homie-app --tests -- --nocapture` 覆盖 Settings 不再是 notice-only。
- `cargo clippy -p homie-storage -p homie-app --all-targets -- -D warnings` 通过。
- parity lock 更新 `UI-006` 为 `partial`，记录证据。

