# OpenSpec Plan — app-surface-shell-view-split

## 概述

将 `homie/crates/homie-app/src/surface_shell/view.rs`（2,475 行，homie-app 最大 God Module）
机械拆分为聚焦子模块，应用设置渲染、远端主机管理渲染、通用 UI 原语各自下沉，
`view.rs` 收尾为 facade。公共 API 与运行时渲染行为完全不变，单文件 < 800 行。

## 模块划分与依赖

```text
surface_shell/
├── mod.rs           facade（常量 + actions! + Surface/SettingsMenu + UtilitySurfaces 结构 +
│                    非渲染 impl + impl Focusable）
├── view.rs          facade 渲染（render_history / render_worktrees + impl Render），< 800 行
├── settings_view.rs 应用设置渲染（render_settings + general/default_agent/update/terminal/
│                    resource/terminal_theme/hibernate/memory），< 800 行
├── hosts_view.rs    远端主机管理渲染（remote_settings/remote_hosts_section/
│                    host_initialization_card/host_editor_panel/host_text_field），< 800 行
├── widgets.rs       通用 UI 原语（surface_button/settings_*_button/danger_button/toggle_row/
│                    setting_section/setting_row/setting_text_stack/wrappable_setting_copy/
│                    settings_note/settings_page/setting_divider/settings_select_button/
│                    settings_dropdown/settings_choice_row/theme_preview/chip/colored_badge/
│                    empty_label/host_field_value/text_offset_for_x），< 800 行
└── tests.rs         既有测试（仅调整 setting_row 导入路径）
```

依赖方向：`view → mod`、`view → widgets`；`settings_view → mod`、`settings_view → widgets`、
`settings_view → hosts_view(remote_settings)`；`hosts_view → mod`、`hosts_view → widgets`；
`widgets → mod`（读常量/颜色）；`tests → widgets(setting_row)`。

## 任务清单

| Task | 描述 | 验收 | 关联验证 Case |
|------|------|------|---------------|
| T1 | 抽取 widgets.rs（通用 UI 原语） | cargo check 全绿 | C1 |
| T2 | 抽取 hosts_view.rs（远端主机管理渲染） | cargo check 全绿 | C2 |
| T3 | 抽取 settings_view.rs（应用设置渲染） | cargo check 全绿 | C3 |
| T4 | view.rs 收尾为 facade | 单文件 < 800 | C4 |
| T5 | 全量验证 + code review + release readiness | fmt/check/test 全绿 | C5/C6 |

## 验证口径

- `cargo fmt --check`
- `cargo check -p homie-app`
- `cargo test -p homie-app`（surface_shell 相关测试原样通过，行为等价）
- `view.rs` / `settings_view.rs` / `hosts_view.rs` / `widgets.rs` 均 < 800 行
