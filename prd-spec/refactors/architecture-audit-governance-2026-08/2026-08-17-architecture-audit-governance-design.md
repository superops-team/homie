# Homie 架构治理总纲（2026-08 审计）设计文档

## 1. 概述

### 1.1 问题/动机

2026-08-17 基于 `brooks-audit`（Brooks-Lint 架构审计）+ `improve-codebase-architecture`
（depth/locality/leverage 视角）对 Homie Rust workspace（`homie/crates/*` 13 个 crate，
108,044 行，206 源文件）做了一次架构审计，Health Score 53/100。

审计结论：依赖方向干净（`proto` 底层无依赖、无循环依赖、`app` 通过 `client`+`proto` 与
daemon 解耦），这是好底子；但两个组合根（`homie-app` 46,570 行、`homie-engine` 26,285 行）
已膨胀成 God Module，且上一轮 `architecture-audit-hardening` 规划的 Phase 1–4 代码重构
至今未执行（目标文件行数与基线完全一致）。

本 PRD 是本次审计的**治理总纲（parent-level）**：记录全部 findings、规划拆分顺序与
child Beads，并为每个 child 明确验收口径。代码落地由 child PRD 各自完成，本 PRD 不
直接承诺任何代码重构。

### 1.2 目标

1. 把 2026-08-17 审计的 11 条 findings 固化为可追踪的 parent PRD，而非停留在聊天结论。
2. 规划 child Beads 拆分拓扑（9 个模块深化切片），明确依赖顺序与优先级。
3. 与存量 `architecture-audit-hardening`（Phase 1–4）对齐，复用其 child 命名与拆分原则，
   不重复新建路线。
4. 每个 child 的验收口径明确：行为不变、纯逻辑优先、测试 seam 清晰、目标文件行数下降。

### 1.3 非目标

- 不在本 PRD 中直接实施任何代码重构；代码落地必须由 child PRD/OpenSpec/evidence 独立交付。
- 不一次性重写 `homie-app` 或 `homie-engine`。
- 不改变现有用户可见功能、协议语义（wire shape）、会话数据或远程运行行为。
- 不机械按行数拆分；只有能降低真实变化半径、明确生命周期或提升测试 seam 时才执行。
- 不覆盖 Swift/Rust 协议 drift（属 `architecture-audit-hardening` Phase 4 / `protocol-parity-quality-gate`，
  不在本次 Rust 结构审计范围内）。

### 1.4 基线快照

- branch: `main`
- baseline commit: `e4c7454`（证据驱动链路修复后的 HEAD）
- audit mode: `brooks-audit`（Architecture Audit）
- health score: `53/100`

每个 child 启动前必须刷新基线（目标文件行数、测试状态、相关 specs、未提交工作区），
不能只复用本文静态行数。

### 1.5 与存量 PRD 的关系

| 存量文档 | 关系 |
|----------|------|
| `architecture-audit-hardening` | 上游 Brooks 审计总纲（homie-om7）。本次审计是其延续与扩展：复用其 Phase 1/2/3 的 Inspector/TerminalPane/ControlServer 拆分原则与 child 命名，补齐其未覆盖的 surface_shell/sidebar/store/root/registry/session/terminal-state 与小 crate |
| `gpui-architecture-hardening` | GPUI shell 合同与 child Beads 思路，复用其"纯逻辑优先、行为不变" |
| `gpui-large-module-test-boundaries` | 大模块纯逻辑测试边界，Phase P1 复用 |
| `specs/gpui-shell.md` | RootView/store 变更必须同步评估是否更新 |
| `specs/engine-session-runtime.md` | engine 拆分必须保持 runtime authority 与 PTY 环境合同不变 |

## 2. 审计 Findings（11 条，Iron Law 全字段在审计报告，此处为摘要索引）

| ID | 风险 | 目标 | 行数 | 严重度 | child change_id |
|----|------|------|------|--------|-----------------|
| F1 | Cognitive Overload | `homie-app` 组合根爆炸 | 46,570（59 文件） | Critical | 见 F3–F6 |
| F2 | Change Propagation | `engine/control.rs` dispatcher+runtime 双职责 | 3,802 | Critical | `engine-control-wire-runtime-split` |
| F3 | Cognitive Overload | `app/inspector.rs` | 4,692 | Critical | `app-inspector-module-split` |
| F4 | Cognitive Overload | `app/surface_shell.rs` | 4,362 | Critical | `app-surface-shell-module-split` |
| F5 | Cognitive Overload | `app/sidebar/view.rs` | 4,310 | Critical | `app-sidebar-view-module-split` |
| F6 | Cognitive Overload | `app/terminal_pane.rs` | 3,495 | Critical | `app-terminal-pane-module-split` |
| F7 | Cognitive Overload | `engine/session.rs` + `engine/registry.rs`（registry 双职责：live session + persistence） | 2,888 + 1,790 | Warning | `engine-registry-session-split` |
| F8 | Knowledge Duplication | app store 与 engine registry 双重投影（session/project 概念） | store 2,434 | Warning | `app-root-store-deepening` |
| F9 | Cognitive Overload | `app/root.rs`（违反 gpui-shell 合同）+ `app/store/mod.rs` | 2,130 + 2,434 | Warning | `app-root-store-deepening` |
| F10 | Cognitive Overload | `homie-terminal-state/src/lib.rs` 单文件 | 1,168 | Suggestion | `terminal-state-module-split` |
| F11 | Accidental Complexity | 单文件小 crate：`homie-mcp`(241)/`homie-usage`(162)/`homie-pty`(430) | — | Suggestion | `small-crate-consolidation-review` |

## 3. 拆分拓扑与依赖顺序

按「引擎核心 → 应用组合根 → 收尾」排序，优先级 P0 > P1 > P2。同优先级可并行。

```text
P0 (引擎核心深化，先做，blast radius 最大)
  ├─ engine-control-wire-runtime-split   (F2, control.rs 3802)
  └─ engine-registry-session-split       (F7, registry 1790 + session 2888)

P1 (应用组合根拆分，4 个 GPUI container + 状态/组合根)
  ├─ app-inspector-module-split          (F3, 4692)
  ├─ app-terminal-pane-module-split      (F6, 3495)
  ├─ app-surface-shell-module-split      (F4, 4362)
  ├─ app-sidebar-view-module-split       (F5, 4310)
  └─ app-root-store-deepening            (F8+F9, root 2130 + store 2434)

P2 (收尾)
  ├─ terminal-state-module-split         (F10, 1168)
  └─ small-crate-consolidation-review    (F11, 评估性)
```

依赖关系：P0 两个 child 相互独立可并行；P1 的 4 个 GPUI container 相互独立可并行，
`app-root-store-deepening` 建议在至少一个 container 拆分完成后启动（以复用拆分 seam 经验）；
P2 独立。

## 4. 每个 child 的通用验收口径

1. 目标文件行数下降（拆分后单文件尽量 < 800 行，拆出的子模块内聚单一职责）。
2. 行为不变：`cargo test --manifest-path homie/Cargo.toml -p <crate>` 全绿，无新增失败。
3. 纯逻辑优先：先抽可独立测试的纯函数/状态机，再抽 GPUI entity。
4. 测试 seam 清晰：拆出的子模块能脱离宿主独立单测（不要求 GPUI render 环境）。
5. 遵守 `docs/development/standards.md` §6 的 RED→GREEN→REFACTOR 与 Tier 分层（大文件拆分属
   Tier 2；涉及并发/凭据/数据丢失的按 Tier 3）。
6. 触及 `specs/*.md` 契约的，同步更新 spec 并记录。

## 5. 验收标准

1. 本 parent PRD 记录全部 11 条 findings，且每条映射到 child change_id 或显式 defer 原因。
2. 9 个 child Beads 已创建，P0 两个有完整 child PRD + OpenSpec。
3. 每个 child 独立交付，不挂在同一个长分支上。

## 6. Beads 追踪

- change_id: `architecture-audit-governance-2026-08`
- 类型: refactor（架构治理总纲，parent）
- 优先级: P0
- child Beads: 见 §3 拓扑，共 9 个
