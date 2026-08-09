# Spec Review Report: Diri Proto Host Catalog and Remote Config

```yaml
change_id: diri-proto-host-remote-config
beads: homie-5kx
status: pass
reviewed_at: 2026-08-08
source_prd: prd-spec/features/diri-proto-host-remote-config/2026-08-08-diri-proto-host-remote-config-design.md
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把 protocol DTO 扩成文件权限/remote runtime 实现。
- 推荐方向：只补 serde DTO 和 fixtures。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 范围控制 | Diri host/remote modules 包含 load/save 权限逻辑。 | proto crate 责任过重。 | 本 slice 只做 DTO/serde。 |
| P1 | Wire 兼容 | `defaultCwd`、`bindHost`、`forwardAnyPort` 使用 camelCase。 | 与 Diri config 不兼容。 | 加 fixture test。 |
| P2 | Minimal schema | HostEntry 只要求 id/ssh。 | 过度 required 会拒绝合法配置。 | 测试 minimal host。 |

## 3. 整改后的完善方案

- 新增 host catalog 和 remote config DTO。
- 新增 protocol contract test。
- 不实现 load/save/owner-only 权限。

## 4. 测试规划

| 类型 | 覆盖点 | 关键用例 |
|---|---|---|
| Unit | hosts serde | documented/minimal/empty schema |
| Unit | remote serde | current/legacy schema |
| Quality | check/clippy/fmt/diff/parity | homie-proto gates |

