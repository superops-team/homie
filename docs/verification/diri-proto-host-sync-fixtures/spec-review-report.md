# Spec Review Report: Diri Proto Host Sync Prefs Fixtures

```yaml
change_id: diri-proto-host-sync-fixtures
beads: homie-vuz
status: pass
reviewed_at: 2026-08-08
source_prd: prd-spec/features/diri-proto-host-sync-fixtures/2026-08-08-diri-proto-host-sync-fixtures-design.md
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把 DTO fixture 扩成真实 remote sync 实现。
- 推荐方向：只补 proto DTO 和 serde contract test。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 范围控制 | host.sync_prefs 有业务实现和协议 DTO 两层。 | 容易改远程逻辑。 | 本 slice 只动 `homie-proto`。 |
| P1 | Wire 兼容 | `error` 成功时必须省略。 | Swift/Diri fixture 不兼容。 | 加 serde test。 |
| P2 | 空状态 | `ok=true` 且 `synced=[]` 是合法“无配置”。 | 可能被误判失败。 | 测试覆盖空 synced。 |

## 3. 整改后的完善方案

- 新增三个 DTO。
- 新增 protocol contract test。
- 不改 runtime/client/remote。

## 4. 测试规划

| 类型 | 覆盖点 | 关键用例 |
|---|---|---|
| Unit | serde roundtrip | Diri host.sync_prefs fixture |
| Quality | check/clippy/fmt/diff/parity | homie-proto gates |

