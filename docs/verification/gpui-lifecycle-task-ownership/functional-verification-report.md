# GPUI lifecycle ownership 功能验证报告

## 1. 结论

`gpui-lifecycle-task-ownership` 验证通过。RootView child entity subscriptions
已经由 `_subscriptions` 显式持有，不再直接 `.detach()`。

## 2. Case 结果

| Case | 目标 | 状态 | 证据 |
|------|------|------|------|
| FC-01 | PRD 边界清晰 | pass | `fc-01-prd-scope.log` |
| FC-02 | RootView child subscriptions 不再 detach | pass | `fc-02-held-subscriptions.log` |
| FC-03 | RootView 和 UtilitySurfaces targeted tests | pass | `fc-03-targeted-tests.log` |
| FC-04 | 静态门禁 | pass | `fc-04-static-gates.log` |

## 3. 边界说明

- 本轮只处理 RootView child subscriptions。
- 其他 `.detach()` 路径属于 service lifetime、hover delay 或 host initialization，未纳入本切片。
- 未修改 `homie-ui`、`homie-engine`、`homie-client`。
