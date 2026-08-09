# Spec Review Report: Diri Worktree CLI Runtime

## 1. 总体结论

- 可行性：高。
- 最大风险：真实 git 子进程在测试中等待输入或受用户全局 git config 影响。
- 推荐方向：固定 `/usr/bin/git`、stdin null、最小 env，并用真实临时 repo E2E。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 运行风险 | git 命令可能 prompt/hang | 测试或 runtime 阻塞 | 禁用 stdin prompt，固定 env |
| P1 | 范围 | UI worktree sheet 未覆盖 | 误报 GIT/UI 完成 | parity lock 保持 partial |
| P2 | 路径 | branch 含 `/` 导致 path 不稳定 | CLI/E2E 不可预测 | 实现 Diri branch slug |

## 3. 完善方案

按 PRD/OpenSpec 实现 runtime/client/CLI 的 create/list/remove，测试用真实 git repo，保持 UI 和默认 branch 自动生成作为后续 lane。

## 4. 测试规划

| 类型 | 覆盖点 | 用例 |
|------|--------|------|
| Unit | porcelain parser | FC-DWCR-001 |
| Integration | runtime git ops | FC-DWCR-002 |
| E2E | CLI real binary | FC-DWCR-003 |
| Quality | gates | FC-DWCR-004 |
