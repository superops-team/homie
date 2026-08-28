# Spec Review Report: llm-gateway-tier3-evidence-hardening

评审对象：`prd-spec/refactors/llm-gateway-tier3-evidence-hardening/2026-08-28-llm-gateway-tier3-evidence-hardening-design.md`
评审依据：`docs/development/standards.md`、`docs/development/quality-gates.md`、`specs/llm-gateway.md`

## 评审概览

| 维度 | 判定 | 关键问题数 |
|------|------|-----------|
| 1. 上下文逻辑连贯性 | 通过 | 0 |
| 2. 内容空洞排查 | 通过 | 0 |
| 3. 歧义点识别 | 需改进 | 1 |
| 4. 大模型语义理解偏差 | 通过 | 0 |
| 5. SDD/TDD 开发模式适配 | 不适用 | 0 |
| 6. 最小化实现原则 | 通过 | 0 |
| 7. 向下兼容隐患 | 不适用 | 0 |
| 8. 存量业务影响 | 不适用 | 0 |
| 9. 功能失效风险预判 | 通过 | 0 |
| 10. 落地可行性评估 | 通过 | 0 |
| 11. 任务拆解与排期 | 通过 | 0 |
| 12. 可扩展性影响 | 通过 | 0 |
| 13. 过度设计警惕 | 通过 | 0 |
| 14. 小而高效改动 | 通过 | 0 |
| 15. 代码优雅性 | 不适用 | 0 |
| 16. 架构统一性 | 通过 | 0 |

总评：12/16 通过，1/16 需改进，3/16 不适用（本变更为纯证据/契约补齐，不涉及生产代码与 SDD/TDD 开发循环）。

## 需改进项详情

### 维度 3：歧义点识别

问题 1：R2 的「变更行覆盖率 fail-under」未给出具体阈值 `<n>`，也未在 PRD 内预先声明「覆盖率/变异工具是否可用、不可用则采用手动 fallback」的结论。这会让执行者对“覆盖率门禁是否达标”产生不同解读。

建议：在执行阶段固化两个事实——(1) 当前环境 `cargo-llvm-cov` / `cargo-tarpaulin` / `cargo-mutants` 均缺失，因此采用手动 fallback；(2) 覆盖率目标明确为「gateway 安全关键路径的每个变更/新增行被某个测试执行」的手动逐行核对，并在 `functional-verification-report.md` 中给出「已核对行数 / 总关键行数」的真实数字。

> 处理：已采纳。见 `functional-cases.md` FC-06 与 `functional-verification-report.md` 的逐行核对表。

## 改进后任务拆解建议

原 PRD 未附 OpenSpec tasks；本报告的后续 `openspec/changes/llm-gateway-tier3-evidence-hardening/tasks.md` 按 R1–R4 拆为 5 个 task，并将 FC-01..FC-06 映射到验收标准。

## 风险清单

| 风险 | 严重度 | 缓解措施 |
|------|--------|---------|
| failure model 只列伤害方式、无真实捕获层（空洞） | H | R1 强制每条标注「捕获层 + 证据编号」，执行时逐条对照现有/新增测试 |
| 手动变异破坏代码后未恢复，污染仓库 | H | 每次变异只改一行、跑完即 `git checkout --` 恢复；FC-04 `git status --short` 门禁拦截残留 |
| 对抗用例只跑单元测试、绕过真实 HTTP 路径 | M | FC-05 强制走 axum router 集成路径（`tests/gateway.rs` 的 wiremock 上游） |
| 安全关键行未被任何测试执行（覆盖率虚高） | M | FC-06 手动逐行核对 + 变异测试交叉验证（突变未被杀死即证明该行无测试） |
