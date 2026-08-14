# Brooks 架构审计治理 PRD Spec Review 报告

## 1. 总体结论

- 可行性：中高。
- 最大风险：PRD 是 parent-level 治理总纲，如果没有明确关闭口径，容易被执行成一次横跨 Inspector、TerminalPane、ControlServer、Swift/Rust parity 的大重构。
- 推荐方向：`homie-om7` 只关闭 Phase 0 规划闭环；Phase 1-4 必须分别新建 child Beads、独立 PRD/OpenSpec/evidence，并按 dev-loop 单独交付。

## 2. 问题清单与修复记录

| 优先级 | 维度 | 问题 | 影响 | 修复状态 |
|--------|------|------|------|----------|
| P1 | 范围控制 | 一个 PRD 同时覆盖 GPUI 大模块、RootView、Engine Control、Swift/Rust parity | 容易被误解为一次性大重构 | 已修复：新增 parent PRD 关闭口径，明确 `homie-om7` 只关闭 Phase 0 |
| P1 | 存量 PRD 关系 | 已存在 GPUI 和 protocol 相关 PRD，原文未说明继承/去重 | 后续任务可能重复规划 | 已修复：新增存量 PRD/spec 映射表 |
| P1 | SDD/TDD 适配 | 原验证计划只有命令，没有功能验证 Case 编号和需求映射 | OpenSpec 难以证明覆盖 | 已修复：新增 FC-01 到 FC-08，并要求 Finding → Phase → child Bead → Task → Case → Evidence 映射 |
| P1 | 最小化实现 | Phase 1 示例目录较完整，容易诱导一次搬迁整个 Inspector | diff 噪声大、回归风险高 | 已修复：明确第一刀只抽 artifact 子域，并列出禁止事项 |
| P2 | 兼容策略 | ControlServer 拆分未定义 wire compatibility 守卫 | 可能改变 error code、JSON shape、event 顺序 | 已修复：新增协议不变约束 |
| P2 | 运行风险 | GPUI 拆分缺少真实 app 验证入口 | GPUI tests 通过但真实窗口行为回退 | 已修复：FC-08 和 Phase 验证计划加入 dev app smoke |
| P2 | 可扩展性 | Swift/Rust parity 与既有 PRD 重叠 | 可能重新设计 fixture 方案 | 已修复：Phase 4 明确复用 `protocol-contract-golden-fixtures` |

## 3. 整改后的方案摘要

本 PRD 作为架构治理父文档，只交付审计问题记录、功能验证 Case、OpenSpec 对齐和 child Beads 路线。代码拆分不在本轮直接执行。

阶段策略：

1. Phase 0：完成治理基线、功能验证 Case、OpenSpec alignment。
2. Phase 1：后续 child Bead 只抽 Inspector artifact 子域。
3. Phase 2：后续 child Bead 只抽 TerminalPane 一个纯逻辑子域。
4. Phase 3：后续 child Bead 先抽 ControlServer 低风险 method family。
5. Phase 4：后续 child Bead 复用 protocol golden fixtures，补质量门禁。

## 4. 门禁结论

- P0/P1 问题：已在 PRD 中修复。
- P2 问题：已明确处理方案。
- 可进入功能验证 Case 设计和 OpenSpec 拆解。
