# OpenSpec Plan — small-crate-consolidation-review

## 概述

评估单文件小 crate `homie-mcp` / `homie-usage` / `homie-pty` 的依赖方向、消费者数量、语义内聚度，产出「合并/保留/重组」决策表，不盲目合并（审计 finding F11，评估性切片）。

## 评估维度

每个 crate 评估：依赖方向（是否底层/leaf）、消费者数量、语义内聚（单一职责）、平台门控、制品形态（库 vs 二进制）。

## 任务清单

| Task | 描述 | 验收 | 关联验证 Case |
|------|------|------|---------------|
| S1 | 采集三 crate 的事实基线（行数/依赖/消费者/语义） | 事实表完整 | C1 |
| S2 | 产出决策表（合并/保留/重组 + 理由） | 每个 crate 有明确决策与理由 | C2 |
| S3 | 记录评估结论到证据目录 | `docs/verification/small-crate-consolidation-review/` 就绪 | C3 |

## 验证口径

- 决策表存在且每个 crate 有明确理由（本切片结论：全部保留，不合并）。
- 无代码落地，故无 `cargo test` 新增运行要求。
