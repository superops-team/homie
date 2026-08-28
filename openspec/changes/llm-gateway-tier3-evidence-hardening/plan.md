# OpenSpec Plan — llm-gateway-tier3-evidence-hardening

## 目标

为 `homie-gateway` 安全关键路径补齐 Tier-3 证据（failure model + 对抗 + 变异 + 逐行覆盖率），并把 failure model 沉淀为长期契约。不改动任何生产代码。

## 模块划分与依赖

| 模块 | 内容 | 依赖 |
|------|------|------|
| M1 文档 | `docs/verification/llm-gateway-tier3-evidence-hardening/failure-model.md` | 前置：gateway 源码已审阅 |
| M2 契约 | `specs/llm-gateway.md` §10 新增 Failure Model | 依赖 M1 |
| M3 验证执行 | FC-01..FC-06 执行与 `fc-*.log` 证据 | 依赖 M1 |
| M4 报告 | `functional-verification-report.md`、`release-readiness-report.md`、`code-review-round-*.md` | 依赖 M3 |

依赖图：M1 → M2；M1 → M3 → M4。

## 边界

- 不新增 Rust 依赖/工具（覆盖率与变异均手动 fallback）。
- 不修改 `homie/crates`、`Sources`、`Tests`。
- 不引入 CI 强制门禁（独立后续变更）。
