# Release Readiness — app-terminal-pane-mod-split

## 变更摘要

将 `homie/crates/homie-app/src/terminal_pane/mod.rs`（1,486 行）机械拆分为 6 个聚焦子模块，
`mod.rs` 收尾 facade（639 行）。公共 API 与运行时行为零变更，引用方零改动。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 639 | 类型/常量/枚举 + 生命周期/驻留协调 + 视图接口 + 状态字形/主题 + 选择 |
| `events.rs` | 187 | 事件分发 + 网格更新 + 重排 + 重绘 |
| `input.rs` | 213 | 键盘输入 |
| `find.rs` | 116 | 查找/搜索 |
| `geometry.rs` | 195 | 网格几何 + 缩放 + 选中几何更新 |
| `clipboard.rs` | 101 | 剪贴板 |
| `scroll.rs` | 92 | 滚动回取 + 滚动 |

全部单文件 < 800 行。

## 可见性管控

仅跨模块调用的方法升为 `pub(super)`，无 `pub` 可见性泄漏到 crate 外：
- `events.rs`：`handle_pane_event`（mod.rs 调用）、`hold_reflow`（geometry.rs 调用）。
- `input.rs`：`handle_key_down`/`handle_key_up`/`handle_modifiers_changed`（view.rs 调用）。
- `find.rs`：`schedule_find`（events/clipboard/input 调用）、`open_find`/`close_find`/
  `close_find_for_selected`/`find_next`/`find_previous`/`navigate_find`（view.rs 调用）。
- `geometry.rs`：`grid_cell_at`/`grid_row_overflow`/`zoom_in`/`zoom_out`/`reset_zoom`/
  `update_selected_geometry`（view.rs 调用）。
- `clipboard.rs`：`copy_selection`/`paste`（view.rs 调用）。
- `scroll.rs`：`pump_scrollback_fetch`（events.rs 调用）、`handle_scroll`（view.rs 调用）。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-app --offline` | ✅ 通过 |
| `cargo clippy -p homie-app --all-targets --offline` | ✅ 0 警告 |
| `cargo test -p homie-app --offline` | ✅ 301 passed（沙箱内 2 个 `daemon_launch` socket bind EPERM 失败属预期） |
| `cargo test -p homie-app --offline daemon_launch::tests`（沙箱外） | ✅ 8/8 passed |
| 引用方零改动 | ✅ `git status` 仅改 `terminal_pane/` 目录 + 文档 |

## 沙箱测试说明

沙箱内 `cargo test -p homie-app` 中 2 个 `daemon_launch` 测试因 fixture daemon 的 socket bind
返回 `PermissionDenied`（EPERM）而失败，属沙箱网络限制预期。沙箱外复测 `daemon_launch::tests`
8/8 全部通过，确认拆分未影响 daemon 启动逻辑。

## 已知限制 / 延期

- 无。`store/mod.rs`（2,434 行）为后续最重拆分目标，建议留到最后。
