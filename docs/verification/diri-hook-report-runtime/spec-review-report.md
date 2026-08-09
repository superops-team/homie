# Spec Review Report: Diri Hook Report Runtime

## 1. 总体结论

- 可行性：高。
- 最大风险：破坏 hook fail-open 行为。
- 推荐方向：只有显式 `--data-dir` 才写 runtime；默认 hook CLI 保持 parse-only。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 运行语义 | hook 解析不进入 runtime | UI/session 状态不会变化 | 写入 session needs-input |
| P1 | 兼容性 | hook CLI 默认写 HOME 会污染本机状态 | hook 集成风险 | 仅 `--data-dir` 写入 |
| P2 | 范围 | PermissionRequest 持久化不等于完整 hook bus | 误报 RT-005 | parity 保持 partial |

