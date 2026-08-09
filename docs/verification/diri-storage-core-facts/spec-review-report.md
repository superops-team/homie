# Spec Review Report: diri-storage-core-facts

```yaml
change_id: diri-storage-core-facts
bead: homie-t3u.2
review_date: 2026-08-09
checkpoint_commit: 48f522b
result: pass
implementation_authorized: false
```

## 评审概览

| 维度 | 判定 | 关键结论 |
|------|------|----------|
| 1. 上下文逻辑连贯性 | 通过 | schema v3 基线、v4 增量、service ownership 连贯 |
| 2. 内容空洞排查 | 通过 | migration、CAS、bounds、safe fields 均有可执行场景 |
| 3. 歧义点识别 | 通过 | live fact 与 durable hint、foundation 与 workflow 分离 |
| 4. 大模型语义偏差 | 通过 | 未把现有表/repository 误写为缺失 |
| 5. SDD/TDD 适配 | 通过 | baseline RED、migration/repository RED、GREEN 和 evidence 明确 |
| 6. 最小化实现 | 通过 | 追加 v4；不拆 storage、不引入 repository framework |
| 7. 向下兼容隐患 | 通过 | v1-v3 不重写；无 downgrade 或双写 fallback |
| 8. 存量业务影响 | 通过 | app/CLI direct-storage 删除和 daemon service 接线已列出 |
| 9. 功能失效风险 | 通过 | rollback、stale revision、PID hint、非法 phase 已覆盖 |
| 10. 落地可行性 | 通过 | 基于现有 schema v3、actor owner 和 async client |
| 11. 任务拆解与排期 | 通过 | storage 单 owner 串行；proto/runtime/client/app/CLI 分 owner |
| 12. 可扩展性影响 | 通过 | versioned snapshot 和 receipt 支撑后续 owner |
| 13. 过度设计警惕 | 通过 | metadata foundation 有明确字段和后续非目标 |
| 14. 小而高效改动 | 通过 | 复用现有 parent、host、account、handoff 和 usage facts |
| 15. 代码优雅性 | 通过 | typed repository/CAS，wire 不暴露 SQL/Storage |
| 16. 架构统一性 | 通过 | runtime daemon 独占 production SQLite，app/CLI 经 client |

总评：16/16 通过，0 个未解决阻断项。

## 已修复问题

1. **P0：与 T-102 的 effective-config storage ownership 冲突。**
   - 修复：`homie-storage/src/lib.rs` 全程仅由 `S103-storage-impl` 修改。
   - 无环 DAG：
     `T-102 G3 -> S103-GREEN-02 -> T-102 G5 -> T-102 shared release -> T-103 integration`。
2. **P1：PRD/plan 状态提前声称 ready/reviewed。**
   - 修复：统一为 `ready_for_review`，等待用户批准。
3. **P1：cross-change 依赖只存在 PRD，未进入 executable tasks。**
   - 修复：`S103-GREEN-02` 已机械依赖 T-102 G3 contract handoff，并要求发布 repository
     GREEN handoff。

## 风险与门禁

| 风险 | 严重度 | 门禁 |
|------|--------|------|
| v4 migration 半提交 | 高 | real v3 fixture + injected rollback + version row 原子性 |
| app/CLI 双路径绕回 SQLite | 高 | normal dependency tree/source scan；禁止 fallback |
| persisted PID/status 被当作 live proof | 高 | T-102 holder/process verification 后才发布 running |
| foundation 被误报为 parity closure | 中 | remote/usage/update/UI rows 保持 partial |

## 验证记录

- `openspec status --change diri-storage-core-facts`: 4/4 complete。
- `openspec validate diri-storage-core-facts --strict`: valid。
- `cargo test -p homie-storage --tests`: 全部通过。
- `git diff --check`: pass。
- 产品代码改动：无。

## 结论

规格质量门禁通过，可提交用户审阅。用户批准前不得启动 RED/产品实现 TraeCLI。
