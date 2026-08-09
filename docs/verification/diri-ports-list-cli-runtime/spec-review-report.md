# Spec Review Report: Diri Ports List CLI Runtime

## 1. 总体结论

- 可行性：高。
- 最大风险：把 ports list 误报为 TCP forwarding 完成。
- 推荐方向：只实现 Diri `ports` list 的本地 runtime-backed 汇总，forwarding 后续单独 lane。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 范围 | ports list 与 TCP forward 容易混淆 | 误报 ART-002 完成 | parity lock 仍 partial |
| P2 | 真实性 | 仅 scanner 单测不代表 CLI 可用 | 用户无法操作 | 使用真实 session output CLI E2E |

## 3. 测试规划

| 类型 | 覆盖点 | 用例 |
|------|--------|------|
| E2E | real session output -> CLI ports | FC-DPLC-001 |
| E2E | empty state | FC-DPLC-002 |
| Quality | check/clippy/diff/parity | FC-DPLC-003 |

