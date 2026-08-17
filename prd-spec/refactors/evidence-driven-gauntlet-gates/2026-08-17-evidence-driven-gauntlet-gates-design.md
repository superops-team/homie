# 引入证据驱动开发链路约束（Evidence-Driven Gauntlet Gates）设计文档

## 1. 概述

### 1.1 问题/动机

Homie 现有开发链路（`AGENTS.md` 的 Required Development Workflow）在「文档驱动 +
证据落盘」维度已经很强：PRD → specs → OpenSpec → verification evidence → Beads，
层层可追责，`docs/development/quality-gates.md` 也按域给出了最小证据清单。

但对照 `old-coder` 的「证据驱动开发链路」标准评审后，发现现有链路在**证据可信度**
上存在缺口——它规定了「运行哪些命令 + 记录哪些文件」，却没有写死「什么条件算失败」，
也没有把「测试本身是否可信」的验证手段（先观察失败、变异测试、覆盖率 fail-under、
checker 负向对照）纳入规范。当前链路能保证"流程走了"，不能保证"证据是真的"。

本次评审识别出的核心缺口（G1–G5）：

- **G1 门禁缺失败条件**：每条门禁只有命令，没有"退出码/阈值即失败"的硬约束；home-grown
  checker 没有 fail-closed 与负向对照要求。
- **G2 RED 缺失**：仅写"SDD/TDD"，未展开"每个测试先运行并观察失败、避免 import error
  式弱失败、立即通过则用一次性变异证明其非空"。
- **G3 覆盖率 fail-under 与变异测试缺失**：对 credential custody / virtual key /
  LLM proxy / orchestration 等安全敏感代码，只有"写测试"要求，没有"测试本身可信"的验证。
- **G4 反游戏化规则与 Tier 3 failure model 缺失**：没有"不弱化测试 / 不同步改测试与实现 /
  不 mock 被测单元 / 不追覆盖率 / 不报未运行层 / gauntlet 失败即 blocked"的硬规则，
  也没有高风险改动的 failure model + adversarial pass。
- **G5 EVIDENCE 缺实际数字 + fresh run + 可复现入口**：有日志文件和稳定命名，但没有
  "粘贴实际数字而非形容词""最后一次 fresh run 才可报告""一条入口命令可复跑所有层"的强制。

### 1.2 目标

1. 把 old-coder 证据驱动链路的**原则**（而非特定工具）引入 Homie 的开发规范，补齐 G1–G5。
2. 采用「工具可用则用、否则手动 fallback」的写法，与 Homie「不引入预防性配置 / 不新增未验证
   依赖」的仓库规则一致，不强制引入 llvm-cov / tarpaulin / cargo-mutants / proptest 等工具。
3. 把 TDD 从一句话展开为 RED→GREEN→REFACTOR 循环，并写死反游戏化规则。
4. 为安全敏感域（credential / virtual key / proxy / orchestration / 并发 / 数据丢失）
   引入 Tier 3 的 failure model + adversarial pass 要求。

### 1.3 非目标

- **不在本 PRD 中新增任何 Rust 依赖或工具链**（不引入 llvm-cov、tarpaulin、cargo-mutants、
  proptest、pytest-randomly 等）；文档只描述"如可用则用，否则手动 fallback"。
- 不修改任何生产代码、`specs/` 长期契约、`homie/crates`、`Sources`、`Tests`。
- 不改变已有质量门禁的既有命令（`cargo fmt --check`、`cargo check`、`cargo test` 等保持不变），
  只在其上叠加失败条件与证据要求。
- 不一次性把 Homie 改造成严格的 old-coder 全套 Tier 3 流程；本次只落地原则性约束，
  具体工具选型留待后续独立 PRD。

### 1.4 受影响文档范围

| 文档 | 改动 |
|------|------|
| `docs/development/standards.md` | 第 6 节 Testing Standards 展开 TDD/RED/REFACTOR、Tier 分层、反游戏化规则、Tier 3 failure model |
| `docs/development/quality-gates.md` | 每条门禁补失败条件；新增覆盖率 / 变异 / suite health 门禁；新增 checker fail-closed + 负向对照；EVIDENCE 硬性字段 |
| `AGENTS.md` | Required Workflow 第 8 步与 Implementation Guidance 小幅引用 standards 的 TDD 规则 |

### 1.5 与存量文档的关系

| 存量文档 | 关系 |
|----------|------|
| `docs/development/quality-gates.md` | 本次增强的主落点之一，在既有分域门禁上叠加失败条件与证据要求 |
| `docs/development/standards.md` | 本次增强的主落点之二，展开 Testing Standards |
| `AGENTS.md` | 顶层权威，只做引用性小幅更新 |
| `specs/*.md` | 本次不触碰；本 PRD 属于开发流程文档，不改变任何模块边界 / 协议 / 数据模型契约 |

## 2. 需求

### R1 — TDD 循环展开（修复 G2）

`standards.md` 的 Testing Standards 必须从"SDD/TDD"一句话展开为：

- 每新增一个测试，先运行并**观察其失败**（RED），失败原因必须是断言失败，而不是 import /
  compile 错误；模块尚不存在时用 stub 抛错（如 `unimplemented!()`）。
- 新测试若立即通过，必须证明其非空：用一次性变异（throwaway mutant）打破实现观察测试变红，
  然后恢复，并记录为"既存行为作为回归护栏"。
- 最小实现使失败测试通过（GREEN），并跑全量测试而非只跑新测试。
- 全量通过后重构（REFACTOR），行为断言冻结：实现重构不碰测试文件；断言改动属于行为变更，
  必须回到 spec。

### R2 — 反游戏化硬规则（修复 G4 的一部分）

`standards.md` 必须写死以下硬规则（不可违反）：

1. 不弱化测试使其通过（不放宽断言、不加 skip、不抬高容差、不删失败测试）。
2. 不在同一步骤里同时改测试与实现来达成绿灯；先改一个、运行、再改另一个。
3. 不 mock 被测单元本身，也不 mock 到只剩 mocks 被验证；mock 边界（网络/时钟/文件系统），不 mock 逻辑。
4. 不追逐覆盖率数字；只为触碰行而加的无意义断言是作弊，变异测试正是用来抓这个。
5. 不报告未运行的层；跳过的层必须说明原因。
6. gauntlet 失败即 blocked，在失败未被解决前不算完成。

### R3 — 门禁失败条件与 checker fail-closed（修复 G1）

`quality-gates.md` 的每一条门禁必须明确"什么退出码 / 阈值算失败"，并新增两条通用规则：

- home-grown checker（grep gate、自定义脚本、手动变异 runner）必须 **fail-closed**：崩溃、
  不可读输入、意外退出码、gate 内部静默跳过某项，都是该层的硬失败，绝不 `|| true`、`2>/dev/null`
  或裸 fallthrough。
- home-grown checker 必须做**负向对照**证明它能失败：用一个已知坏输入运行并观察它失败（RED 原理
  应用于 checker 本身），并把对照记录进证据。负向对照只证明"一个已知坏样本触达失败路径"，
  不证明 checker 识别全部违规；当 gate 覆盖面窄于其声称的规则时，在规则处写清楚。

### R4 — 覆盖率与变异测试（修复 G3）

`quality-gates.md` 新增门禁，采用工具可用则用、否则手动 fallback：

- **变更行覆盖率 fail-under**：覆盖率约束的是"变更/新增行是否被执行"，不是全局百分比。
  该层必须在阈值未达时非零退出（`cargo llvm-cov --fail-under-lines <n>` / `cargo tarpaulin
  --fail-under <n>`，或等价）；只打印百分比且退出 0 的是报告不是门禁。
- **变异测试**：有 `cargo-mutants` 则用工具；否则手动变异——对新增代码一次引入 3–5 个真实 bug
  （翻转比较、边界 off-by-one、删条件、提前 return），测试套件必须逐个杀死，之后恢复。
- 这两个门禁对安全敏感域（credential custody / virtual key / LLM proxy / orchestration /
  并发 / 数据丢失）为**强制**；对普通改动可按 Tier 分层。

### R5 — Tier 分层与 Tier 3 failure model（修复 G4 的另一部分）

`standards.md` 引入按爆炸半径分层的 effort 校准：

- **Tier 1 trivial**（typo / 注释 / 配置值）：全量测试 + lint；不强制新测试，但说明为何不可测或已覆盖。
- **Tier 2 normal**（bug 修复 / 小特性）：完整循环；bug 修复必须先写一个能复现 bug 的 RED 测试。
- **Tier 3 high-stakes**（money / auth / 数据丢失 / 并发 / 公开 API / 密钥与凭据）：
  先写 **failure model**——列出该改动的具体伤害方式（竞态、部分写入、恶意输入、溢出、无界增长、
  回滚失败等），并为每种方式加一个能真正捕获它的层（并发用 race/stress 测试、解析器用 fuzzing、
  迁移用回滚演练、延迟预算用 benchmark、公开库用 API 兼容检查、服务边界用契约测试、静默生产失败用
  日志/指标断言）。随后：完整循环 + 属性测试 + 变异测试 + 一轮 adversarial pass（显式尝试用恶意输入
  破坏自己的实现）。未覆盖的 failure mode 必须作为 known limits 记入 EVIDENCE。

### R6 — EVIDENCE 硬性字段（修复 G5）

`quality-gates.md` 的证据报告必须满足：

- 每个 behavior 映射到验证它的测试 / 门禁层 / 或"显式跳过+原因"，绝不静默缺席。
- 每层给出**实际运行的命令与真实数字**（粘贴结果，不用形容词），例如"47/47 测试通过，
  变更行覆盖率 100%（31/31 行），5/5 手动变异杀死"。
- 所有数字来自**最后一次 fresh run**（最后一次代码编辑之后），mid-task 的陈旧结果不得报告。
- 报告必须**从仓库本身可复现**：引用的每条命令（含变异脚本）必须作为仓库内持久文件存在；
  记录工具版本或锁定版本；提供一条入口命令可复跑所有层；标识源状态（commit SHA 或树哈希）。
- 明确列出跳过的层及原因；诚实记录失败与如何解决。

## 3. 测试计划

本 PRD 是开发流程文档改动，不涉及 Rust/Swift 测试代码。验证方式为文档一致性检查：

- FC-01：`standards.md` 第 6 节包含 R1（RED→GREEN→REFACTOR）与 R2（反游戏化六条）。
- FC-02：`standards.md` 第 6 节包含 R5（Tier 1/2/3 分层与 Tier 3 failure model）。
- FC-03：`quality-gates.md` 每条门禁含失败条件，并含 R3（fail-closed + 负向对照）通用规则。
- FC-04：`quality-gates.md` 含 R4（覆盖率 fail-under + 变异测试）门禁。
- FC-05：`quality-gates.md` 含 R6（EVIDENCE 硬性字段：实际数字 / fresh run / 可复现入口 / 跳过原因）。
- FC-06：`AGENTS.md` 第 8 步与 Implementation Guidance 引用了 standards 的 TDD 规则。
- FC-07：`git diff --check` 与 `git status --short` 通过，无残留脏文件。
- FC-08：改动不触碰 `specs/*.md`、`homie/crates`、`Sources`、`Tests`。

## 4. 验收标准

1. R1–R6 全部落地到 `standards.md` / `quality-gates.md` / `AGENTS.md`。
2. FC-01 至 FC-08 全部通过，证据记录到 `docs/verification/evidence-driven-gauntlet-gates/`。
3. OpenSpec `plan.md` / `tasks.md` / `alignment-report.md` 将 R1–R6 映射到任务与用例。
4. Beads issue 仅在证据匹配交付状态后关闭。

## 5. Beads 追踪

- change_id: `evidence-driven-gauntlet-gates`
- 类型: refactor（开发流程/质量门禁增强）
- 优先级: P0（本次任务要求修复）
