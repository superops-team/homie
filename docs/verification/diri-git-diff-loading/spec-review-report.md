# Spec Review Report: Diri Git Diff Loading

## 1. 总体结论

- 可行性：高。
- 最大风险：DTO patch wire 与 Diri 不一致。
- 推荐方向：新增 base64 serializer，runtime 用真实 git fixture 验证，不宣称 UI 完成。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 协议 | patch 如果按默认 bytes 数组序列化会偏离 Diri | client 不兼容 | 使用 base64 string |
| P1 | 真实性 | 只解析 patch 字符串不代表 runtime 可用 | UI 无真实数据 | 用真实 git repo + session cwd E2E |
| P2 | 范围 | runtime diff 不等于 inspector UI 完成 | 误报 UI-004 | parity 保持 partial |

