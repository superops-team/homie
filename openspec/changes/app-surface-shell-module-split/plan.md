# OpenSpec Plan — app-surface-shell-module-split

## 概述

将 `homie/crates/homie-app/src/surface_shell.rs`（约 4,362 行）机械拆分为 `surface_shell/` 子模块目录，纯逻辑子域（host 表单状态机 / host 初始化生命周期 / 投影）与 GPUI 渲染分离，公共 API 与行为完全不变。

## 模块划分与依赖

```text
surface_shell/
├── mod.rs          facade（常量 + actions! + Surface/SettingsMenu + UtilitySurfaces 结构 +
│                   非渲染 impl + impl Focusable），依赖 host_editor/host_init/projection/view
├── host_editor.rs  host 表单状态机（HostFormField/HostEditor/text_editor），无 GPUI 渲染
├── host_init.rs    host 初始化生命周期（HostPreparationKind/HostInitialization/
│                   HostInitializationCardModel/expire_completed_reinstall），无 GPUI 渲染
├── projection.rs   纯投影（ui_agent/ui_default_agent/folder_name/relative_parent/
│                   update_detail/relative_time），无 GPUI 渲染
├── view.rs         渲染（impl Render + render_* 方法 + 自由渲染辅助），依赖 mod/投影
└── tests.rs        16 测试原样迁移（use super::*）
```

依赖方向：`view → mod → {host_editor, host_init, projection}`；`host_editor → mod(常量)`。

## 任务清单

| Task | 描述 | 验收 | 关联验证 Case |
|------|------|------|---------------|
| S1 | 抽取纯投影 projection.rs | cargo test 全绿 | C1 |
| S2 | 抽取 host 表单状态机 host_editor.rs | cargo test 全绿 | C2 |
| S3 | 抽取 host 初始化生命周期 host_init.rs | cargo test 全绿 | C3 |
| S4 | 渲染收敛到 view.rs | cargo test 全绿 | C4 |
| S5 | 测试迁移到 tests.rs + facade 收尾 | cargo test 全绿 + fmt/check | C5 |

## 验证口径

- `cargo check -p homie-app`（0 警告）
- `cargo fmt --check`
- `cargo test -p homie-app`（16 个 surface_shell 相关测试原样迁移，行为等价）
