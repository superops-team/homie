# 发布就绪报告 — app-surface-shell-module-split

## 变更概述

将 `homie/crates/homie-app/src/surface_shell.rs`（约 4,362 行）机械拆分为 `surface_shell/` 子模块目录，将纯逻辑子域（host 表单状态机、host 初始化生命周期、纯投影）与 GPUI 渲染分离。公共 API 与运行时行为完全不变。

- change_id：`app-surface-shell-module-split`
- Beads：`homie-ubu.5`
- 类型：refactor（机械拆分，删除旧单文件，不做向后兼容）

## 模块划分

```text
surface_shell/
├── mod.rs          facade（常量 + actions! + Surface/SettingsMenu + UtilitySurfaces 结构 + 非渲染 impl + impl Focusable）
├── host_editor.rs  host 表单状态机（HostFormField / HostEditor / text_editor），无 GPUI 渲染
├── host_init.rs    host 初始化生命周期（HostPreparationKind / HostInitialization / HostInitializationCardModel / expire_completed_reinstall）
├── projection.rs   纯投影（ui_agent / ui_default_agent / folder_name / relative_parent / update_detail / relative_time）
├── view.rs         渲染（impl Render + render_* 方法 + 自由渲染辅助）
└── tests.rs        16 测试原样迁移（use super::*）
```

依赖方向：`view → mod → {host_editor, host_init, projection}`；`host_editor → mod(常量)`。

## 交付切片 S1–S5

| 切片 | 内容 | 状态 |
|------|------|------|
| S1 | 抽取纯投影 projection.rs | 完成 |
| S2 | 抽取 host 表单状态机 host_editor.rs | 完成 |
| S3 | 抽取 host 初始化生命周期 host_init.rs | 完成 |
| S4 | 渲染收敛到 view.rs | 完成 |
| S5 | 测试迁移到 tests.rs + facade 收尾 | 完成 |

## 验证证据

| 验证项 | 命令 | 结果 |
|--------|------|------|
| 编译检查 | `cargo check -p homie-app` | 通过，0 警告 |
| 格式检查 | `cargo fmt --check` | 通过 |
| 全量测试 | `cargo test -p homie-app` | **303 passed / 0 failed / 1 ignored** |
| surface_shell 单测 | `cargo test -p homie-app surface_shell` | **16 passed / 0 failed** |

- 16 个 surface_shell 相关测试原样迁移到 `tests.rs`，行为等价，全部通过。
- 公共 API 兼容性：`root.rs` 经 `cargo check -p homie-app` 编译通过，证明 `UtilitySurfaces` 及 `pub(crate)` 方法（`new`/`open_history`/`open_worktrees`/`open_settings`/`open_add_remote_host`/`toggle_history`/`key_down`/`is_open`）签名与可达性不变。

## 已知限制与后续

- 无已知限制。
- 机械重构未改变任何运行时行为，公共 API 不变，删除旧文件、不做向后兼容。

## 结论

所有验收标准（C1–C6）均已满足，验证证据齐备，可发布。
