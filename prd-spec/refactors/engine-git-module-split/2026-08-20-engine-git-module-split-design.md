# PRD — engine-git-module-split

## 背景

`homie/crates/homie-engine/src/git.rs`（819 行）是 Git 相关逻辑的单文件模块，同时承载三类关注点：
仓库/branch 读取（`branch`/`is_linked_worktree`/`git_dir`/`is_repository`/`repository_root`）、
worktree 操作（`list_worktrees`/`parse_porcelain`/`create_worktree`/`remove_worktree`/
`worktree_overview`/branch 命名）、以及 working-diff 探测（`working_diff`/`working_diff_remote`）。
单文件超过 800 行阈值，且三类关注点彼此独立，阅读与变更成本高，违背仓库「组件模块化、关注点清晰」
原则。

## 目标

将 `git.rs` 按关注点拆分为目录化聚焦子模块，公共 API 与运行时行为零变更，引用方零改动，单文件
< 800 行。

## 非目标

- 不改变任何 Git 操作的运行时行为。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动任何函数签名语义。
- 不合并或重命名函数。

## 用户场景

1. 开发者定位「branch / repository 读取」时，聚焦在 `repo.rs`。
2. 开发者定位「worktree 管理」时，聚焦在 `worktree.rs`。
3. 开发者定位「working diff 探测」时，聚焦在 `diff.rs`。

## 模块划分方案

```text
git/
├── mod.rs       facade：模块文档 + 子模块声明 + `pub use` 再导出 + 共享 `run` 辅助 + 测试
├── repo.rs      branch / is_linked_worktree / git_dir / is_repository / repository_root
├── worktree.rs  WorktreeInfo / list_worktrees / parse_porcelain / generated_branch_name /
│                branch_to_path_slug / create_worktree / remove_worktree / worktree_overview
│                （ADJECTIVES / NOUNS 常量）
└── diff.rs      working_diff / working_diff_remote / working_diff_input / parse_working_diff /
                 diff_failure（WORKING_DIFF_SCRIPT 等常量）
```

## 可见性设计

- 所有原 `pub fn` 保持 `pub`，经 `mod.rs` 的 `pub use` 再导出，公共 API 不变。
- `run` 原为私有 `fn`，因被 `repo.rs` 与 `worktree.rs` 跨模块调用，提升为 `pub(crate) fn`，保留在
  `mod.rs` 作为共享辅助。
- `git_dir` 仅在 `repo.rs` 内部使用，保持私有 `fn`。
- `working_diff_input`、`diff_failure` 仅在 `diff.rs` 内部使用，保持私有 `fn`。
- `parse_working_diff`、`WORKING_DIFF_SCRIPT`、`ADJECTIVES`、`NOUNS` 因被测试模块引用，提升为
  `pub(crate)`，并在 `mod.rs` 以 `#[cfg(test)]` 限定再导出（仅测试可见，不泄漏到生产编译）。
- 各子模块以显式 `use` 引入所需依赖。

## 影响面

- 仅 `git.rs` 的函数/常量拆分为 4 个聚焦子模块，生产代码与其它模块零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo fmt --all --check` 通过。
- `cargo check -p homie-engine --offline` 全绿。
- `cargo clippy -p homie-engine --all-targets --offline` 0 警告。
- `cargo build -p homie-engine --offline` 通过。
- `cargo check --workspace --offline` 全绿。
- `cargo test -p homie-engine --offline` 全绿（303 passed / 0 failed / 3 ignored）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件（819 行）拆为 4 子模块 + facade。
- C2：公共 API 不变，引用方零改动。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：函数逐字迁移，公共可见性不变，私有辅助仅内部提升为 `pub(crate)`。
- C6：release readiness 证据写入 `docs/verification/engine-git-module-split/`。

## Beads

- `homie-9ql`
