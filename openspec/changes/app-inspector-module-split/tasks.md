# App Inspector Module Split — Tasks

change_id: `app-inspector-module-split` · Beads: `homie-ubu.3`

## T1 — 建立拆分基线与验证骨架

- 确认 `inspector.rs`（4,692 行）现状，锁定现有 10 个测试 fixture 全绿。
- 创建 `openspec/changes/app-inspector-module-split/{plan,tasks,alignment-report}.md`。
- 记录基线：`cargo test -p homie-app` 全绿、`cargo fmt --check` clean。

验收：基线测试全绿，OpenSpec 文档齐全。

## T2 — S1 下沉纯状态类型到 `state.rs`

- 新建 `inspector/state.rs`，搬迁：
  - `impl InspectorTab`（`ALL`/`label`/`index`/`debug_selector`）
  - `DiffContext`、`LoadState`、`ReviewLoadState`、`ReviewAction`、`AskDraft`
- `inspector.rs` 改为 `inspector/mod.rs` 门面，`mod state; pub use state::*;`。
- 函数体一字不改，仅调整 `use` 路径。

验收：`cargo test -p homie-app` 全绿、`cargo check -p homie-app` 0 warning、`cargo fmt --check` clean。

## T3 — S2 下沉纯投影函数到 `projection.rs`

- 搬迁 21 个纯函数：`sorted_pr_checks`、`checks_rollup`、`discussion_state`、
  `pull_request_can_merge`、`merge_blocker_label`、`humanize_github_state`、`artifact_count`、
  `artifact_visible`、`ui_agent_kind`、`session_status`、`pull_request_state`、
  `pull_request_discussion`、`artifact_title`、`pr_number`、`linear_key`、`url_authority`、
  `folder_name`、`format_bytes`、`relative_time`、`prompt_layer`、`patch_creates_file`。
- 函数体一字不改，调整 `use` 路径。

验收：`cargo test -p homie-app` 全绿、`cargo check` 0 warning、`cargo fmt --check` clean。

## T4 — S3 下沉 review/diff/merge 策略到 `policy.rs`

- 识别 review action 策略、快捷键决策等纯策略函数，下沉到 `policy.rs`。
- 函数体一字不改。

验收：`cargo test -p homie-app` 全绿、`cargo check` 0 warning、`cargo fmt --check` clean。

## T5 — S4 收尾，render 留在 `view.rs`

- `view.rs` 承接全部 GPUI render + click handler，< 800 行。
- `mod.rs` 仅剩门面重导出。
- 删除遗留的兼容层与死代码。

验收：`inspector/view.rs` < 800 行；`cargo test -p homie-app` 全绿；视觉/行为不变。

## T6 — 发布就绪与收尾

- 记录证据到 `docs/verification/app-inspector-module-split/`（含 release-readiness-report）。
- 打 annotated tag（预计 `v0.1.10`），push commits + tags。
- 关闭 Beads `homie-ubu.3`。

验收：证据齐全、tag 已打、bead 已关闭。
