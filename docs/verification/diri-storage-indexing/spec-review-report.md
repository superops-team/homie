# Spec Review Report: Diri Storage/Indexing Parity Phase 1

```yaml
change_id: diri-storage-indexing
beads: homie-q7n
reviewed:
  - prd-spec/features/diri-storage-indexing/2026-08-07-diri-storage-indexing-design.md
  - specs/storage-indexing/README.md
status: pass_after_inline_revision
review_method: local review-spec workflow
```

## 1. 总体结论

- 可行性：高。
- 最大风险：storage schema 作为 L0 foundation，一旦字段或唯一约束缺失，后续 runtime/context/LLM/UI lane 会绕过 repository API 或重复改 migration。
- 推荐方向：本阶段只固化 schema inventory、唯一约束、索引和 repository/query API，不做 UI/runtime/git/transcript/LLM proxy，实现面保持在 `crates/homie-storage`。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 | 处理 |
|---|---|---|---|---|---|
| P0 | Diri 对齐 | 原 `specs/storage-indexing/README.md` 只列粗表名，没有 M05/M06/M07/M17/M19 的字段与唯一约束 | 后续 migration 会漂移，无法证明 Diri parity | 增加 table-by-table schema inventory | 已在 `specs/storage-indexing/README.md` 第 12 节补齐 |
| P0 | OpenSpec 准备 | 原需求没有把 PRD requirement 映射到可执行任务与验证 case | 可能跳过 dev-loop 直接编码 | 新建中文 PRD，后续 OpenSpec 逐项映射 FR/FC | 已新建 PRD，OpenSpec 待写 |
| P1 | Repository API | 原规格没有说明各模块是否能直接写 SQL | runtime/LLM/UI 可能绕过 storage 边界 | 增加 repository/query API ownership 表 | 已补齐 API ownership |
| P1 | 安全 | usage/preferences/history JSON 字段缺少 safe-field 限制 | 可能写入 raw key、Authorization 或完整 tool args/result | 增加 storage security rules | 已补齐第 12.5 节 |
| P1 | 测试可执行性 | 原测试计划没有绑定新表字段、索引、API | 测试可能只覆盖表存在 | 增加 FC-STOR-001..007 | 已在 spec 第 12.4 节定义，functional cases 待写 |
| P2 | 范围控制 | Diri 源中包含 scanner/parser/UI 行为，容易扩大到非 storage | 超出 worker-storage 写入范围 | PRD 明确非目标：不做 UI/runtime/git/parser | 已补齐非目标 |

## 3. 整改后的完善方案

### 3.1 目标与范围

本阶段把 Diri storage/indexing parity 的基础合同落地为长期组件规格和 `homie-storage` 最小实现。范围仅限 SQLite schema、migration、关系/唯一约束、repository/query API 和 storage tests。

### 3.2 非目标

不实现 command palette、quick open UI、settings UI、worktree shell 命令、history scanner、usage transcript parser、LLM proxy、runtime process lifecycle 或 remote sync。

### 3.3 设计原则

- SQLite 是本地事实源。
- 模块通过 `homie-storage` repository/query API 访问，不直接散落裸 SQL。
- schema forward-only，不保留旧版本兼容 fallback。
- raw secret、Authorization、cookie、raw prompt/body、完整 tool args/result 不进入 storage。

### 3.4 核心方案

- `specs/storage-indexing/README.md` 第 12 节成为 M05/M06/M07/M17/M19 的持久化 contract。
- `crates/homie-storage` schema version 前进，新增缺失字段、索引、usage scan/rollup 表。
- 增加 typed API：history、project/worktree、session core metadata、usage record/query、schema inventory。
- storage tests 通过 public API 和 SQLite metadata 验证行为。

### 3.5 兼容与风险控制

- 不做 downgrade/fallback。新版 schema 只向前迁移，遇到更高版本 fail closed。
- 对既有 v1/v2 空库测试保持可迁移；不承诺旧用户数据兼容。
- 若其他 lane 的 Cargo workspace 失败，不把非本 lane 失败写成 pass；release readiness 中标注阻塞来源。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 补齐中文 PRD 与长期组件规格 | PRD、`specs/storage-indexing/README.md` | Diri inventory | P0 |
| Verification Design | 写 functional cases 和覆盖矩阵 | `functional-cases.md` | PRD/spec review | P0 |
| OpenSpec | 写 plan/tasks/alignment | `openspec/changes/diri-storage-indexing/*` | functional cases | P0 |
| TDD | 先写 storage RED tests | `crates/homie-storage/tests/diri_storage_indexing.rs` | OpenSpec tasks | P0 |
| Implementation | schema/API 最小实现 | `crates/homie-storage/src/lib.rs` | RED tests | P0 |
| Verification | 运行 focused gates | results/release readiness | 实现完成 | P0 |
| Review | 两轮 code review | `code-review-round-1.md`, `code-review-round-2.md` | verification | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| Schema inventory | 表、字段、索引、唯一约束 | FC-STOR-001 | TDD RED/GREEN |
| Repository API | preferences/history/project/worktree/session/usage | FC-STOR-002..006 | TDD RED/GREEN |
| Migration | 空库迁移、幂等、schema too new | existing + focused tests | verification |
| Security | raw-sensitive 字段不进入 API/报告 | review + test naming/fields | review |
| Quality | fmt/check/test/diff/pre-commit | FC-STOR-007 | final verification |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| 1 | 当前 | PRD/spec review/functional cases | 防止实现前范围漂移 | PRD、spec review、functional cases |
| 2 | 下一步 | OpenSpec plan/tasks/alignment | 任务必须绑定 FC | OpenSpec 三件套 |
| 3 | 后续 | TDD storage tests + 实现 | migration 和约束错误最容易暴露 | focused tests pass |
| 4 | 最后 | 验证、两轮 review、readiness | workspace 可能受其他 lane 影响 | evidence reports |

## 7. 待确认问题

- 无需用户补充信息；本轮只按 `homie-q7n` 的 phase 1 storage/indexing 范围推进。
