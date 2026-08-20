# OpenSpec Plan — engine-git-module-split

## 目标

将 `homie/crates/homie-engine/src/git.rs`（819 行）按关注点拆分为 4 个聚焦子模块：`repo.rs`
（branch/repository 读取）、`worktree.rs`（worktree 管理 + branch 命名）、`diff.rs`（working-diff
探测）。`mod.rs` 保留模块文档 + 子模块声明 + `pub use` 再导出 + 共享 `run` 辅助 + 测试模块。
所有函数逐字迁移，公共 API 不变，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（facade，`pub use` 再导出 + 共享 `run`）→ 各子模块（显式 `use` 引入依赖）。
- `repo.rs` 提供 `is_repository`，被 `worktree.rs` 的 `worktree_overview` 调用；`run` 被
  `repo.rs` 与 `worktree.rs` 共享，保留在 `mod.rs` 为 `pub(crate) fn`。
- 公共函数保持 `pub` 并经 `pub use` 再导出；私有辅助仅在跨模块/测试需要时提升为 `pub(crate)`。
- 测试模块（13 个用例）逐字迁移至 `mod.rs`，`parse_working_diff`/`WORKING_DIFF_SCRIPT`/
  `ADJECTIVES`/`NOUNS` 以 `#[cfg(test)]` 限定再导出。
- 无生产代码语义变更，无外部 API 泄漏。

## 交付切片

- T1：函数/常量边界扫描，定位 15 函数 + 5 常量的闭合边界。
- T2：生成 `repo.rs`/`worktree.rs`/`diff.rs` 子模块。
- T3：重建 `mod.rs`（文档 + 声明 + 再导出 + `run` + 测试），删除旧 `git.rs`，编译验证。
- T4：全量验证（fmt/check/clippy/build/workspace-check/test）。
- T5：code review + release readiness 证据。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/engine-git-module-split/`。
