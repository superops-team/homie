# OpenSpec Plan — codex-acp-harness-runtime

## 1. 变更概述

本变更是 **Codex ACP harness + GPUI Chat Session Surface 的设计/规范交付**（非代码实现）。
锁定 ACP 协议 + pinned `codex-acp`，定义 backend harness、typed session/capability、非 PTY
transcript、composer send/steer/stop、approval 四态，并产出 Apple/design 规范、Comet 模块边界
学习、gpui-base/gpui-component 兼容性门禁。

真实 `codex-acp` 进程端到端运行与完整 UI 实现属后续 child Bead，不属本变更。

## 2. 模块划分与依赖

```text
设计交付
├── PRD（prd-spec/features/codex-acp-harness-runtime/）        ← 主规范
├── Apple/design 规范（docs/design/apple-design-principles.md）
├── Comet 模块边界学习（docs/research/comet-gpui-chat-boundaries.md）
├── gpui-component 兼容性门禁（docs/research/gpui-component-compat-gate.md）
└── 证据（docs/verification/codex-acp-harness-runtime/）
```

依赖：本变更不产生 Rust 代码，因此无 crate 依赖图。文档间依赖：

- PRD 引用 design/research 三份文档（§6）；
- OpenSpec alignment 引用 PRD 需求项（FR-1..FR-7）映射。

## 3. 层级关系

| 层 | 产物 |
|----|------|
| 需求 | `prd-spec/.../2026-08-16-codex-acp-harness-runtime-design.md` |
| 规范 | `docs/design/apple-design-principles.md` + `docs/research/*` |
| 执行 | 本 OpenSpec plan/tasks/alignment |
| 证据 | `docs/verification/codex-acp-harness-runtime/` |

## 4. 后续 child Bead 依赖（本变更只声明，不实现）

- `codex-acp-host-runtime`（backend harness 代码落地）
- `chat-surface-gpui`（GPUI chat canvas 代码落地）
- 真实 provider（Claude/OpenCode ACP）接入
