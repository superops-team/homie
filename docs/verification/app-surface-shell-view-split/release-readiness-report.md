# 发布就绪报告 — app-surface-shell-view-split

## 变更概述

将 `homie/crates/homie-app/src/surface_shell/view.rs`（2,475 行，homie-app 最大 God Module）
机械拆分为聚焦子模块：应用设置渲染、远端主机管理渲染、通用 UI 原语各自下沉，
`view.rs` 收尾为 facade。公共 API 与运行时渲染行为完全不变，单文件 < 800 行。

- change_id：`app-surface-shell-view-split`
- Beads：`homie-75w`
- 类型：task（机械拆分，行为不变）
- 上游：`architecture-audit-governance-2026-08`（F4 延续）

## 模块划分

```text
surface_shell/
├── mod.rs           facade（常量 + actions! + Surface/SettingsMenu + UtilitySurfaces 结构 + 非渲染 impl + impl Focusable）
├── view.rs          facade 渲染（render_history / render_worktrees + impl Render），431 行
├── settings_view.rs 应用设置渲染（render_settings + general/default_agent/update/terminal/resource/terminal_theme/hibernate/memory），765 行
├── hosts_view.rs    远端主机管理渲染（remote_settings/remote_hosts_section/host_initialization_card/host_editor_panel/host_text_field），757 行
├── widgets.rs       通用 UI 原语（21 个自由渲染辅助），537 行
└── tests.rs         既有测试（仅调整 setting_row 导入路径）
```

依赖方向：`view → widgets`；`settings_view → {mod, widgets, hosts_view(remote_settings)}`；
`hosts_view → {mod, widgets}`；`widgets → mod`；`tests → widgets(setting_row)`。

## 交付切片 T1–T5

| 切片 | 内容 | 状态 |
|------|------|------|
| T1 | 抽取通用 UI 原语 widgets.rs | 完成 |
| T2 | 抽取远端主机管理渲染 hosts_view.rs | 完成 |
| T3 | 抽取应用设置渲染 settings_view.rs | 完成 |
| T4 | view.rs 收尾为 facade | 完成 |
| T5 | 全量验证 + code review + release readiness | 完成 |

## 验证证据

| 验证项 | 命令 | 结果 |
|--------|------|------|
| 编译检查 | `cargo check -p homie-app` | 通过，0 警告 |
| 格式检查 | `cargo fmt --all --check` | 通过 |
| 静态检查 | `cargo clippy -p homie-app --all-targets` | 通过，0 警告 |
| 全量测试 | `cargo test -p homie-app` | **303 passed / 0 failed / 1 ignored** |
| 单文件行数 | `wc -l` | view 431 / settings_view 765 / hosts_view 757 / widgets 537，均 < 800 |

- 所有 surface_shell 相关测试原样通过，渲染行为与拆分前等价。
- 公共 API 兼容性：`impl Render for UtilitySurfaces` 入口签名不变，`UtilitySurfaces` 结构体字段与 `Render` 契约未改动，`cargo check -p homie-app` 编译通过证明可达性不变。
- 仅 `render_settings` / `remote_settings` 两个跨模块调用方法升为 `pub(super)`，21 个 UI 原语升为 `pub(super)`，无 `pub` 泄漏到 crate 外。

## 已知限制与后续

- 无已知限制。
- 机械重构未改变任何运行时行为、公共 API 不变；纯职责搬迁，不做向后兼容。

## 结论

所有验收标准（C1–C6）均已满足，验证证据齐备，可发布。
