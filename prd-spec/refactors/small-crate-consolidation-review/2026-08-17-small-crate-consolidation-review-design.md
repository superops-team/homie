# 单文件小 Crate 合并评估设计文档

## 1. 概述

### 1.1 问题/动机

`homie-mcp`（241 行）、`homie-usage`（162 行）、`homie-pty`（430 行）为单文件小 crate，边界价值
与维护成本需评估：是否合并、保留或重组。2026-08 审计 finding **F11（Suggestion）**：Accidental
Complexity。本 PRD 是评估性切片，先产出合并/保留决策与理由，再决定是否代码落地。

### 1.2 目标

评估三小 crate 的依赖方向、消费者数量、语义内聚度，产出「合并/保留/重组」决策表与证据，不
盲目合并。

### 1.3 非目标

不机械合并；不改变 public API 除非决策明确要求；评估未通过前不做代码移动。

### 1.4 基线

- branch `main`，commit `e4c7454`；目标 `homie-mcp`/`homie-usage`/`homie-pty`。

## 2. 方案设计

### 2.1 评估维度

每个 crate 评估：依赖方向（是否底层）、消费者数量、语义内聚（是否单一职责）、变化频率。

### 2.2 决策输出

产出决策表（合并到 X / 保留 / 拆分），记录到 `docs/verification/small-crate-consolidation-review/`。

### 2.3 实施顺序

S1 评估与决策表；S2 若决策为合并则执行并验证 `cargo test` 全 workspace 绿。

## 3. 测试与验收

- 验收：决策表存在且每个 crate 有明确理由；若合并，全 workspace `cargo test` 全绿且 API 兼容。
- 证据目录：`docs/verification/small-crate-consolidation-review/`

## 4. Beads 追踪

- change_id `small-crate-consolidation-review`；parent `homie-ubu`；child `homie-ubu.9`；P2。
