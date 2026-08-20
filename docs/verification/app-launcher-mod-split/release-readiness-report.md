# Release Readiness — app-launcher-mod-split

## 变更摘要

将 `homie/crates/homie-app/src/launcher.rs`（1,046 行）机械拆分为 2 个聚焦子模块，
`mod.rs` 收尾 facade（522 行）。公共 API 与运行时行为零变更，引用方零改动。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 522 | 布局常量 + 状态模型 + 生命周期/事件/提交/编辑 + Focusable + 纯函数辅助 |
| `render.rs` | 520 | harness/project picker 与面板/composer 渲染 + Render impl |
| `tests.rs` | 7 | 既有测试（原样下沉） |

全部单文件 < 800 行。

## 可见性管控

渲染方法同模块内聚，零 `pub(super)` 新增，无 `pub` 可见性泄漏到 crate 外：
- `render.rs` 作为 `impl super::LauncherOverlay` 扩展块 + `impl Render for super::LauncherOverlay`，
  通过 `use super::*` 访问父模块私有字段/方法（`can_submit`/`blocker`/`selected_harness_label`/
  `choose_folder`/`edit_prompt` 等）与私有 free function（`ui_agent_kind`/`composer_text_height`）。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-app --offline` | ✅ 通过 |
| `cargo clippy -p homie-app --all-targets --offline` | ✅ 0 警告 |
| `cargo test -p homie-app --offline launcher` | ✅ 1/1 passed |
| `cargo test -p homie-app --offline` | ✅ 301 passed（沙箱内 2 个 `daemon_launch` socket bind EPERM 失败属预期） |
| `cargo test -p homie-app --offline daemon_launch::tests`（沙箱外） | ✅ 8/8 passed |
| 引用方零改动 | ✅ `git status` 仅改 `launcher/` 目录 + 文档 |

## 沙箱测试说明

沙箱内 `cargo test -p homie-app` 中 2 个 `daemon_launch` 测试因 fixture daemon 的 socket bind
返回 `PermissionDenied`（EPERM）而失败，属沙箱网络限制预期。沙箱外复测 `daemon_launch::tests`
8/8 全部通过，确认拆分未影响 daemon 启动逻辑。

## 已知限制 / 延期

- 无。`store/mod.rs`（2,434 行）为后续最重拆分目标，建议留到最后。
