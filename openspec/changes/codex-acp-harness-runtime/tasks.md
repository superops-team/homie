# OpenSpec Tasks — codex-acp-harness-runtime

本变更为设计/规范交付，tasks 为文档产出与验收动作，不含 Rust 实现。

## T1: ACP 协议选型与 pinning 决策固化

- 验收：PRD §FR-1 明确 ACP + pinned `codex-acp`，direct app-server 仅作 adapter 参考。
- 关联验证 Case：FC-1。

## T2: backend harness 模块边界与数据模型定义

- 验收：PRD §4.1/§4.2 定义 `homie-engine/src/acp/*` 模块边界、`AgentDriverControl` trait、
  ACP transcript 数据模型，与 `typed-agent-driver-capabilities` 对齐。
- 关联验证 Case：FC-2。

## T3: GPUI chat canvas 交互契约定义

- 验收：PRD §FR-4/§4.3 定义 composer（send/steer/stop）、transcript、approval_view 边界，
  遵循 `specs/gpui-shell.md` render contract。
- 关联验证 Case：FC-3。

## T4: approval 四态语义定义

- 验收：PRD §FR-5 定义 allow/deny once + always allow/deny for session。
- 关联验证 Case：FC-4。

## T5: Apple/design 规范产出

- 验收：`docs/design/apple-design-principles.md` 覆盖 HIG 原则、design tokens、动效、平台偏好。
- 关联验证 Case：FC-5。

## T6: Comet 模块边界学习 + gpui-component 兼容性门禁

- 验收：`docs/research/comet-gpui-chat-boundaries.md` 与
  `docs/research/gpui-component-compat-gate.md` 产出，结论可追溯。
- 关联验证 Case：FC-6。

## T7: OpenSpec alignment + 证据 + 关闭

- 验收：`openspec/.../alignment-report.md`、`docs/verification/codex-acp-harness-runtime/`
  证据齐备，Beads `homie-sc6` 关闭。
- 关联验证 Case：FC-7。
