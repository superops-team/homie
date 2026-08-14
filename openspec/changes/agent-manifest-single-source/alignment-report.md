# agent-manifest-single-source Alignment Report

## 1. 对齐结论

- 对齐状态：通过
- PRD：`prd-spec/refactors/agent-manifest-single-source/2026-08-13-agent-manifest-single-source-design.md`
- Spec Review：`docs/verification/agent-manifest-single-source/spec-review-report.md`
- 功能验证 Case：`docs/verification/agent-manifest-single-source/functional-cases.md`
- OpenSpec Plan：`openspec/changes/agent-manifest-single-source/plan.md`
- OpenSpec Tasks：`openspec/changes/agent-manifest-single-source/tasks.md`
- Beads：`homie-rc2`

所有 P0/P1 需求均已映射到功能验证 Case 和 OpenSpec Task。进入实现前无未解决 P0/P1 对齐缺口。

## 2. PRD 需求覆盖

| PRD 需求 | 功能验证 Case | OpenSpec Task | 对齐状态 |
|----------|---------------|---------------|----------|
| Rust Engine manifest 是唯一人工源 | FC-01, FC-02, FC-06, FC-07 | T1, T2, T7, T9 | 通过 |
| Swift manifest 是生成镜像，不能人工维护 | FC-02, FC-03, FC-04, FC-05 | T2, T3, T6 | 通过 |
| 文档、脚本和测试路径统一 | FC-09, FC-10 | T4, T5, T10 | 通过 |
| CI 阻断漂移 | FC-01, FC-03, FC-10 | T3, T4, T10 | 通过 |
| 不改变 user override 目录 | FC-08 | T8 | 通过 |
| 不改变 manifest schema | FC-04, FC-06 | T6, T7 | 通过 |
| 不把所有 agent 转成 typed driver | FC-04, FC-06 | T6, T7 | 通过 |
| 打包产物仍从 Rust source catalog 复制 | FC-07 | T9 | 通过 |

## 3. Task 覆盖验证

| Task | 验收标准来源 | 关联 Case | 是否存在无验证 Task |
|------|--------------|-----------|---------------------|
| T1 | PRD 目标 1/3，Review P1/P2 | FC-01 | 否 |
| T2 | PRD 目标 1/2 | FC-02 | 否 |
| T3 | PRD 目标 3，验收标准 3 | FC-03 | 否 |
| T4 | PRD 目标 4，本地/CI gate | FC-01, FC-03, FC-10 | 否 |
| T5 | PRD 目标 3，文档一致性 | FC-09 | 否 |
| T6 | Swift CLI/Core 兼容 | FC-04, FC-05 | 否 |
| T7 | Rust Engine runtime 兼容 | FC-06 | 否 |
| T8 | user override 非目标边界 | FC-08 | 否 |
| T9 | package 来源验证 | FC-07 | 否 |
| T10 | 准出汇总 | FC-01 至 FC-10 | 否 |

## 4. Case 覆盖验证

| Case | 必要性 | 覆盖缺口 |
|------|--------|----------|
| FC-01 | 证明 drift check 能抓当前真实问题 | 无 |
| FC-02 | 证明同步脚本能恢复一致状态 | 无 |
| FC-03 | 证明内容级 drift 会失败 | 无 |
| FC-04 | 证明 Swift Core 不回退 | 无 |
| FC-05 | 证明 Swift CLI/MCP vocabulary 不回退 | 无 |
| FC-06 | 证明 Rust Engine 不回退 | 无 |
| FC-07 | 证明 package 仍从 Rust source 打包 | 无；若环境阻塞需记录替代验证 |
| FC-08 | 证明 user overrides 不被脚本误处理 | 无 |
| FC-09 | 证明文档不再误导人工源 | 无 |
| FC-10 | 证明全量本地 gate 通过 | 无 |

## 5. 修复记录

| 问题 | 发现阶段 | 修复 |
|------|----------|------|
| PRD 推荐方案在“删除 Swift manifest”和“生成 Swift manifest”之间未收敛 | Step 1 Spec Review | 已改为第一阶段保留 Swift resource mirror，但由 Rust source 生成 |
| PRD 实施步骤仍写“Swift CLI/Core 不再读取旧 manifest” | Step 4 Alignment | 已改为“Swift CLI/Core 可继续读取生成镜像” |
| PRD 推荐方案编号重复 | Step 4 Alignment | 已修正编号 |

## 6. 最终门禁

进入 SDD/TDD 开发前必须满足：

1. PRD、Case、OpenSpec plan/tasks 全部存在。
2. P0/P1 需求均有 Case 和 Task 覆盖。
3. Task 无无验证项。
4. 本报告记录的修复项全部完成。

当前状态满足以上门禁，可进入 Step 5 SDD + TDD 开发。
