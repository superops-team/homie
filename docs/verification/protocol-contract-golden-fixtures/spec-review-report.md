# Swift/Rust 协议契约 Golden Fixtures Spec Review Report

## 1. 总体结论

- 可行性：高。
- 最大风险：把 P0 漂移门禁扩大成协议重写、schema/codegen 或运行时兼容层，导致首阶段无法小步落地。
- 推荐方向：首阶段只固化现有 control protocol envelope，通过共享 fixture 让 Swift/Rust 双端测试共同守住 wire 行为。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 修复状态 |
|---|---|---|---|---|
| P0 | 范围控制 | 原 PRD 同时提到 fixture、变更流程和后续 schema/codegen，容易被执行成协议体系改造 | P0 任务过大，无法快速关闭漂移风险 | 已修复：新增首阶段关闭口径，只覆盖现有 envelope、null/empty params、error 和最大行边界 |
| P0 | 安全 | fixture 未明确禁止真实 prompt、token、cookie、私有路径 | 测试数据可能泄漏敏感信息 | 已修复：PRD 要求只使用人工构造最小 JSON，禁止敏感 payload |
| P1 | durable spec 边界 | Swift/Rust 不一致时是否改实现、改 spec 还是改 fixture 不清晰 | 执行者可能用 fixture 固化错误行为 | 已修复：实现前必须评估 `specs/engine-session-runtime.md`，如目标语义变化先更新 durable spec |
| P1 | 打包边界 | 新增 `protocol-fixtures/` 未说明是否进入 app bundle | package/dev bundle 可能携带无用测试资源 | 已修复：fixture 只作为测试契约，不作为运行时配置或打包资源 |
| P2 | CI 风险 | 直接要求 CI gate，未说明本地 gate 顺序 | CI 波动可能阻塞 P0 开发 | 已修复：先接入本地 `scripts/check.sh`，CI 作为同一 change 最后一步 |

## 3. 整改后的完善方案

首阶段目标是创建语言无关的 `protocol-fixtures/control-message/`，覆盖 request、event、success response、error response、null/empty params 和 max-line boundary。Swift `HomieProtocolTests` 与 Rust `homie-proto` 测试必须读取同一批 fixture。

非目标保持不变：不迁移 Protobuf/Cap'n Proto，不改变 NDJSON wire format，不新增兼容层，不重写 Swift CLI。

实现前必须补齐 OpenSpec 三件套，并在 alignment 中说明每个 fixture case 对应的 Swift/Rust 测试和 evidence path。若 review 发现当前 Swift/Rust 行为冲突，先明确 durable contract，再修改任一端实现。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 补 OpenSpec plan/tasks/alignment | `openspec/changes/protocol-contract-golden-fixtures/*` | 本报告 | P0 |
| Contract | 明确 fixture README 和敏感信息规则 | `protocol-fixtures/control-message/README.md` | OpenSpec | P0 |
| Rust | 让 `homie-proto` 测试读取共享 fixture | focused test log | fixture | P0 |
| Swift | 让 `HomieProtocolTests` 读取共享 fixture | focused test log | fixture | P0 |
| Gate | 接入本地 drift gate | `scripts/check.sh` 或等价脚本证据 | Swift/Rust tests | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| Rust focused | decode/encode 规范化 | request/event/ok/null/err | 开发中 |
| Swift focused | decode/encode 规范化 | 与 Rust 同一 fixture | 开发中 |
| Negative | 非 object、缺 id、超长 line | invalid cases | 开发中 |
| Gate | 本地漂移阻断 | 修改一端 envelope 行为应失败 | 准出前 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| Phase 0 | 先 | OpenSpec + fixture contract | 防止范围扩大 | alignment report |
| Phase 1 | 次 | Rust/Swift focused RED/GREEN | 行为冲突需先定 spec | test logs |
| Phase 2 | 后 | 本地 gate + release evidence | CI 接入可后置 | verification report |

## 7. 待确认问题

- 是否把 shared fixture 目录放在仓库根 `protocol-fixtures/`，还是放在 `Tests/Fixtures/` 并由 Rust 通过相对路径读取。推荐根目录，降低语言偏向。
