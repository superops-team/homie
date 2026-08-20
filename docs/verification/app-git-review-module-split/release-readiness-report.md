# 发布就绪报告 — app-git-review-module-split

## 变更概述

将 `homie/crates/homie-app/src/git_review.rs`（1,427 行，原生 Git 审查 God Module）
机械拆分为目录化聚焦子模块：子进程执行、porcelain-v2 状态解析、路径/补丁辅助、测试各自下沉，
`mod.rs` 收尾为 facade。公共 API 与运行时行为完全不变，`inspector/mod.rs` 与
`inspector/state.rs` 零改动，单文件 < 800 行。

- change_id：`app-git-review-module-split`
- Beads：`homie-42t`
- 类型：task（机械拆分，行为不变）
- 上游：`architecture-audit-governance-2026-08`（模块降熵序列延续）

## 模块划分

```text
git_review/
├── mod.rs      facade（公共类型/常量 + GitReviewError + impl GitRepository + mod 声明），499 行
├── process.rs  GitOutput/ensure_success/run_git/read_bounded/join_reader + 子进程常量，190 行
├── status.rs   porcelain-v2 解析 + MAX_STATUS_ENTRIES，228 行
├── paths.rs    路径/补丁辅助，111 行
└── tests.rs    既有测试（原样下沉，#[cfg(test)]），436 行
```

依赖方向：公共类型/常量全部在 `mod.rs` 定义，子模块 `process`/`status`/`paths` 读取 facade
类型，`paths.rs` 额外引用 `process::GitOutput`；`mod.rs` 反向调用三个子模块的 `pub(super)` 函数。
`inspector/mod.rs` 与 `inspector/state.rs` 仅依赖 `crate::git_review::{...}` 公共 API，路径不变。

## 交付切片 T1–T5

| 切片 | 内容 | 状态 |
|------|------|------|
| T1 | 抽取子进程执行 process.rs | 完成 |
| T2 | 抽取状态解析 status.rs | 完成 |
| T3 | 抽取路径/补丁辅助 paths.rs | 完成 |
| T4 | 抽取测试 tests.rs + mod.rs 收尾 facade | 完成 |
| T5 | 全量验证 + code review + release readiness | 完成 |

## 验证证据

| 验证项 | 命令 | 结果 |
|--------|------|------|
| 编译检查 | `cargo check -p homie-app --offline` | 通过，0 警告 |
| 格式检查 | `cargo fmt --all --check` | 通过 |
| 静态检查 | `cargo clippy -p homie-app --all-targets --offline` | 通过，0 警告 |
| 全量测试 | `cargo test -p homie-app --offline` | **301 passed / 2 failed / 1 ignored**（2 failed 为沙箱 socket bind EPERM 环境限制，非本变更引入） |
| 沙箱外复测 | `cargo test -p homie-app --offline daemon_launch::tests` | 8 passed / 0 failed（确认 2 个失败纯属沙箱限制） |
| 单文件行数 | `wc -l` | mod 499 / process 190 / status 228 / paths 111 / tests 436，均 < 800 |

- `git_review` 相关 13 个测试（`cargo test -p homie-app --offline git_review`）全部原样通过，
  状态解析 / 路径校验 / hunk 补丁 / 提交校验行为与拆分前等价。
- 公共 API 兼容性：`GitRepository`、`ReviewStatus`、`BranchInfo`、`FileChange`、`ChangeKind`、
  `PatchMutation`、`CommitResult`、`GitReviewError` 路径不变；`cargo check -p homie-app` 编译通过
  证明可达性不变。
- `inspector/mod.rs` 与 `inspector/state.rs` 零改动（`git status` 确认仅删除旧单文件、新增目录）。
- 可见性管控：仅跨模块调用函数（`GitOutput` 及 `status`/`stdout`/`stderr_message`/`failure`、
  `ensure_success`/`run_git`、`parse_status`/`path_from_output_line`/`trim_line_ending`、
  `patch_creates_file`/`patch_rejected`/`literal_path_command`/`validate_paths`）升为 `pub(super)`，
  无 `pub` 泄漏到 crate 外。

## 已知限制与后续

- 无已知限制。
- 机械重构未改变任何运行时行为、公共 API 不变；纯职责搬迁，不做向后兼容。
- 后续候选（`store/mod.rs` 2,434 行、`terminal_pane/mod.rs` 1,486 行、`navigation.rs` 1,376 行）
  依赖密、风险更高，建议作为更谨慎的独立切片处理。

## 结论

所有验收标准（C1–C6）均已满足，验证证据齐备，可发布。
