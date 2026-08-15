# Full Dev App Bundle 与 Smoke 验证 Spec Review Report

## 1. 总体结论

- 可行性：高。
- 最大风险：把 full dev bundle 做成第二套 release package，重复 universal/notary/DMG/remote helper 逻辑。
- 推荐方向：首阶段只做本机架构 full dev bundle smoke，证明随包 Engine 和随包 CLI 在临时 app support 下真实连通。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 修复状态 |
|---|---|---|---|---|
| P0 | 范围控制 | 原 PRD 没有明确首阶段是否需要 universal、remote helpers、sidecar、CI | P0 dev smoke 容易扩成完整 release packaging | 已修复：首阶段限定本机架构 app、Engine、holder、askpass、MCP、Swift CLI、manifests |
| P0 | 用户数据安全 | smoke 只说临时目录，未要求验证不会连接真实 daemon/socket | 可能污染真实 `~/Library/Application Support/Homie` 或会话 | 已修复：要求临时 `HOMIE_APP_SUPPORT`/socket，并扫描 smoke log 确认不触碰真实状态 |
| P1 | 与 release 重复 | 未约束与 `package-release-phases` 的 verify helper 关系 | 后续出现两套 bundle 结构知识 | 已修复：要求 smoke 检查项与 package verify 对齐，可后置抽共同 helper |
| P1 | 自动 launch 风险 | `dev.sh` 现有语义是 build 后 exec app，full smoke 若沿用可能接管真实环境 | 验证路径不稳定，可能影响用户当前 daemon | 已修复：`--full --smoke` 默认不 launch UI，不继承 `HOMIE_SOCKET` |
| P2 | CI 稳定性 | 原 PRD 直接建议 CI 跑 full dev bundle smoke | macOS CI 工具链/超时可能影响普通开发 | 已修复：先记录本地 evidence，CI 接入作为稳定后步骤 |

## 3. 整改后的完善方案

首阶段实现 `dev.sh --full --no-launch --smoke` 或等价脚本，生成本机架构 `.app`，包含 Homie GUI、Rust Engine、holder、askpass、MCP、Swift CLI 和 agent manifest catalog。smoke 只在临时 app support 下启动随包 Engine，并用随包 CLI 执行 `status` / `doctor` 类连通检查。

非目标：不做 universal、不做 notarization、不生成 DMG/update zip、不构建三平台 remote helper catalog、不复制 sidecar，除非 OpenSpec 证明当前 smoke 需要。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 补 OpenSpec 三件套 | `openspec/changes/full-dev-bundle-smoke/*` | 本报告 | P0 |
| Script | 新增 full dev bundle 参数或脚本 | `dev.sh --full --smoke` | OpenSpec | P0 |
| Bundle | 复制本机核心 runtime 资源 | `.app` path evidence | Script | P0 |
| Smoke | 临时 Engine + 随包 CLI 连通 | smoke log | Bundle | P0 |
| Docs | 记录 quick dev 与 full dev 的边界 | README/homie README update | Smoke | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| Script | 参数解析 | `--full --no-launch --smoke` 不启动 UI | 开发中 |
| Bundle | 资源完整性 | 核心二进制、manifest 数量、codesign | 开发中 |
| Runtime | 随包 Engine 连通 | 临时 app support + 随包 CLI `status` | 准出前 |
| Regression | 快速 dev 不回退 | `dev.sh --settings remote` 或等价路径 | 准出前 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| Phase 0 | 先 | OpenSpec 和 smoke case | 防止与 release scope 混淆 | alignment report |
| Phase 1 | 次 | 本机 full bundle 构建 | 工具缺失需记录 | build log |
| Phase 2 | 后 | 临时 Engine smoke + 文档 | 真实状态污染检查 | verification report |

## 7. 待确认问题

- 是否选择修改 `dev.sh` 还是新增 `dev-bundle.sh`。推荐优先 `dev.sh --full`，但 OpenSpec 需要评估现有 `dev.sh` exec 语义的兼容风险。
