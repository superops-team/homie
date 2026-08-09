# T-102 16 维规格评审报告

```yaml
change_id: diri-agent-session-runtime
review_date: 2026-08-09
review_scope: PRD + OpenSpec + delegation + two long-lived component specs
result: pass
blocking_findings: 0
```

## 评审结论

| # | 维度 | 结果 | 评审证据 |
|---|------|------|----------|
| 1 | 逻辑连贯性 | PASS | 从 2 RED 根因到 holder-first reconciliation、manifest spawn、status/resource/recovery 形成单一路径 |
| 2 | 内容完整性 | PASS | 背景、目标、非目标、场景、11 个 FR、4 capabilities、plan/tasks/evidence 齐全 |
| 3 | 歧义 | PASS | live authority、behavior authority、hibernate/archive、local/remote migration 均有明确边界 |
| 4 | 大模型语义偏差 | PASS | 明确禁止把 holder stat GREEN 写成 RED、storage row 当 running、shell fallback、remote placeholder |
| 5 | SDD/TDD 适配 | PASS | tasks 严格 RED -> GREEN -> REFACTOR -> EVIDENCE，现有 RED 不得弱化 |
| 6 | 最小化实现 | PASS | 保留 Wave 1A transport 和 per-session holder；不强制 shared holder manager |
| 7 | 向下兼容 | PASS | 不增加 compatibility layer；holder protocol 只允许 additive behavior 以保持 live adoption |
| 8 | 存量影响 | PASS | 列出 runtime/agents/proto/client/storage/tests；shared actor/lib 串行 ownership |
| 9 | 风险预判 | PASS | 覆盖 probe race、PID reuse、secret leak、STOP memory trade-off、resume failure、process leak |
| 10 | 可行性 | PASS | 所需 manifest/reducer/holder/storage tables 已存在；schema 不足设为显式 stop condition |
| 11 | 任务拆解与排期 | PASS | 21 个单 TraeCLI item，依赖、owner、预算、命令、cleanup 明确 |
| 12 | 可扩展性 | PASS | frozen config、structured plan、canonical signal 为后续 agent/remote 提供稳定输入，但不预做 remote |
| 13 | 过度设计 | PASS | 不引入 shared manager、新 RPC framework、generic plugin system 或 schema migration |
| 14 | 小而高效改动 | PASS | 先关闭两个现有 RED，再按独立 vertical slices 接入 agent/status/resource/recovery |
| 15 | 代码优雅性 | PASS | 新模块按 reconciliation/launch/status/governor/recovery 分责；删除重复旧路径 |
| 16 | 架构统一性 | PASS | 保持 daemon sole owner、single actor、bounded lane、holder PTY owner、storage durable projection |

## 关键一致性检查

- Current baseline 始终是 `14 tests: 12 passed, 2 failed`。
- RED 只有：
  - `runtime_reopen_can_adopt_holder_and_continue_session`
  - `runtime_spawn_shell_uses_live_pty`
- `runtime_holder_stat_tracks_resize_and_log_offsets` 始终作为保持 GREEN 门禁。
- 根因始终是 bulk detach 先于 adoption，并导致 registry/storage/projection 分裂。
- holder live evidence 证明 liveness；reducer 证明 live behavior；storage row 单独不证明 running。
- T-102 只交付 local migration substrate；remote `session.migrate`/handoff 不发布，RT-010 保持 partial。
- UI、remote node、provider proxy/virtual-key issuance 均未提前承诺。
- 真实 daemon/holder/PTY process E2E 和 exact fixture cleanup 是 blocking gate。
- 已记录失败测试曾泄漏临时 holder PID `87051`（用户已手工终止，packaged holder 未触碰）；
  RED/panic/timeout/success 都要求 panic-safe guard 和 holder PID+start-time 前后集合差门禁。

## 设计取舍复核

### 保留 per-session holder

规格要求行为 parity，而非复制 Diri 的进程拓扑。当前 holder 已拥有 PTY/output 并通过
stat/resize/log-offset 测试；先关闭 reconciliation 和 lifecycle 缺口比引入 shared manager
更小、更可验证。若 race evidence 证明不足，必须回规格评审。

### 不新增 storage schema

T-102 使用既有 effective-config/session tables 和 runtime-owned sanitized launch record。
若无法原子关联，implementation 必须 blocked，交由 storage owner 修订规格，不允许隐藏
migration。

### Hibernate 使用 STOP/CONT

这保留真实 PTY/process continuity；不声称释放 resident memory。Governor 只对
idle+unattached+unpinned session 自动执行，并保护 running/needs-input。

## 未阻断风险

| Risk | Control |
|------|---------|
| holder Stat 后立即退出 | normal process signal/status reconciliation，input fail closed |
| manifest version upgrade | frozen launch/resume facts；missing/incompatible fail closed |
| platform footprint sample unavailable | safe unknown，不 exit/kill |
| assertion panic 泄漏 holder | PID+start-time ledger、Drop、bounded terminate/kill/reap |

## 评审决定

规格可进入 OpenSpec strict/status 和文档一致性校验。实施必须从 task 1.1 开始，且在任何
storage schema、Wave 1A wire、shared holder manager、credential、remote 或 UI 扩展出现时
重新评审。

## 最终验证记录

| Check | Result |
|-------|--------|
| `openspec status --change diri-agent-session-runtime` | PASS，4/4 artifacts complete |
| `openspec validate diri-agent-session-runtime --strict` | PASS，change is valid |
| parsed deltas | PASS，29 requirements / 75 scenarios |
| PRD coverage | PASS，11 FR |
| task checklist | PASS，26 items，其中 4 个 specification gates 已完成 |
| `git diff --check` on allowed paths | PASS |
| invisible marker / unresolved marker scan | PASS |
| product-code diff from S102 | PASS，无 `crates/`/`apps/`/Cargo/Makefile 修改 |

本轮未运行 product tests；当前 12/14 结果来自前述 checkpoint 实测，implementation RED
仍留给 task 1.1 起始执行。
