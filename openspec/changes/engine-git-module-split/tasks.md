# OpenSpec Tasks — engine-git-module-split

## T1 函数/常量边界扫描

- [x] 定位 15 个函数（branch/is_linked_worktree/git_dir/is_repository/repository_root/
  list_worktrees/parse_porcelain/run/generated_branch_name/branch_to_path_slug/create_worktree/
  remove_worktree/working_diff/working_diff_remote/worktree_overview + 私有 working_diff_input/
  parse_working_diff/diff_failure）+ 5 常量边界，跳过 doc 注释与多行签名。
- 验收：全部解析，无缺失/重复。关联 C2/C5。

## T2 生成子模块

- [x] `repo.rs`（5 函数）、`worktree.rs`（WorktreeInfo + 8 函数 + ADJECTIVES/NOUNS）、
  `diff.rs`（2 pub 函数 + 3 私有辅助 + 5 常量）。
- 验收：函数体逐字迁移，`pub` 保持 `pub`。关联 C1/C5。

## T3 重建 mod.rs + 编译验证

- [x] 移除旧 `git.rs`，新增 `git/mod.rs`（文档 + 声明 + `pub use` 再导出 + `run` + 测试）。
- [x] `cargo check -p homie-engine --offline` 全绿（0 警告）。
- 验收：通过。关联 C2/C4。

## T4 全量验证

- [x] `cargo fmt --all --check`
- [x] `cargo check -p homie-engine --offline`
- [x] `cargo clippy -p homie-engine --all-targets --offline`
- [x] `cargo build -p homie-engine --offline`
- [x] `cargo check --workspace --offline`
- [x] `cargo test -p homie-engine --offline`（303 passed / 0 failed / 3 ignored）
- 验收：全部通过。关联 C3/C4。

## T5 code review + release readiness

- [x] code review：拆分边界清晰、函数逐字迁移、无行为变更、可见性正确、单文件 < 800 行。
- [x] release readiness 证据写入 `docs/verification/engine-git-module-split/`。
- 验收：通过。关联 C6。
