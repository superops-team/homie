# OpenSpec Plan — app-git-review-module-split

## 概述

将 `homie/crates/homie-app/src/git_review.rs`（1,427 行）机械拆分为目录化聚焦子模块：
子进程执行、状态解析、路径/补丁辅助、测试各自下沉，`mod.rs` 收尾为 facade。
公共 API 与运行时行为完全不变，`inspector` 两个引用方零改动，单文件 < 800 行。

## 模块划分与依赖

```text
git_review/
├── mod.rs      facade（公共常量/类型 + GitReviewError + impl GitRepository + mod 声明），< 800
├── process.rs  GitOutput/ensure_success/run_git/read_bounded/join_reader + 子进程常量，< 800
├── status.rs   porcelain-v2 解析 + MAX_STATUS_ENTRIES，< 800
├── paths.rs    路径/补丁辅助，< 800
└── tests.rs    既有测试（原样下沉，#[cfg(test)]），< 800
```

依赖方向：

- `process.rs`（最底层）：`GitOutput`/`ensure_success`/`run_git`
- `paths.rs → process`（`patch_rejected` 接收 `GitOutput`）
- `status.rs → mod`（公共类型 `ReviewStatus`/`BranchInfo`/`FileChange`/`ChangeKind`/`GitReviewError`）
- `mod → process`（`run_git`/`ensure_success`）、`mod → status`（`parse_status`/`path_from_output_line`/
  `trim_line_ending`）、`mod → paths`（`validate_paths`/`literal_path_command`/`patch_creates_file`/
  `patch_rejected`）
- `tests → mod` + `tests → status`（`parse_status`）+ `tests → crate::diff`

公共类型/常量全部在 `mod.rs` 定义，子模块通过 `use super::*;` 访问；子模块私有类型/函数
必要时以 `pub(super)` 暴露给兄弟模块或 `mod.rs`。

## 任务清单

| Task | 描述 | 验收 | 关联验证 Case |
|------|------|------|---------------|
| T1 | 抽取 process.rs（子进程执行 + GitOutput） | cargo check 全绿 | C1 |
| T2 | 抽取 status.rs（porcelain-v2 解析） | cargo check 全绿 | C2 |
| T3 | 抽取 paths.rs（路径/补丁辅助） | cargo check 全绿 | C3 |
| T4 | 抽取 tests.rs + mod.rs 收尾为 facade | 单文件 < 800 | C4 |
| T5 | 全量验证 + code review + release readiness | fmt/check/clippy/test 全绿 | C5/C6 |

## 验证口径

- `cargo fmt --check`
- `cargo check -p homie-app`
- `cargo clippy -p homie-app --all-targets`
- `cargo test -p homie-app`（git_review 相关 13 个测试原样通过，行为等价）
- `mod.rs` / `process.rs` / `status.rs` / `paths.rs` / `tests.rs` 均 < 800 行
- `inspector/mod.rs` 与 `inspector/state.rs` 零改动
