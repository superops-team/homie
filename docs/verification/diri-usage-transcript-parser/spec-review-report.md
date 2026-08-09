# Spec Review Report: Diri Usage Transcript Parser

```yaml
change_id: diri-usage-transcript-parser
beads: homie-hd1
status: pass
reviewed_at: 2026-08-08
source_prd: prd-spec/features/diri-usage-transcript-parser/2026-08-08-diri-usage-transcript-parser-design.md
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把 parser 扩成 watcher/cache/storage import，导致范围膨胀。
- 推荐方向：先实现纯 parser 和 neutral event model，后续单独接 storage importer/watcher。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 范围控制 | Diri parser 与 store/watcher/cache 绑定较深。 | 一次实现会影响 storage、watcher、UI。 | 本 slice 只做 parse file -> event vec。 |
| P1 | 数据语义 | Claude cache_creation_input_tokens 与 ephemeral 5m/1h 要同时保留。 | 后续 storage 无法区分 cache write 总数与 duration。 | Event model 保留 cache_write_tokens、cache_write_5m_tokens、cache_write_1h_tokens。 |
| P1 | 稳定去重 | transcript event id 必须稳定。 | storage dedupe 无法可靠工作。 | 使用 path + line offset + provider id hash。 |
| P2 | 错误输入 | transcript 中常见坏 JSON/非 usage 行。 | parser 过于严格会丢整文件。 | 坏行跳过，文件 IO 错误才返回 Err。 |

## 3. 整改后的完善方案

- 目标与范围：纯 transcript parser，输出 neutral usage events。
- 非目标：不做 watcher、offset cache、storage write、UI/fleet。
- 设计原则：Diri 合同 1:1、纯函数、可测试、坏行容忍。
- 核心方案：实现 Claude/Codex JSONL parser，复用 pricing helper。
- 风险控制：fixture tests 覆盖正常、异常和 id 稳定性。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 更新 llm-proxy spec | `specs/llm-proxy/README.md` | PRD | P1 |
| Test | 新增 parser tests | `usage_transcript_parser.rs` | Diri parser source | P1 |
| Logic | 实现 neutral parser | `homie-llm/src/lib.rs` | RED tests | P1 |
| Evidence | 验证、review、readiness、parity lock | `docs/verification/...` | GREEN gates | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| Unit | Claude parser | tokens/cache/session/model/cost | 开发中 |
| Unit | Codex parser | model carry/token_count/cache/cost | 开发中 |
| Unit | Safety | bad JSON/unknown model/negative token/id stable | 开发中 |
| Quality | lint/format/diff/parity | clippy、fmt、diff、make parity-lock | 准出 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| S1 | 1 | PRD/spec/OpenSpec/case | 锁定 parser 范围 | 文档齐备 |
| S2 | 2 | RED tests | 确认 parser 缺失 | failing tests |
| S3 | 3 | 实现 parser | 避免接 storage | GREEN tests |
| S4 | 4 | 门禁与证据 | 如有回归，回到 S3 | readiness report |

## 7. 待确认问题

- 无。
