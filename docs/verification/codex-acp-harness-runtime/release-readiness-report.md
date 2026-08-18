# Release Readiness Report — codex-acp-harness-runtime

## 1. 变更类型

设计/规范交付（无 Rust 代码变更）。change_id: `codex-acp-harness-runtime`，Beads: `homie-sc6`。

## 2. 交付物清单

| 文件 | 状态 |
|------|------|
| `prd-spec/features/codex-acp-harness-runtime/2026-08-16-codex-acp-harness-runtime-design.md` | 新增 |
| `docs/design/apple-design-principles.md` | 新增 |
| `docs/research/comet-gpui-chat-boundaries.md` | 新增 |
| `docs/research/gpui-component-compat-gate.md` | 新增 |
| `openspec/changes/codex-acp-harness-runtime/{plan,tasks,alignment-report}.md` | 新增 |
| `docs/verification/codex-acp-harness-runtime/{spec-review,functional-cases,functional-verification,release-readiness}.md` | 新增 |

## 3. 门禁检查

- 无 Rust 代码变更，故 `cargo check`/`fmt`/`test` 无需重跑（工作区除文档外无 diff）。
- spec review：16 维度 PASS / N/A，无 P0-P3 问题。
- 功能验证：7/7 PASS。
- OpenSpec alignment：FR-1..FR-7 全覆盖，零漏项。

## 4. 版本标签

按 `AGENTS.md` 版本规则，本变更为"documentation/process updates that affect development
behavior" → **patch** 递增。下一个 tag：`v0.1.16`。

## 5. 后续 child Bead（不属本变更）

- `codex-acp-host-runtime`（backend harness 代码落地）
- `chat-surface-gpui`（GPUI chat canvas 代码落地）
- 真实 provider（Claude/OpenCode ACP）接入

## 6. 结论

设计交付完整、证据齐备，可提交并关闭 Beads `homie-sc6`。
