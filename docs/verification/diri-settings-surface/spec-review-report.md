# Spec Review Report

```yaml
change_id: diri-settings-surface
beads: homie-s0w
status: pass
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把静态设置面板误标为完整 settings parity。
- 推荐方向：先实现持久化偏好和真实 settings surface，把 `UI-006` 从 `missing` 推到 `partial`。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 真实性 | `OpenSettings` 当前只写 notice | 用户无法配置偏好 | 改为打开 settings surface |
| P1 | 持久化 | preferences 表无 typed API | UI 无法证明配置保存 | 增加 typed preferences API 和测试 |
| P2 | 范围 | Remote settings 完整 pairing 不在本轮 | 容易扩大实现 | 只持久化 companion access，不处理 token |

## 3. 测试规划

| 类型 | 覆盖点 | 命令 |
|---|---|---|
| Storage | preferences roundtrip | `cargo test -p homie-storage --test storage_bootstrap -- --nocapture` |
| App | settings command/surface | `cargo test -p homie-app --tests -- --nocapture` |
| Quality | clippy | `cargo clippy -p homie-storage -p homie-app --all-targets -- -D warnings` |

## 4. Gate Decision

Decision: pass

