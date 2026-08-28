# Code Review Round 1 — llm-gateway-tier3-evidence-hardening

审查范围：本 change 的全部产出（PRD、OpenSpec、failure-model、specs 契约、evidence 文档）。本 change 无生产代码改动，故审查聚焦文档正确性、一致性、证据真实性。

## Findings

| ID | 严重度 | 类别 | 位置 | 证据与影响 | 处置 |
|----|--------|------|------|-----------|------|
| R1-01 | medium | Consistency | `failure-model.md` FM-04 | 引用「FC-04 变异 M4（漏记 output 被杀死）」，但最终变异清单 M4 实为 `routes::apply_model_route` 删守卫，无「漏记 output」变异 | fixed：改为引用 M3（rate-limit off-by-one） |
| R1-02 | low | Correctness | `failure-model.md` FM-03 | 证据标为「fc-03-adversarial.log（单元段）」，但 fc-03 是纯集成日志；apply_model_route 单测在 fc-01 全量日志中 | fixed：改为 `fc-01-baseline.log` |
| R1-03 | low | Correctness | `functional-cases.md` FC-04 初始版 | M1 描述「hash_key 返回明文」为对称变异，不会被测试杀死 | fixed：改为非对称「accept 查找侧不哈希」 |

## 验证

- `git diff --check`（scoped）通过；`specs/llm-gateway.md` §10.1 markdown 表格管道对齐（9 行 `|`）。
- 三处 finding 均已修复，修复后 `failure-model.md` / `functional-cases.md` 与 `functional-verification-report.md` 的变异清单一致。

## 结论

显性问题已全部修复；无 P0/P1 阻断。
