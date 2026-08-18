# OpenSpec Alignment Report — codex-acp-harness-runtime

## 1. 对齐结论

本变更的 PRD 需求项与 OpenSpec tasks、功能验证 Case 已逐项对齐，零漏项、零错配。

## 2. PRD 需求项 → Tasks → 验证 Case 映射

| PRD 需求项 | 内容 | Task | 验证 Case |
|-----------|------|------|-----------|
| FR-1 | ACP 协议选型与 pinning | T1 | FC-1 |
| FR-2 | ACP host harness（backend） | T2 | FC-2 |
| FR-3 | typed session/capability 对齐 | T2 | FC-2 |
| FR-4 | GPUI New Session chat canvas | T3 | FC-3 |
| FR-5 | approval 四态语义 | T4 | FC-4 |
| FR-6 | Apple 设计一致性 | T5 | FC-5 |
| FR-7 | Comet 边界 + gpui-component 门禁 | T6 | FC-6 |
| 验收 §8 | OpenSpec alignment + 证据 + 关闭 | T7 | FC-7 |

## 3. 非目标对齐确认

- 不实现真实 provider 端到端运行：已在本 PRD §1.3 与 plan §4 明确，tasks 无对应实现项。
- 不引入 gpui-base/gpui-component：已在兼容性门禁结论明确，无对应依赖引入任务。
- 不改 MCP/CLI typed control：已在 §1.3 明确，tasks 无对应项。

## 4. 不一致项

无。

## 5. 结论

对齐 100%，可进入证据产出与关闭流程。
