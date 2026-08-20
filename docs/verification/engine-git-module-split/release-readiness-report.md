# Release Readiness — engine-git-module-split

## 变更摘要

将 `homie/crates/homie-engine/src/git.rs`（819 行）按关注点拆分为 4 个聚焦子模块。15 个函数 +
5 个常量逐字迁移，公共 `pub fn` 保持 `pub` 并经 `mod.rs` 的 `pub use` 再导出（公共 API 不变）。
`run` 因跨模块共享提升为 `pub(crate) fn`；`parse_working_diff`/`WORKING_DIFF_SCRIPT`/
`ADJECTIVES`/`NOUNS` 因被测试引用以 `#[cfg(test)]` 限定再导出。测试模块（13 用例）逐字迁移。
生产代码语义零变更。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 302 | facade：文档 + 子模块声明 + `pub use` 再导出 + `run` + 测试模块 |
| `repo.rs` | 75 | branch / is_linked_worktree / git_dir / is_repository / repository_root |
| `worktree.rs` | 286 | WorktreeInfo / list_worktrees / parse_porcelain / generated_branch_name / branch_to_path_slug / create_worktree / remove_worktree / worktree_overview |
| `diff.rs` | 184 | working_diff / working_diff_remote / working_diff_input / parse_working_diff / diff_failure |

全部单文件 < 800 行（最大 `mod.rs` 302 行，含测试）。

## 函数逐字迁移

- 15 个函数 + 5 个常量全部逐字迁移，Git 操作语义零变更。
- 公共 `pub fn`（branch/is_linked_worktree/is_repository/repository_root/list_worktrees/
  parse_porcelain/generated_branch_name/branch_to_path_slug/create_worktree/remove_worktree/
  working_diff/working_diff_remote/worktree_overview + `pub struct WorktreeInfo`）保持 `pub`，
  经 `pub use` 再导出。
- `run` 原私有 `fn`，因 `repo.rs`/`worktree.rs` 跨模块共享，提升为 `pub(crate) fn`。
- `git_dir` 私有，仅在 `repo.rs` 内使用；`working_diff_input`/`diff_failure` 私有，仅在
  `diff.rs` 内使用。
- 测试辅助 `parse_working_diff`/`WORKING_DIFF_SCRIPT`/`ADJECTIVES`/`NOUNS` 以 `#[cfg(test)]`
  限定再导出，不泄漏到生产编译。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-engine --offline` | ✅ 通过（0 警告） |
| `cargo clippy -p homie-engine --all-targets --offline` | ✅ 0 警告 |
| `cargo build -p homie-engine --offline` | ✅ 通过 |
| `cargo check --workspace --offline` | ✅ 通过 |
| `cargo test -p homie-engine --offline` | ✅ 303 passed / 0 failed / 3 ignored |
| 引用方零改动 | ✅ 仅 `git.rs` → `git/` 目录 + 文档 |

## 已知限制 / 延期

- 无。后续候选（按行数排序）：`remote/manager.rs`（1099 行）、`mcp/host.rs`（863 行）、
  `pr_monitor.rs`（822 行）等。
