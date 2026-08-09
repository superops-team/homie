# Spec Review Report: Diri Proto Node Checkpoint Move Fixtures

```yaml
change_id: diri-proto-node-checkpoint-move
beads: homie-uxm
status: pass
reviewed_at: 2026-08-08
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把 DTO 扩成文件传输或 node runtime。
- 推荐方向：只补 checkpoint/move DTO 和 serde fixtures。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 范围控制 | checkpoint/move 是协议和 runtime 双层能力。 | 过度实现。 | 本 slice 只做 `homie-proto` DTO。 |
| P1 | Wire 兼容 | `providerSessionId`、`quarantinePath`、`targetNodeId` 等需 camelCase。 | Diri node wire 不兼容。 | fixture 覆盖。 |

## 3. 测试规划

| 类型 | 覆盖点 | 关键用例 |
|---|---|---|
| Unit | checkpoint DTO | manifest/blob/stage serde |
| Unit | move DTO | commit/abort/record serde |
| Quality | gates | homie-proto tests/check/clippy/fmt |
