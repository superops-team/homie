# Spec Review Report

```yaml
change_id: diri-notification-center
beads: homie-5pe
status: pass
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把通知模型误当作 native notification 完整 E2E。
- 推荐方向：先实现可测试的 notification center model 和 app rollup，把 `UI-008` 从 `missing` 推到 `partial`。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 行为真实性 | 当前没有 app 通知中心 | 用户无法看到 needs-input rollup | 增加 `homie-ui` notification model 和 app inspector rollup |
| P1 | 安全 | approve/deny 不应绕过 runtime | 可能错误执行 agent keystroke | 本轮只输出 action descriptor |
| P2 | native E2E | osascript 可能依赖系统权限 | CI 不稳定 | 本轮只测试命令构建和转义 |

## 3. Gate Decision

Decision: pass

