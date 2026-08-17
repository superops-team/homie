# App Inspector Module Split 发布就绪报告

change_id: `app-inspector-module-split` · Beads: `homie-ubu.3`

## 1. 就绪结论

S1（state）、S2（projection）、S3（policy）、S4（render 下沉）四个切片全部就绪。
`inspector.rs` 由约 4,692 行收敛为 `inspector/mod.rs`（712 行）+ 12 个子模块，
审计 finding **F3（Cognitive Overload）** 已解除。可提交并打 tag。

## 2. 交付物

| 模块 | 行数 | 职责 |
|------|------|------|
| `inspector/mod.rs` | 712 | 门面 + 结构体定义 + effect dispatch（非 render 方法） |
| `inspector/view.rs` | 747 | render 门面：header/info/git/artifacts/message + `impl Render` + `section_label`/`detail_row` |
| `inspector/ask.rs` | 314 | ask 预设/输入框 render + 快捷键 `handle_key_down` |
| `inspector/changes.rs` | 506 | diff/comparison/layer/file-navigator/changes render |
| `inspector/review.rs` | 407 | review 控制面板 render |
| `inspector/scrollbar.rs` | 133 | 滚动条度量/拖拽/渲染 |
| `inspector/pr.rs` | 705 | PR 卡片/徽章/检查/讨论项 render（自由函数） |
| `inspector/artifacts.rs` | 72 | artifact 行/图标 render（自由函数） |
| `inspector/diff.rs` | 407 | diff 行虚拟化 render + `DiffRowRenderContext`（自由函数） |
| `inspector/state.rs` | 87 | tab 状态机 + review/ask 工作流状态（纯类型） |
| `inspector/projection.rs` | 323 | 21 个纯投影/文案/格式化函数 |
| `inspector/policy.rs` | 23 | git 错误分类 + 加载门控策略（纯函数） |
| `inspector/tests.rs` | 349 | 原 test 模块随迁 |

## 3. 验证汇总

| 门禁 | 结果 |
|------|------|
| `cargo check -p homie-app` | 0 warning |
| `cargo fmt --check` | clean |
| `cargo test -p homie-app` | 303 passed / 0 failed / 1 ignored |
| `inspector/view.rs` < 800 行 | 通过（747 行） |
| state/projection/policy 无 `Window`/`Context`/`Entity`/render 依赖 | 通过 |
| 函数体逐字搬迁，视觉/行为不变 | 通过 |

## 4. 范围说明

本 change_id 仅做机械拆分，不改任何视觉样式、交互语义、GPUI 组件层级、磁盘 schema 或
store effect 语义。公共 API 不变：`root.rs` 仍 `use crate::inspector::{InspectorEvent, WorkbenchInspector}`。

S4 采用「8 个聚焦 render 模块」而非单一 800 行 `view.rs`：把 render 树按领域切成
view / ask / changes / review / scrollbar / pr / artifacts / diff 八个小子模块，
`view.rs` 仅作为 render 门面（747 行）。这比 PRD 2.2 的单一 `view.rs` 拓扑更细粒度，
在达成 `< 800 行` 验收的同时保持每个模块更小、更易维护。跨模块引用的纯函数/方法通过
`pub(super)` + `mod.rs` 内 `use` 重导出保持可达，未引入任何 `pub(crate)` 公共 API 扩散。

## 5. 已知限制（残余风险，留待后续 PRD/切片）

- `mod.rs` 仍保留 712 行 effect dispatch（`new`/`refresh`/`run_review_action`/`submit_commit`/
  `submit_ask` 等非 render 方法）。这些是状态变更与 store effect 编排，非 render 树，符合
  PRD「先抽纯函数、再抽 effect、render 只负责调用展示」的分层；如需进一步拆分
  effect dispatch 领域（如 review 编排 / ask 编排）可作后续切片。
- `pr.rs`（705 行）仍偏大，PR 卡片 render 内部可进一步拆分为 check/讨论 子领域，属可选项。
