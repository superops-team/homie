# 功能验证 Case 清单 — codex-acp-harness-runtime

本变更为设计/规范交付，验证 Case 验证文档/规范的可判定性与完整性，而非运行时行为。

## FC-1: ACP 协议选型与 pinning 决策

- 断言：PRD §FR-1 明确"ACP + pinned `codex-acp`"，direct app-server 仅作 adapter 参考。
- 预期：文本中出现"ACP"、"pinned `codex-acp`"、"adapter 内部/历史参考"。

## FC-2: backend harness 模块边界与数据模型

- 断言：PRD §4.1 定义 `homie-engine/src/acp/*`，§4.2 定义 transcript 数据模型与
  `AgentDriverControl` trait，并引用 `typed-agent-driver-capabilities` 对齐。
- 预期：文本含模块清单、trait、SessionTurn/MessageBlock/ToolCallBlock/PermissionBlock。

## FC-3: GPUI chat canvas 交互契约

- 断言：PRD §FR-4/§4.3 定义 composer（send/steer/stop）、transcript、approval_view，并引用
  `specs/gpui-shell.md` render contract。
- 预期：文本含 `homie-app/src/chat/*` 模块清单与 render contract 约束。

## FC-4: approval 四态语义

- 断言：PRD §FR-5 定义 allow/deny once + always allow/deny for session 四态。
- 预期：文本含四态枚举与 per-session 记忆语义。

## FC-5: Apple/design 规范

- 断言：`docs/design/apple-design-principles.md` 覆盖 HIG 原则、design tokens、动效、平台偏好
  （reduce motion 等）。
- 预期：文本含"清晰/遵从/深度"、"reduce motion"、"design tokens"。

## FC-6: Comet 边界 + gpui-component 门禁

- 断言：`docs/research/comet-gpui-chat-boundaries.md` 与
  `docs/research/gpui-component-compat-gate.md` 产出，结论可追溯，门禁结论为"不引入依赖"。
- 预期：Comet 结论含 6 条边界原则；门禁结论含"不引入 + license 审计前置"。

## FC-7: OpenSpec alignment + 证据齐备

- 断言：`openspec/changes/codex-acp-harness-runtime/` 三文件齐备，alignment 映射 FR-1..FR-7。
- 预期：plan/tasks/alignment 存在，alignment 表覆盖全部 FR。

## 覆盖矩阵

| PRD 需求项 | 验证 Case |
|-----------|-----------|
| FR-1 | FC-1 |
| FR-2/FR-3 | FC-2 |
| FR-4 | FC-3 |
| FR-5 | FC-4 |
| FR-6 | FC-5 |
| FR-7 | FC-6 |
| 验收 §8 | FC-7 |
