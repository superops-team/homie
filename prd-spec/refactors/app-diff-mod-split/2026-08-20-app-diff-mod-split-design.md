# PRD — app-diff-mod-split

## 背景

`homie/crates/homie-app/src/diff.rs`（955 行）是工作区 diff 加载与 unified patch 解析的 God Module：
一个文件同时承载数据类型（`DiffLayer`/`DiffRowKind`/`DiffRow`/`DiffFile`/`DiffHunk`/`DiffSnapshot`/
`DiffError`）、git 进程加载与比较逻辑（`discover_repository`/`load_diff_from_repository`/
`append_*`/`resolve_comparison`/`git*`）、patch 状态机解析（`parse_unified_diff_bytes`/`finish_*`/
`fnv1a64`/`parse_hunk_start`）、公开接口（`load_worktree_diff_against`/`load_local_diff`/
`parse_unified_diff`/`snapshot_from_read_diff`）、以及测试，单文件超过 800 行阈值，阅读与变更成本高，
违背仓库「组件模块化、关注点清晰」原则。

## 目标

将 `diff.rs` 机械拆分为目录化聚焦子模块，公共 API 与运行时行为零变更，引用方零改动，
单文件 < 800 行。

## 非目标

- 不改变任何 diff 加载、git 命令、patch 解析、行号/fingerprint 计算行为。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动 `DiffLayer`/`DiffRowKind`/`DiffRow`/`DiffFile`/`DiffHunk`/`DiffSnapshot`/`DiffError`
  类型与 `load_worktree_diff_against`/`load_local_diff`/`parse_unified_diff`/
  `snapshot_from_read_diff` 等公共函数签名。

## 用户场景

1. 开发者定位「git 加载与比较逻辑」时，直接进入 `load.rs`。
2. 开发者定位「patch 状态机解析与 fingerprint」时，聚焦在 `parse.rs`。
3. 开发者定位「数据类型与公开接口 facade」时，聚焦在 `mod.rs`。
4. 开发者定位「既有测试」时，聚焦在 `tests.rs`。

## 模块划分方案

```text
diff/
├── mod.rs     facade：doc + 类型定义 + Display/Error + 公开接口（load_*/parse_unified_diff/
│              snapshot_from_read_diff）+ LocalDiffSource + 模块声明
├── load.rs    git 加载/比较：discover_repository/load_diff_from_repository/append_*  +
│              resolve_comparison/resolve_default_branch_ref/git/git_command/append_output/
│              append_bytes/git_failure
├── parse.rs   patch 状态机：parse_unified_diff_bytes/trim_patch_line/push_row/finish_hunk/
│              finish_file/fnv1a64/diff_path/parse_hunk_start/range_start
└── tests.rs   既有测试（原样下沉，#[cfg(test)]）
```

职责边界：
- `mod.rs` 保留：`DiffLayer`/`DiffRowKind`/`DiffRow`/`DiffFile`/`DiffHunk`/`DiffSnapshot`/
  `DiffError` 类型 + `fmt::Display`/`Error` impl；`MAX_DIFF_BYTES`/`MAX_UNTRACKED_FILES` 常量；
  `LocalDiffSource` 枚举；`load_worktree_diff_against`/`load_local_diff`/`parse_unified_diff`/
  `snapshot_from_read_diff` 公开接口。
- `load.rs` 下沉：`discover_repository`/`load_diff_from_repository`/`append_cached_diff`/
  `append_working_diff`/`append_untracked_diffs` 与 `ComparisonResolution`/`resolve_comparison`/
  `resolve_default_branch_ref`/`git`/`git_command`/`append_output`/`append_bytes`/`git_failure`。
- `parse.rs` 下沉：`parse_unified_diff_bytes`/`trim_patch_line`/`push_row`/`finish_hunk`/
  `finish_file`/`fnv1a64`/`diff_path`/`parse_hunk_start`/`range_start`。

## 可见性设计

- 公共 API（`DiffSnapshot` 等类型与 `load_worktree_diff_against`/`load_local_diff`/
  `parse_unified_diff`/`snapshot_from_read_diff`）路径不变，仍通过 `crate::diff::{...}` 可达。
- 跨子模块互调采用 `pub(super)`（仅对父模块 `diff` 可见，不泄漏到 crate 外）：
  - `mod.rs` → `load.rs`：`discover_repository`/`load_diff_from_repository`。
  - `mod.rs` → `parse.rs`：`parse_unified_diff_bytes`。
  - `load.rs` → `parse.rs`：`parse_unified_diff_bytes`。
  - `mod.rs` → `load.rs`/`parse.rs` 数据传递：`LocalDiffSource`、`MAX_DIFF_BYTES`、
    `MAX_UNTRACKED_FILES` 标记 `pub(super)`。
  - `tests.rs` → `load.rs`/`parse.rs`：`git_command`/`fnv1a64`/`parse_hunk_start` 标记 `pub(super)`。
- 无 `pub` 可见性泄漏到 crate 外，所有跨模块符号均为 `pub(super)` 或保持私有。

## 影响面

- 引用方：仅依赖 `crate::diff::{DiffLayer, DiffSnapshot, DiffHunk, load_local_diff}` 等公共 API，
  零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo check -p homie-app --offline` 全绿。
- `cargo fmt --all --check` 通过。
- `cargo clippy -p homie-app --all-targets --offline` 0 警告。
- `cargo test -p homie-app --offline diff` 全绿（9/9）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件删除。
- C2：公共 API 不变，引用方零改动（`git status` 确认仅改 `diff/` 目录）。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：无 `pub` 可见性泄漏到 crate 外（跨模块符号均为 `pub(super)` 或私有）。
- C6：release readiness 证据写入 `docs/verification/app-diff-mod-split/`。

## Beads

- `homie-hhv`
