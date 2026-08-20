# OpenSpec Plan — app-diff-mod-split

## 目标

将 `homie/crates/homie-app/src/diff.rs`（955 行）机械拆分为目录化聚焦子模块：
`load.rs`（git 加载/比较）、`parse.rs`（patch 状态机）、`tests.rs`，`mod.rs` 保留 facade
（类型定义 + Display/Error + 公开接口 + LocalDiffSource）。公共 API 与运行时行为零变更，
引用方零改动，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（父）→ `load.rs`/`parse.rs`（子）；`load.rs` → `parse.rs`（parser 复用）；
  `tests.rs` → `mod.rs`/`load.rs`/`parse.rs`。
- 跨子模块互调采用 `pub(super)`：`discover_repository`/`load_diff_from_repository`/
  `parse_unified_diff_bytes`/`git_command`/`fnv1a64`/`parse_hunk_start`，以及 `LocalDiffSource`/
  `MAX_DIFF_BYTES`/`MAX_UNTRACKED_FILES`。全部仅对父模块 `diff` 可见，无 `pub` 泄漏。
- git 加载与比较逻辑下沉 `load.rs`；patch 状态机解析下沉 `parse.rs`；既有测试原样下沉 `tests.rs`。

## 交付切片

- T1：抽取 git 加载/比较逻辑 → `load.rs`。
- T2：抽取 patch 状态机解析 → `parse.rs`。
- T3：测试原样下沉 → `tests.rs`。
- T4：全量验证 + code review + release readiness。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/app-diff-mod-split/`。
