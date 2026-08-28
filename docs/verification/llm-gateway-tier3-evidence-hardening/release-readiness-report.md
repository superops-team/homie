# Release Readiness Report — llm-gateway-tier3-evidence-hardening

## 1. 结论

`llm-gateway-tier3-evidence-hardening` 证据补齐已就绪。本 change 为 Tier-3 证据/契约补齐，**未改动任何生产代码**；为 `homie-gateway` 安全关键路径补上了此前缺失的 failure model、对抗、变异与逐行覆盖率证据，并把 failure model 沉淀进 `specs/llm-gateway.md`。

## 2. 交付物

| 层 | 位置 |
|----|------|
| PRD/spec | `prd-spec/refactors/llm-gateway-tier3-evidence-hardening/` |
| OpenSpec | `openspec/changes/llm-gateway-tier3-evidence-hardening/` |
| 长期契约 | `specs/llm-gateway.md` §10.1 Failure Model |
| Failure model | `docs/verification/.../failure-model.md` |
| 证据 | `docs/verification/.../`（6 个 FC 的日志 + 3 份报告） |

## 3. 行为 → 验证层映射

| 行为/风险 | 验证层 | 实际结果 |
|-----------|--------|---------|
| 全量 gateway 测试 | `cargo test -p homie-gateway` | 33/33 单元 + 11/11 集成 = **44/44 pass, exit 0** |
| 密钥泄露负向对照 | grep gate 对坏样本 / 源码 | 坏样本命中、源码零命中 |
| 对抗（真实 axum 路由） | `cargo test -p homie-gateway --test gateway` | **11/11 pass, exit 0** |
| 手动变异 | M1–M5 逐个引入 → 测试失败 → 恢复 | **5/5 杀死**，恢复后 44/44 |
| 逐行覆盖率 | 手动函数级核对 | 21 函数：19 covered / 2 partial / 0 uncovered |
| 静态门禁 | `git diff --check`（scoped） | exit 0 |

所有数字来自最后一次 fresh run（`fc-10-e2e.log`，在最后编辑之后执行），非形容词。

## 4. 跳过层与原因

- 覆盖率工具（`cargo-llvm-cov`/`cargo-tarpaulin`）与变异工具（`cargo-mutants`）缺失 → 按 `quality-gates.md` §4.1/§4.2 手动 fallback。
- 未新增 Rust 依赖 / CI 强制门禁 → PRD 非目标，显式排除。

## 5. 已知限制（known limits）

见 `failure-model.md` §Known Limits 汇总：KL-04a/b、KL-05a/b、KL-06a、KL-07a/b、KL-08a。均为后续加固项，非本次 scope。

## 6. 可复现性

- 命令入口：`cargo test --manifest-path homie/Cargo.toml -p homie-gateway`
- 工具版本：Rust 1.95.0（`homie/rust-toolchain.toml` 锁定）
- 环境注意：wiremock 集成需 loopback 绑定权限（沙箱外/授权执行）

## 7. 失败与解决

无失败；仅一处文档引用不一致（FM-03/FM-04）在 code-review round 1 修复。

## 8. 残余风险

- `policy::record_audit` 的 `gateway_audit` 行无行级断言（partial，建议独立 bugfix）。
- 沙箱环境限制已如实记录，未影响结论。
