# Package/Release 阶段化 Spec Review Report

## 1. 总体结论

- 可行性：中高。
- 最大风险：在重排 `package.sh` 时改变默认 release 行为，或者引入新的发布运行时导致工具链和 CI 复杂度上升。
- 推荐方向：首阶段保留 shell 和默认完整流程，只新增 `preflight`、`verify-only` 和本机快速构建能力。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 修复状态 |
|---|---|---|---|---|
| P0 | 兼容性 | 原 PRD 提到 shell 和 TS 两种方向，未强制首阶段保守路径 | 实现可能一次性引入新 runtime，扩大 review 面 | 已修复：首阶段只允许在现有 `package.sh` 内函数化和参数化 |
| P0 | 发布风险 | phase 化可能改变无参数默认 release 行为 | 破坏签名、notary、DMG、update zip 链路 | 已修复：默认无参数完整流程必须保持不变并通过 characterization evidence |
| P1 | verify-only 副作用 | 未明确 verify phase 是否只读 | 验证命令可能隐式重签名或修改 bundle | 已修复：`verify` 禁止签名、复制、notary、删除，只读检查 |
| P1 | 与 full dev smoke 重叠 | package verify 与 dev smoke 可能维护两套检查 | 后续 bundle 结构修改时漂移 | 已修复：要求复用同一检查 helper 或在 OpenSpec 中说明差异 |
| P2 | local-arm64 语义 | `--local-arm64` 容易被误当 release 产物 | 发布错误架构或缺少 helper catalog | 已修复：local-only 明确不替代 universal/notary/DMG/update zip |

## 3. 整改后的完善方案

首阶段在 `homie/scripts/package.sh` 内部建立 phase 函数和参数解析，新增 `--phase preflight`、`--phase verify --app <path>` 和 `--local-arm64`。默认无参数仍执行现有完整 release package 流程。

`verify` phase 只能做只读检查：plist、核心二进制、manifest 数量、remote helper catalog、codesign、可选临时 Engine smoke。签名、notary、DMG 和 updater artifact 格式不在首阶段改变。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 补 OpenSpec plan/tasks/alignment | `openspec/changes/package-release-phases/*` | 本报告 | P0 |
| Characterization | 记录当前默认 package 关键输出 | package structure log | OpenSpec | P0 |
| Script | 函数化现有 package 流程 | `package.sh` diff | Characterization | P0 |
| Verify | 增加只读 verify phase | verify log | Script | P0 |
| Docs/CI | 更新 PACKAGING 和 bundle job | docs + CI evidence | Verify | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| Script syntax | shell 语法 | `bash -n homie/scripts/package.sh` | 开发中 |
| Default flow | 行为兼容 | 无参数 package 成功，关键 artifact 数量一致 | 准出前 |
| Preflight | 前置失败 | 缺工具/target/signing env 自洽检查 | 开发中 |
| Verify-only | 只读验证 | 对已存在 app 执行，不改 mtime/签名 | 开发中 |
| CI | bundle job | CI 复用 verify phase | 后置 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| Phase 0 | 先 | OpenSpec + characterization | 防止默认行为漂移 | report/log |
| Phase 1 | 次 | shell phase 化 + preflight | 保持默认路径 | focused logs |
| Phase 2 | 后 | verify-only + docs/CI | full CI 可后置 | release readiness |

## 7. 待确认问题

- `verify` phase 是否在首阶段执行临时 Engine smoke，还是只提供可选 flag。推荐默认做结构/codesign，临时 Engine smoke 用显式 flag，避免 verify-only 在 CI 中过慢。
