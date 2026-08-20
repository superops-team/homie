# Release Readiness — app-diff-mod-split

## 变更摘要

将 `homie/crates/homie-app/src/diff.rs`（955 行）机械拆分为 3 个聚焦子模块，
`mod.rs` 收尾 facade（168 行）。公共 API 与运行时行为零变更，引用方零改动。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 168 | doc + 类型定义 + Display/Error + 公开接口（load_*/parse_unified_diff/snapshot_from_read_diff）+ LocalDiffSource + 模块声明 |
| `load.rs` | 311 | git 加载/比较：discover_repository/load_diff_from_repository/append_* + resolve_comparison/git*/append_* |
| `parse.rs` | 213 | patch 状态机：parse_unified_diff_bytes/finish_*/fnv1a64/diff_path/parse_hunk_start/range_start |
| `tests.rs` | 279 | 既有测试（原样下沉） |

全部单文件 < 800 行。

## 可见性管控

跨模块符号均采用 `pub(super)`（仅对父模块 `diff` 可见），无 `pub` 可见性泄漏到 crate 外：
- `pub(super) const MAX_DIFF_BYTES / MAX_UNTRACKED_FILES`、`pub(super) enum LocalDiffSource`（mod.rs）。
- `pub(super) fn discover_repository / load_diff_from_repository / git_command`（load.rs）。
- `pub(super) fn parse_unified_diff_bytes / fnv1a64 / parse_hunk_start`（parse.rs）。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-app --offline` | ✅ 通过 |
| `cargo clippy -p homie-app --all-targets --offline` | ✅ 0 警告 |
| `cargo test -p homie-app --offline diff` | ✅ 9/9 passed |
| `cargo test -p homie-app --offline` | ✅ 301 passed（沙箱内 2 个 `daemon_launch` socket bind EPERM 失败属预期） |
| `cargo test -p homie-app --offline daemon_launch::tests`（沙箱外） | ✅ 8/8 passed |
| 引用方零改动 | ✅ `git status` 仅改 `diff/` 目录 + 文档 |

## 沙箱测试说明

沙箱内 `cargo test -p homie-app` 中 2 个 `daemon_launch` 测试因 fixture daemon 的 socket bind
返回 `PermissionDenied`（EPERM）而失败，属沙箱网络限制预期。沙箱外复测 `daemon_launch::tests`
8/8 全部通过，确认拆分未影响 daemon 启动逻辑。

## 已知限制 / 延期

- 无。后续候选（按行数排序）：`store/mod.rs`（2,434 行）、`store/tests.rs`（1,658 行）、
  `sidebar/popover.rs`（1,249 行）、`sidebar/sections.rs`（1,202 行）、`root/mod.rs`（1,190 行）、
  `macos/menu_bar.rs`（962 行）等。
