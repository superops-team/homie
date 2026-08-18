# OpenSpec Tasks — small-crate-consolidation-review

## S1 采集事实基线

- [x] 核实三 crate 行数、类型（库/二进制）、依赖、消费者数量、语义。
- [x] 事实表记录于证据报告。
- 验收：事实表完整。关联 C1。

## S2 产出决策表

- [x] 逐个 crate 评估依赖方向、消费者数量、平台门控、制品形态。
- [x] 产出决策：`homie-usage` 保留 / `homie-pty` 保留 / `homie-mcp` 保留（不合并）。
- 验收：每个 crate 有明确决策与理由。关联 C2。

## S3 记录评估结论

- [x] 决策表与结论写入 `docs/verification/small-crate-consolidation-review/release-readiness-report.md`。
- 验收：证据目录就绪。关联 C3。
