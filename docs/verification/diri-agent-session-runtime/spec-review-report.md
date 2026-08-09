# Spec Review Report: diri-agent-session-runtime

```yaml
change_id: diri-agent-session-runtime
bead: homie-t3u.1
review_date: 2026-08-09
checkpoint_commit: 48f522b
result: pass
implementation_authorized: false
```

## 评审概览

| 维度 | 判定 | 关键结论 |
|------|------|----------|
| 1. 上下文逻辑连贯性 | 通过 | 2 RED 根因、holder-first reconciliation 与任务链一致 |
| 2. 内容空洞排查 | 通过 | timeout、状态矩阵、cleanup 和 evidence 均可验证 |
| 3. 歧义点识别 | 通过 | live/behavior/durable authority 与 local/remote 边界明确 |
| 4. 大模型语义偏差 | 通过 | 明确 2 RED + 1 retained GREEN，禁止沿用旧 3-failure 结论 |
| 5. SDD/TDD 适配 | 通过 | RED -> GREEN -> REFACTOR -> EVIDENCE 顺序固定 |
| 6. 最小化实现 | 通过 | 保留 Wave 1A transport 和 per-session holder |
| 7. 向下兼容隐患 | 通过 | 删除错误 fallback，不增加 compatibility layer |
| 8. 存量业务影响 | 通过 | runtime/agent/proto/client/shared-file 顺序已列出 |
| 9. 功能失效风险 | 通过 | PID reuse、holder race、panic leak、shutdown race 已覆盖 |
| 10. 落地可行性 | 通过 | 现有 holder、manifest、reducer 和 daemon 边界可复用 |
| 11. 任务拆解与排期 | 通过 | 单任务 owner、依赖、预算、命令与 cleanup 明确 |
| 12. 可扩展性影响 | 通过 | 冻结 launch contract，不提前实现 remote migration |
| 13. 过度设计警惕 | 通过 | 不引入 shared holder manager、RPC framework 或插件层 |
| 14. 小而高效改动 | 通过 | 先关闭 adoption/PTY 根因，再接 manifest/status/resource |
| 15. 代码优雅性 | 通过 | reconciliation、launch、status、governor、recovery 分责 |
| 16. 架构统一性 | 通过 | daemon/actor/holder/storage 权威边界保持一致 |

总评：16/16 通过，0 个未解决阻断项。

## 已修复问题

1. **P0：T-102/T-103 同时拥有 `homie-storage/src/lib.rs`。**
   - 修复：T-103 成为 schema/repository 唯一 owner。
   - 最终 DAG：`T-102 G3 -> T-103 S103-GREEN-02 -> T-102 G5`。
2. **P1：production manifest package closure 不明确。**
   - 修复：根目录 `assets/agent-descriptors/*.json` 作为 canonical input，通过显式
     `include_str!` 表编译进 immutable catalog；packaged runtime 不依赖 cwd 或外部资源。
3. **P1：技术栈和测试文件现状不准确。**
   - 修复：使用现有 `rustix-openpty`；不存在的 `tests/process_tree.rs` 标为新建。
4. **P1：Bead acceptance 仍写 3 个失败。**
   - 修复：`homie-t3u.1` 已同步为 2 RED + 1 retained GREEN。

## 风险与门禁

| 风险 | 严重度 | 门禁 |
|------|--------|------|
| RED assertion 失败遗留 holder | 高 | panic-safe PID/start-time ledger；after-minus-before 为空 |
| T-103 repository handoff 延迟 | 高 | G5 不启动；禁止 T-102 临时写 storage |
| holder Stat 后进程立即退出 | 中 | 后续 IPC/process signal 重新 reconcile，input fail closed |
| scope 滑入 remote migration | 中 | 无 remote method/capability；RT-010 保持 partial |

## 验证记录

- `openspec status --change diri-agent-session-runtime`: 4/4 complete。
- `openspec validate diri-agent-session-runtime --strict`: valid。
- `git diff --check`: pass。
- 当前 runtime 实测：14 tests，12 passed，2 failed。
- 产品代码改动：无。

## 结论

规格质量门禁通过，可提交用户审阅。用户批准前不得启动 RED/产品实现 TraeCLI。
