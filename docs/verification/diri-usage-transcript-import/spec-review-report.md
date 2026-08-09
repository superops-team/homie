# Spec Review Report: Diri Usage Transcript Storage Import

```yaml
change_id: diri-usage-transcript-import
beads: homie-dh8
status: pass
reviewed_at: 2026-08-08
source_prd: prd-spec/features/diri-usage-transcript-import/2026-08-08-diri-usage-transcript-import-design.md
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把 importer 扩成 watcher/cache 或修改 storage schema。
- 推荐方向：新增最小 importer API，复用现有 `record_usage` 和唯一索引去重。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 范围控制 | importer 容易扩成扫描目录和 offset cache。 | 改动面扩大。 | 本 slice 只接收已解析 events。 |
| P1 | 去重语义 | transcript event id 必须进入 source_event_id。 | 重复导入会重复计费。 | request_id/source_event_id 都使用 event id，复用 storage unique index。 |
| P2 | Metadata | Transcript event 不包含 runtime/agent/llm profile。 | RecordUsage 必填字段无法构造。 | caller 提供 `UsageImportDefaults`。 |

## 3. 整改后的完善方案

- 目标与范围：neutral transcript event -> storage usage record。
- 非目标：不做 watcher、offset cache、pricing snapshot、UI/fleet。
- 核心方案：`Storage::record_transcript_usage_events(events, defaults)`。
- 风险控制：测试重复导入、summary totals、estimated cost 和 cache fields。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Test | 新增 importer tests | `usage_transcript_import.rs` | parser event model | P1 |
| Logic | 实现 importer API | `homie-storage/src/lib.rs` | RED tests | P1 |
| Evidence | 验证和 parity lock | verification docs | GREEN gates | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| Integration | import totals | two events into storage summary | 开发中 |
| Integration | dedupe | same event twice skipped | 开发中 |
| Regression | storage indexing | existing storage tests | 实现后 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| S1 | 1 | PRD/spec/OpenSpec/case | 锁定 scope | 文档齐备 |
| S2 | 2 | RED tests | 确认 API 缺失 | failing tests |
| S3 | 3 | 实现 importer | 不改 schema | GREEN tests |
| S4 | 4 | 门禁与证据 | 如有回归，回到 S3 | readiness report |

## 7. 待确认问题

- 无。
