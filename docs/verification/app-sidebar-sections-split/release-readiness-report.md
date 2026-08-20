# Release Readiness — app-sidebar-sections-split

## 变更摘要

将 `homie/crates/homie-app/src/sidebar/sections.rs`（1,202 行）机械拆分为 5 个聚焦子模块 + facade
（`mod.rs`）。10 个 `pub(crate)` 方法逐字迁移，`pub(crate)` 可见性保持不变，生产代码语义零变更。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 7 | facade：`use super::*;` + 模块声明 |
| `chrome.rs` | 170 | 顶栏与空状态：new_agent_row/top_bar/empty_state |
| `project.rs` | 234 | 项目区块：project_section |
| `session.rs` | 377 | 会话行与折叠：session_row/disclosure |
| `archive.rs` | 258 | 归档：archived_bucket/archived_row |
| `footer.rs` | 164 | 页脚：update_pill/account_footer |

全部单文件 < 800 行（最大 `session.rs` 377 行）。

## 方法逐字迁移

- 10 个 `pub(crate) fn` 方法全部逐字迁移，方法体、渲染逻辑、交互行为零变更。
- `pub(crate)` 可见性保持不变，crate 内调用方经 `Sidebar` 类型访问，路径零改动。
- 各子模块以 `use super::*;` 引入 `Sidebar` 与渲染依赖，`impl Sidebar` 跨子模块实现。
- 无生产代码语义变更，无 `pub` 可见性泄漏。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-app --offline` | ✅ 通过 |
| `cargo clippy -p homie-app --all-targets --offline` | ✅ 0 警告 |
| `cargo build -p homie-app --offline` | ✅ 通过 |
| `cargo test -p homie-app --offline` | ✅ 301 passed / 2 failed（沙箱内 2 个 `daemon_launch` socket bind EPERM 失败属预期） |
| 引用方零改动 | ✅ `git status` 仅改 `sections.rs` → `sections/` 目录 + 文档 |

## 沙箱测试说明

沙箱内 `cargo test -p homie-app` 中 2 个 `daemon_launch` 测试因 fixture daemon 的 socket bind
返回 `PermissionDenied`（EPERM）而失败，属沙箱网络限制预期，与本次拆分无关。

## 已知限制 / 延期

- 无。后续候选（按行数排序）：`root/mod.rs`（1,190 行）、`terminal_pane/view.rs`（863 行）等。
