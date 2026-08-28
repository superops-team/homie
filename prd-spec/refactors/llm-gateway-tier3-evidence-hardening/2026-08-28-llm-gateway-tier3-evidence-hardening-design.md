# LLM Gateway Tier-3 证据补齐（failure model + 变异 + 对抗）设计文档

## 1. 背景与问题

### 1.1 来源

本次变更由 `docs/ai-coding-workflow-analysis.md` 的 GAP-1 触发：

> 四个安全敏感的 LLM-gateway 变更——`llm-gateway-virtual-keys`、
> `llm-gateway-policy-quota`、`llm-gateway-credential-login`、
> `llm-gateway-model-routing`——正是 `docs/development/standards.md` §6.3
> 点名要求按 **Tier 3** 处理的「credential custody / virtual key issuance /
> LLM proxying / concurrency」领域，但每个目录仅含
> `release-readiness-report.md` 与 `spec-review-report.md` 两份证据，
> **均无 `failure-model.md`**。全仓库 66 个证据目录中仅 2 个含
> `failure-model.md`（`engine-session-runtime-split`、
> `process-tree-governor-verification`）。

### 1.2 规则缺口

`docs/development/standards.md` §6.3 对 Tier 3 的硬性要求是：

1. 先写 **failure model**，列出该改动可能造成的具体伤害方式；
2. 为每种伤害方式增加一个能真正捕获它的验证层；
3. 完整循环 + 属性测试 + **变异测试** + 一轮 **adversarial pass**；
4. 未覆盖的 failure mode 必须作为 known limits 记入 EVIDENCE。

上述四个 gateway 变更已合入 `main` 并打 tag，但证据层未满足该要求。
**代码本身不在此次变更中改动**——本次只补齐「证明代码安全」的证据，
并把安全门禁从「写在规范里」变成「落盘可核查」。

### 1.3 为什么合并成一个 change_id

四个变更共用同一个安全面：`homie-gateway` 库 crate（虚拟密钥、上游转发、
模型路由、策略/配额、凭据来源），且都已关闭。为四个已关闭的 change_id
分别重建薄证据会产生碎片化文档，不如在单一安全面下产出一份**合并的
failure model** 与一组覆盖四个变更的验证用例。此次使用新 change_id
`llm-gateway-tier3-evidence-hardening`，证据统一落在
`docs/verification/llm-gateway-tier3-evidence-hardening/`。

## 2. 目标

1. 为 `homie-gateway` 安全关键路径产出一份可执行的 **failure model**，
   覆盖：凭据泄漏、虚拟密钥误发/重放、上游 key 泄露、未授权转发、
   模型重写绕过、策略/配额绕过、并发竞态、部分写入/数据丢失、恶意输入。
2. 对每条 failure mode 对应一个能真正捕获它的验证层（race/stress、
   契约测试、负向输入、日志/指标断言等），并在 evidence 中记录。
3. 对 `homie-gateway` crate 运行 **变更行覆盖率 fail-under**（工具可用则用，
   否则手动逐行核对）与 **变异测试**（工具可用则用，否则手动引入 3–5 个
   真实 bug 并逐个杀死）。
4. 运行一轮 **adversarial pass**：显式用恶意/畸形输入攻击自己的实现
   （畸形 JSON、超长 body、伪造 header、并发配额绕过、重启后密钥状态等）。
5. 把 failure model 的权威形态沉淀进 `specs/llm-gateway.md` §10 的
   Security And Recovery，作为长期契约；证据仍按 convention 落在
   `docs/verification/`。

## 3. 非目标

- **不改动任何生产代码**（`homie/crates/homie-gateway` 及依赖其的
  `homie-engine` 保持不变），除非验证过程中发现真实缺陷——此时缺陷另立
  bugfix change_id 修复，不在本 PRD 内顺手改代码。
- 不新增任何 Rust 依赖 / 工具链（不引入 llvm-cov、tarpaulin、
  cargo-mutants、proptest 等）；「工具可用则用、否则手动 fallback」。
- 不重建四个已关闭变更的完整 OpenSpec 文档；只补 Tier-3 证据。
- 不引入 CI 强制门禁（enforcement gate）；「如何机械阻止 Tier-3 变更
  在无 failure-model 时关闭」作为独立后续变更，不在本 PRD 内实现。

## 4. 用户场景

- **场景 A（审计者）**：一名未来审计者要确认「LLM gateway 的凭据与虚拟
  密钥路径有 Tier-3 级别的证据」。他打开
  `docs/verification/llm-gateway-tier3-evidence-hardening/failure-model.md`，
  能看到每条伤害方式及其捕获层与证据编号。
- **场景 B（维护者）**：某人要改 `homie-gateway` 的 quota / rate-limit /
  credential 逻辑。他先读 failure model 了解已知风险，再跑同一套对抗与
  变异脚本做回归护栏。
- **场景 C（发布决策者）**：发布前需要一条可复现入口命令，重跑所有
  gateway 安全层并看到真实数字（非形容词），据以放行。

## 5. 需求

### R1 — 合并 failure model

`docs/verification/llm-gateway-tier3-evidence-hardening/failure-model.md`
必须列出 `homie-gateway` 安全关键路径的伤害方式与对应捕获层，至少覆盖：

| 伤害方式 | 捕获层 |
|---|---|
| 上游 key / master key / 原始虚拟密钥泄露（日志、错误体、审计表、agent env） | 负向 grep + 测试断言 sanitized body / masked logs |
| 虚拟密钥重放 / 未授权转发 | 401 契约测试 + 手动 adversarial |
| 模型重写被畸形 body 绕过 | 非 JSON / 缺失 / 非字符串 `model` 的 pass-through 契约测试 |
| 策略/配额被绕过（`0` 语义、并发、跨自然日边界） | 单元 + race/stress + 边界（00:00 翻转）测试 |
| 并发竞态（rate-limit 滑动窗口、配额聚合、密钥创建） | race/stress 测试 |
| 部分写入 / 数据丢失（SQLite usage/audit） | 事务与恢复断言 + 重启回放 |
| 恶意输入（超长 body、超大数字、畸形 JSON） | fuzz 或手动 adversarial pass |
| 重启后密钥/配额状态不一致 | 持久化 + 重启回放测试 |

每条 failure mode 必须标明：预期行为、失败时如何被捕获、对应证据文件编号。

### R2 — 对抗与变异验证

- **adversarial pass**：对 `homie-gateway` 以恶意/畸形输入显式攻击自己的
  实现，记录到 `functional-cases.md` 与对应 `fc-*.log`。
- **变更行覆盖率**：工具可用则 `cargo llvm-cov --fail-under-lines <n>`；
  否则手动逐行核对每个新增/变更行被某个测试执行，并在报告中写明
  「tool-based coverage skipped + 原因」。
- **变异测试**：`cargo-mutants` 可用则用并分类 survivor；否则手动引入
  3–5 个真实 bug（翻转比较、off-by-one、删条件、提前 return），测试套件
  必须逐个杀死后恢复。

### R3 — 长期契约沉淀

`specs/llm-gateway.md` §10（Security And Recovery）新增一段「Failure Model」，
把 R1 的伤害方式与捕获层固化为长期契约，供后续变更引用。

### R4 — 证据落盘

`docs/verification/llm-gateway-tier3-evidence-hardening/` 至少包含：

- `failure-model.md`
- `functional-cases.md`
- `functional-verification-report.md`
- `release-readiness-report.md`
- `code-review-round-1.md`（如审查发现真实缺陷，需 round-2）
- `fc-<n>-*.log`（变异、覆盖率、对抗、race/stress、契约测试等实际日志）

## 6. 受影响 specs

| 文档 | 改动 |
|---|---|
| `specs/llm-gateway.md` | §10 新增 Failure Model 段（R3） |
| `docs/verification/llm-gateway-tier3-evidence-hardening/` | 新增全部证据（R1/R2/R4） |

不触碰 `homie/crates`、`Sources`、`Tests`（除非发现真实缺陷另立 bugfix）。

## 7. 测试计划

本 PRD 是证据/契约补齐，不新增生产代码。验证方式：

- FC-01：`failure-model.md` 覆盖 R1 表格全部伤害方式并逐条标注捕获层。
- FC-02：adversarial / 变异 / 覆盖率证据存在且给出真实数字（非形容词）。
- FC-03：`specs/llm-gateway.md` §10 含 Failure Model 段。
- FC-04：`git diff --check` 与 `git status --short` 通过，无脏文件。
- FC-05：未改动任何生产代码（`git diff --stat` 不含
  `homie/crates/homie-gateway` 的 .rs 文件）。

## 8. 验收标准

1. R1–R4 全部落地；证据目录包含 R4 列出的文件。
2. 每条 failure mode 有对应捕获层证据；跳过的层显式写明原因。
3. 变异测试杀死全部手工 mutant；覆盖率满足 fail-under 或写明手动核对结果。
4. Beads issue 仅在证据匹配交付状态后关闭。

## 9. Beads 追踪

- change_id: `llm-gateway-tier3-evidence-hardening`
- 类型: refactor（验证/证据硬化）
- 优先级: P1（安全敏感代码缺失 Tier-3 证据；非运行时回归）
- 来源: `docs/ai-coding-workflow-analysis.md` GAP-1
