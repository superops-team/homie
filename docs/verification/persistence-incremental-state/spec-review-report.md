# 持久化分层/行级状态演进 Spec Review Report

## 1. 总体结论

- 可行性：中。
- 最大风险：数据迁移直接默认启用，导致真实 `state.json` 损坏、会话丢失或难以回滚。
- 推荐方向：首阶段只做 split JSON store 和安全迁移，不引入 SQLite；通过 dry-run、backup、checksum 和 opt-in/internal gate 降低风险。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 修复状态 |
|---|---|---|---|---|
| P0 | 数据安全 | 原 PRD 写了迁移失败不删除旧文件，但缺 dry-run、backup 校验、record hash 对比 | 真实用户会话可能因迁移 bug 丢失 | 已修复：新增 dry-run、backup、record count/hash 校验、失败回滚 |
| P0 | 默认启用 | 未说明 split store 首阶段是否默认启用 | 风险未经真实数据验证就进入主路径 | 已修复：默认启用/opt-in 策略必须在 OpenSpec 明确，必要时先 internal gate |
| P1 | 双写风险 | 可能为了兼容同时写旧 envelope 和 split store | 两个存储源可能漂移 | 已修复：首阶段选择单一 active store；双写需独立 OpenSpec |
| P1 | durable spec | 存储格式成为长期合同但未要求 specs 更新 | 后续实现依赖 PRD 细节，缺长期边界 | 已修复：OpenSpec 需评估更新 `specs/engine-session-runtime.md` 或新增 persistence spec |
| P2 | 性能验证 | 只描述性能目标，缺规模基线 | 5000 sessions 下文件枚举可能退化 | 已修复：要求 100/1000/5000 records 基线 |

## 3. 整改后的完善方案

首阶段实现 `PersistenceStore` 窄 trait、`JsonEnvelopeStore` 和 `SplitJsonStore`，并提供旧 `state.json` 到 split files 的安全迁移。迁移必须支持 dry-run，成功后保留 backup，失败不删除旧文件。单 session 文件损坏只 quarantine 单个文件，并在 migration report 中记录恢复路径。

非目标：不迁移 OutputLog，不改 remote binding，不引入 SQLite，不引入云同步，不迁移 provider credential/config。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 补 OpenSpec 与 durable spec 影响评估 | alignment report | 本报告 | P0 |
| Store | 定义窄 `PersistenceStore` trait | Rust code + tests | OpenSpec | P0 |
| Migration | dry-run/backup/checksum/quarantine | migration tests | Store | P0 |
| Registry | Registry 依赖 store trait | integration tests | Migration | P0 |
| Performance | 规模基线 | benchmark log/report | Registry | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| Unit | migration dry-run | 不写目标文件 | 开发中 |
| Unit | migration rollback | 失败后旧 `state.json` 仍可加载 | 开发中 |
| Unit | quarantine | 单 session 文件坏不影响其他 session | 开发中 |
| Integration | daemon restart | create/archive/rename 后重启一致 | 准出前 |
| Performance | 写放大和 load time | 100/1000/5000 sessions | 准出前 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| Phase 0 | 先 | OpenSpec + fixture/real data risk plan | 默认启用需谨慎 | alignment report |
| Phase 1 | 次 | store trait + migration tests | 保持旧 envelope 可回滚 | test logs |
| Phase 2 | 后 | Registry 接入 + restart/perf evidence | opt-in gate 可作为缓冲 | verification report |

## 7. 待确认问题

- 首阶段是否默认启用 split store。推荐在 OpenSpec 中先设为 opt-in/internal gate，除非有足够真实 `state.json` fixture 验证迁移安全。
