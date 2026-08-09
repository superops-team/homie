# Spec Review Report: Diri Usage Pricing Estimate

```yaml
change_id: diri-usage-pricing
beads: homie-t3e
status: pass
reviewed_at: 2026-08-08
source_prd: prd-spec/features/diri-usage-pricing/2026-08-08-diri-usage-pricing-design.md
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把 pricing helper 扩成完整 transcript watcher 或 billed cost 系统。
- 推荐方向：先 1:1 迁移 Diri `diri-usage` pricing helper 到 `homie-llm`，用单元测试锁定模型匹配和 cache 价格。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 范围控制 | USAGE-001 包含 parser、pricing、watcher、UI/fleet，多项不应一次实现。 | 改动过大且难以验证。 | 本 slice 只实现 pricing estimate helper。 |
| P1 | 语义准确 | 模型匹配顺序如果不与 Diri 一致，会导致 opus/codex 等泛化规则抢先命中。 | 成本估算错误。 | 测试覆盖 specific-to-generic 顺序。 |
| P1 | cache 规则 | Claude cache write 5m/1h 与 cache read 价格倍率不同。 | cache 成本低估或高估。 | 单独测试 cache read/write 5m/write 1h。 |
| P2 | billing 表述 | estimated cost 不是 provider billed cost。 | UI/报告误导用户。 | 文档和 API 命名保留 estimate。 |

## 3. 整改后的完善方案

- 目标与范围：Diri-compatible local pricing estimate helper。
- 非目标：不实现 watcher、storage write、UI/fleet、billed spend。
- 设计原则：Diri 规则 1:1、纯函数、无外部依赖。
- 核心方案：在 `homie-llm` 添加 `ModelPricing`、provider matchers 和 estimate functions。
- 风险控制：所有关键规则都有测试，unknown model 返回 None。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 更新 llm-proxy spec | `specs/llm-proxy/README.md` | PRD | P1 |
| Test | 新增 pricing tests | `usage_pricing.rs` | Diri pricing source | P1 |
| Logic | 实现 pricing helper | `homie-llm/src/lib.rs` | RED tests | P1 |
| Evidence | 验证、review、readiness、parity lock | `docs/verification/...` | GREEN gates | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| Unit | model matching | Claude/OpenAI specific-to-generic | 开发中 |
| Unit | cache rates | cache read/write 5m/write 1h | 开发中 |
| Unit | safety | unknown model, negative tokens | 开发中 |
| Quality | lint/format/diff/parity | clippy、fmt、diff、make parity-lock | 准出 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| S1 | 1 | PRD/spec/OpenSpec/case | 锁定 pricing 范围 | 文档齐备 |
| S2 | 2 | RED tests | 确认 helper 缺失 | failing tests |
| S3 | 3 | 实现 helper | 对齐 Diri 规则 | GREEN tests |
| S4 | 4 | 门禁与证据 | 如有回归，回到 S3 | readiness report |

## 7. 待确认问题

- 无。本 slice 可独立开发。
