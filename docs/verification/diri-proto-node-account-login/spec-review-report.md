# Spec Review Report: Diri Proto Node Account Login Fixtures

```yaml
change_id: diri-proto-node-account-login
beads: homie-05q
status: pass
reviewed_at: 2026-08-08
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把 DTO 扩成账号存储或登录 runtime。
- 推荐方向：只补 account/login/provider call DTO 和 serde fixtures。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 范围控制 | account/login 是协议和运行时双层能力。 | 过度实现。 | 本 slice 只做 DTO。 |
| P1 | Wire 兼容 | installation/login 字段使用 camelCase 且多 optional。 | Diri wire 不兼容。 | fixture 覆盖 optional omission。 |

