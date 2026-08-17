# App Inspector Module Split — Alignment Report

## PRD ↔ OpenSpec 映射

PRD：`prd-spec/refactors/app-inspector-module-split/2026-08-17-app-inspector-module-split-design.md`

| PRD 章节 | 内容 | OpenSpec Task |
|----------|------|---------------|
| 1.1 问题/动机 | F3 Cognitive Overload，4692 行单文件 | T1 基线 |
| 1.2 目标 | 拆「纯逻辑/render/effect」分层，render 留宿主 | T2–T5 |
| 2.1 拆分原则 | 先抽无 Window/Context/Entity 依赖纯函数 | T2–T4 |
| 2.2 模块拓扑 | view/state/projection/policy/tests | T2–T5 |
| 2.3 实施顺序 | S1 state → S2 projection → S3 policy → S4 view | T2 → T3 → T4 → T5 |
| 3 测试与验收 | view.rs < 800 行；子模块无 GPUI 依赖；test 全绿 | T5、T6 |
| 4 Beads 追踪 | change_id、homie-ubu.3 | T1、T6 |

## 切片顺序对齐

PRD 2.3 明确顺序为：S1 `state.rs` → S2 `projection.rs` → S3 `policy.rs` → S4 `view.rs`。
`plan.md` 与 `tasks.md` 均已按此顺序对齐（T2=state、T3=projection、T4=policy、T5=view）。

## 验收标准对齐

- `inspector/view.rs` < 800 行 → T5。
- 子模块无 GPUI render 依赖、可独立单测 → T2/T3/T4。
- `cargo test -p homie-app` 全绿、视觉行为不变 → T2–T5 每步门禁。

## 结论

100% 对齐原始 PRD，零漏项、零错配。可进入实现。
