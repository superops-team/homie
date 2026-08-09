# Spec Review Report: Diri Usage Scan File Offset Cache

```yaml
change_id: diri-usage-scan-cache
beads: homie-xaz
status: pass
reviewed_at: 2026-08-08
source_prd: prd-spec/features/diri-usage-scan-cache/2026-08-08-diri-usage-scan-cache-design.md
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把 repository API 扩成 watcher 或文件系统扫描器。
- 推荐方向：只补 `usage_scan_files` CRUD/query，后续 watcher 复用。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 范围控制 | offset cache 与 watcher 强相关。 | 容易引入 notify/文件系统依赖。 | 本 slice 禁止扫描文件，仅存取状态。 |
| P1 | 更新语义 | 同 path upsert 必须覆盖旧 offset/model。 | 重写文件后状态不一致。 | 使用 `ON CONFLICT(path) DO UPDATE`。 |
| P2 | 查询边界 | profile_id 可能为空。 | SQL 过滤语义易错。 | query 使用 Option，None 不过滤。 |

## 3. 整改后的完善方案

- 目标与范围：`usage_scan_files` repository API。
- 非目标：watcher、tail hash 计算、parser invocation、usage record import。
- 核心方案：新增 state/query struct 和 upsert/get/list。
- 风险控制：测试覆盖 upsert overwrite、provider/profile filtering。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Test | 新增 scan cache tests | `usage_scan_cache.rs` | existing schema | P1 |
| Logic | 实现 repository API | `homie-storage/src/lib.rs` | RED tests | P1 |
| Evidence | 验证和 parity lock | verification docs | GREEN gates | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| Integration | upsert/get | same path overwrite | 开发中 |
| Integration | list filter | provider/profile filters | 开发中 |
| Regression | schema inventory | existing storage indexing | 实现后 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| S1 | 1 | PRD/spec/OpenSpec/case | 锁定 repository 范围 | 文档齐备 |
| S2 | 2 | RED tests | 确认 API 缺失 | failing tests |
| S3 | 3 | 实现 repository | 不改 schema | GREEN tests |
| S4 | 4 | 门禁与证据 | 如有回归，回到 S3 | readiness report |

## 7. 待确认问题

- 无。
