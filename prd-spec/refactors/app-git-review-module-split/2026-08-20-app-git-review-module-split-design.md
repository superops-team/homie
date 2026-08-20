# git_review.rs 模块拆分设计文档

## 1. 背景

2026-08 架构审计（`architecture-audit-governance-2026-08`）的模块降熵序列已先后拆分
`surface_shell/view.rs`、`terminal_pane/mod.rs`、`sidebar`、`inspector`、`session_surfaces.rs`、
`code_intelligence.rs` 等 God Module。当前 homie-app 前几大单文件：

| 文件 | 行数 |
|------|------|
| `store/mod.rs` | 2,434 |
| `terminal_pane/mod.rs` | 1,486 |
| `git_review.rs` | 1,427 |
| `navigation.rs` | 1,376 |

`git_review.rs`（1,427 行）是「原生 Git review 操作」模块，为 review cockpit 提供仓库发现、
结构化状态读取、路径/补丁级变更操作，内部混装：

1. 公共数据结构（`GitRepository`/`ReviewStatus`/`BranchInfo`/`FileChange`/`ChangeKind`/
   `PatchMutation`/`CommitResult`，约 33–97 行）；
2. 错误类型（`GitReviewError` + Display/Error，约 103–223 行）；
3. `impl GitRepository`（发现/状态/暂存/取消暂存/丢弃/补丁/提交，约 224–500 行）；
4. 路径/补丁辅助（`patch_creates_file`/`patch_rejected`/`literal_path_command`/`validate_paths`/
   `invalid_path`/`os_str_eq_ignore_ascii_case`/`os_str_contains_nul`，约 501–597 行）；
5. porcelain-v2 状态解析（`parse_status`/`parse_branch_header`/`parse_prefixed_count`/
   `add_tracked_change`/`is_unmerged`/`change_kind`/`split_fields`/`require_fields`/
   `path_from_bytes`/`path_from_output_line`/`trim_line_ending`，约 598–815 行）；
6. Git 子进程执行（`GitOutput`/`ensure_success`/`run_git`/`read_bounded`/`join_reader`，
   约 817–990 行）；
7. 内联测试（`mod tests`，约 991–1427 行，13 个测试）。

引用方仅 `inspector/mod.rs` 与 `inspector/state.rs`，只依赖公共 API（`GitRepository`、
`GitReviewError`、`PatchMutation`、`ReviewStatus`）。拆分边界清晰、依赖单一，是下一个
机械拆分的理想切片。

## 2. 目标

- 把 `git_review.rs`（1,427 行）拆为目录化聚焦子模块，**单文件 < 800 行**。
- 路径/补丁辅助、状态解析、子进程执行、测试各自下沉到独立子模块，`mod.rs` 收尾为 facade。
- 公共 API 与运行时行为完全不变；`inspector` 两个引用方零改动。

## 3. 非目标

- 不重设计任何 Git 操作/解析/子进程硬化逻辑，不改运行语义。
- 不改公共类型路径（`crate::git_review::{...}` 保持可用）。
- 不合并/删除任何既有方法；纯职责搬迁。
- 不触及任何 `specs/` 合同（本次不涉及长生命周期组件接口变更）。

## 4. 需求

### FR-1: 子进程执行下沉

`GitOutput` + `ensure_success` / `run_git` / `read_bounded` / `join_reader` 及常量
`GIT_TIMEOUT` / `POLL_INTERVAL` / `MAX_STDOUT_BYTES` / `MAX_STDERR_BYTES` 下沉到
`git_review/process.rs`。

### FR-2: 状态解析下沉

`parse_status` / `parse_branch_header` / `parse_prefixed_count` / `add_tracked_change` /
`is_unmerged` / `change_kind` / `split_fields` / `require_fields` / `path_from_bytes` /
`path_from_output_line` / `trim_line_ending` 及常量 `MAX_STATUS_ENTRIES` 下沉到
`git_review/status.rs`。

### FR-3: 路径/补丁辅助下沉

`patch_creates_file` / `patch_rejected` / `literal_path_command` / `validate_paths` /
`invalid_path` / `os_str_eq_ignore_ascii_case` / `os_str_contains_nul` 下沉到
`git_review/paths.rs`。

### FR-4: 测试下沉

内联 `mod tests` 下沉到 `git_review/tests.rs`，`#[cfg(test)]` 保持；测试中对
`parse_status` 的直接引用改为 `use super::status::parse_status;`。

### FR-5: mod.rs 收尾为 facade

`git_review/mod.rs` 保留公共常量/类型 + `GitReviewError` + `impl GitRepository` + 子模块声明，
行数 < 800。

### FR-6: 行为不变

拆分后 `cargo check -p homie-app`、`cargo test -p homie-app`、`cargo fmt --check` 全绿，
Git 操作/解析/子进程行为与拆分前等价；`inspector` 两个引用方零改动。

## 5. 涉及文件

- `homie/crates/homie-app/src/git_review.rs`（拆分源，转为目录）
- `homie/crates/homie-app/src/git_review/mod.rs`（新增，facade）
- `homie/crates/homie-app/src/git_review/process.rs`（新增，子进程执行）
- `homie/crates/homie-app/src/git_review/status.rs`（新增，状态解析）
- `homie/crates/homie-app/src/git_review/paths.rs`（新增，路径/补丁辅助）
- `homie/crates/homie-app/src/git_review/tests.rs`（新增，测试）

## 6. 验证计划

```bash
cargo fmt --check
cargo check -p homie-app
cargo clippy -p homie-app --all-targets
cargo test -p homie-app
```

人工验收：

1. Git 操作/解析/子进程行为与拆分前等价（既有 13 个测试原样通过）。
2. 每个新子模块与 `mod.rs` 均 < 800 行。
3. `inspector/mod.rs` 与 `inspector/state.rs` 零改动，`crate::git_review::{...}` 路径不变。

## 7. Beads

- change_id: `app-git-review-module-split`
- 类型: task（机械拆分，行为不变）
- 优先级: P1（homie-app 组合根降熵）
- 上游: `architecture-audit-governance-2026-08`（模块降熵序列延续）
