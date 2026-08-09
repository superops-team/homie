# Spec Review Report

```yaml
change_id: diri-virtual-key-credentials
beads: homie-e1s
dev_loop_step: 1
source_prd: prd-spec/features/diri-virtual-key-credentials/2026-08-07-diri-virtual-key-credentials-design.md
status: pass_after_revision_plan
reviewed_at: 2026-08-07
```

## 1. 总体结论

- 可行性：高。
- 最大风险：只实现 `homie-llm` 的 virtual key 行为而没有把 raw provider key 禁止传播写入长期组件规格，会导致后续 remote/MCP/agent lane 在各自实现中重新发明 credential 传递方式。
- 推荐方向：先把 `specs/virtual-key-credentials/README.md` 升级为 L0 security gate，再在 `homie-llm` 用最小公共 API 和 contract tests 固化第一阶段行为；不在本 lane 扩大到完整 LLM proxy、secret envelope 或 remote node。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P0 | 安全边界 | 当前组件 spec 虽写了 raw provider key 不进 agent env/config/log/event/context/memory/metrics/report，但缺少 remote/MCP/LLM/agent 的强制 gate 表 | 后续 lane 可能只引用本组件名称，而没有统一禁止传播规则 | 在 `specs/virtual-key-credentials/README.md` 增加 cross-spec mandatory gates 和 raw-key forbidden destinations |
| P0 | 可验证性 | 现有 `homie-llm` 测试覆盖 virtual key issue/expiry/revoke/model scope，但没有覆盖 agent-visible config 和 raw-key propagation denial | 无法证明 managed agent、remote/MCP payload 不泄漏 provider key | 新增 contract tests：managed config 序列化不含 raw key；remote/MCP/agent/log destination 拒绝 raw key |
| P1 | Diri/Homie 适配 | Diri 的 provider account/node protocol 和 Homie virtual-key proxy 是不同架构，PRD 若只说复刻 Diri 会产生误导 | 机械迁移 Diri provider.call 可能绕过 Homie 本机 credential custody | PRD 和 spec 增加 “Diri behavior parity / Homie credential adaptation” 双栏 |
| P1 | SDD/TDD 拆解 | 如果一次性把 secret envelope、HTTP proxy、usage accounting、remote handoff 都纳入，会超过 foundation-security 第一阶段写入范围 | 触碰 `homie-remote`、MCP、storage migration 等非本 lane 文件，和用户限定范围冲突 | OpenSpec 只拆第一阶段 contract 和 `homie-llm` tests，实现外延标为后续 lane |
| P1 | 运行风险 | 错误和 Debug 输出若回显输入 payload，测试 fake raw key 也可能进入报告或日志 | 安全测试自身制造泄漏样例 | 错误类型只返回目的地和错误类别，不包含 secret 值；测试断言序列化和错误字符串均不含 fake key |
| P2 | 依赖策略 | 为 secret redaction 引入新依赖会增加供应链面 | 本阶段不需要复杂 regex 或 crypto | 使用现有类型与明确 allow/deny contract，不新增依赖 |

## 3. 整改后的完善方案

目标与范围：

- 将 `virtual-key-credentials` 固化为 L0 security foundation，阻塞 `llm-proxy`、`remote-node-handoff`、`mcp-automation`、`observability` 和 agent adapter 的 credential 行为。
- 第一阶段只做规格和最小 `homie-llm` contract，不实现完整 proxy 或 remote。

设计原则：

- Raw provider key 只能存在于本机 secret custody 与短期 provider resolver 内存中。
- Managed agent 可见面只允许 local proxy URL、virtual key、expiry、scope metadata。
- Remote node 和 MCP 自动化默认 secretless；如 remote node 需要 provider login，只能走 node-local user explicit login，不从 Homie 复制 raw key。
- 所有错误、事件和报告记录 key id、scope id、目的地和错误码，不记录 secret 值。

核心方案：

- 更新组件 spec：增加 Diri/Homie adaptation、cross-spec gates、raw-key forbidden matrix、first-stage test mapping。
- 更新 `homie-llm`：保留现有 `InMemoryVirtualKeyStore`，新增 agent-visible managed config 和 raw-key propagation denial API。
- 新增测试：issue/revoke/expired/scope/model denied 继续覆盖；新增 no-raw-key-to-remote/MCP/agent/log/event contract tests。

兼容与风险控制：

- 仓库规则默认不保留旧兼容；本阶段没有旧 API 迁移。
- 不引入新依赖，不触碰 remote/MCP 文件。
- 若后续发现 remote/MCP spec 也必须同步修改，本 lane final 只标注 follow-up。

验收标准：

- P0/P1 问题在 spec、OpenSpec 和 tests 中均有落点。
- `cargo test -p homie-llm` 通过。
- 验证报告不得把未运行命令写成 pass。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 补强 `virtual-key-credentials` 长期合同 | `specs/virtual-key-credentials/README.md` | PRD | P0 |
| Functional cases | 设计可执行安全用例 | `docs/verification/diri-virtual-key-credentials/functional-cases.md` | PRD/spec review | P0 |
| OpenSpec | 拆 plan/tasks/alignment | `openspec/changes/diri-virtual-key-credentials/*` | functional cases | P0 |
| TDD | 补 contract tests | `crates/homie-llm/tests/virtual_key.rs` | OpenSpec | P0 |
| Implementation | 补最小 contract API | `crates/homie-llm/src/lib.rs` | RED tests | P0 |
| Verification | 跑 focused tests 与质量门禁 | verification reports | implementation | P0 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| Unit/contract | virtual key 生命周期 | issue、validate、revoke、expired、unknown | RED/GREEN |
| Unit/contract | scope 权限 | wrong session、wrong profile、wrong provider、wrong model | RED/GREEN |
| Security contract | no raw key propagation | remote node、MCP result、agent config、log/event destination 拒绝 raw key | RED/GREEN |
| Serialization | managed config secretless | JSON 不含 fake raw provider key、secret ref、Authorization | GREEN |
| Quality gate | workspace 编译和格式 | `cargo fmt --all -- --check`, `cargo check --workspace`, `git diff --check` | verification |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| Step 1 | 1 | PRD/spec review | 范围过大则退回非目标 | 本报告 |
| Step 2 | 2 | 功能验证 Case | 每个 P0 至少一个 Case | `functional-cases.md` |
| Step 3-4 | 3 | OpenSpec plan/tasks/alignment | 防止任务无 Case | OpenSpec 三文件 |
| Step 5-6 | 4 | TDD 实现 | 严禁削弱测试 | `homie-llm` tests/code |
| Step 7-10 | 5 | Case 执行、review、E2E/quality gate | 环境失败必须写 blocked/not_run | verification reports |

## 7. 待确认问题

- 后续 `remote-node-handoff` 和 `mcp-automation` lane 是否要各自补一份引用本 security gate 的 spec patch；本 lane 按用户要求不编辑这些文件。
- Secret envelope 最终使用 `age` 还是 RustCrypto primitives；本阶段只保留后续决策入口。

