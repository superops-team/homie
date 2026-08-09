# Spec Review Report: Diri Proto Node Hello Usage Fixtures

```yaml
change_id: diri-proto-node-usage-fixtures
beads: homie-7i0
status: pass
reviewed_at: 2026-08-08
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把 node DTO 扩成 checkpoint/move/runtime 网络实现。
- 推荐方向：只补 hello/status/usage DTO 和 wire tests。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 范围控制 | Diri node protocol 很大。 | 容易过度实现。 | 本切片只做 hello/status/usage。 |
| P1 | 安全 | hello result 不能序列化 token/secret。 | 泄漏凭证语义。 | 测试确保 result JSON 不含 token/secret。 |
| P2 | Wire 兼容 | ProviderKind 作为 map key 应为 lowercase。 | fleet usage map 不兼容。 | 测试 BTreeMap 序列化。 |

## 3. 测试规划

| 类型 | 覆盖点 | 关键用例 |
|---|---|---|
| Unit | node hello/status | camelCase + no token |
| Unit | usage DTO | usage event/query/result and provider map |
| Quality | gates | homie-proto tests/check/clippy/fmt |
