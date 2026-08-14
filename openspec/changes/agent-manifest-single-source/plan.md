# agent-manifest-single-source OpenSpec Plan

## 1. 目标

将 Homie 内置 agent manifest 从 Swift/Rust 双人工源收敛为单人工源：

- 权威人工源：`homie/crates/homie-engine/manifests/`
- Swift resource mirror：`Sources/HomieCore/Resources/manifests/`
- 同步方向：Rust source -> Swift mirror
- 门禁：本地与 CI 均能阻断 Swift/Rust manifest drift

本变更不删除 Swift `AgentCatalog`，不改变 user overrides，不改变 manifest schema。

## 2. 输入文档

- PRD：`prd-spec/refactors/agent-manifest-single-source/2026-08-13-agent-manifest-single-source-design.md`
- Spec Review：`docs/verification/agent-manifest-single-source/spec-review-report.md`
- 功能验证 Case：`docs/verification/agent-manifest-single-source/functional-cases.md`
- Beads：`homie-rc2`

## 3. 模块规划

### M1: Manifest 同步与漂移检测脚本

职责：

- 从 Rust Engine manifest source 同步 Swift resource mirror。
- 检测两目录是否一致。
- 输出可执行修复提示。

涉及：

- `scripts/sync-agent-manifests.sh`
- `scripts/check-agent-manifest-drift.sh`

关联 Case：FC-01、FC-02、FC-03、FC-08。

### M2: 本地与 CI 门禁接入

职责：

- 把 drift check 接入本地 contributor gate。
- 把 drift check 接入 GitHub Actions。
- 保证 drift 在测试前快速失败。

涉及：

- `scripts/check.sh`
- `.github/workflows/ci.yml`

关联 Case：FC-01、FC-03、FC-10。

### M3: Swift mirror 语义文档化

职责：

- 更新 README/CONTRIBUTING/源码注释。
- 明确 Swift resource manifest 是生成镜像，不是人工源。
- 保持 Swift CLI/Core 读取 resource bundle 的行为。

涉及：

- `README.md`
- `CONTRIBUTING.md`
- `Package.swift`
- `Sources/HomieCore/AgentCatalog.swift`
- `Sources/HomieCore/AgentDescriptor.swift`
- `Sources/HomieCore/ResourceBundle.swift`

关联 Case：FC-04、FC-05、FC-09。

### M4: Catalog 回归验证

职责：

- 保证 Rust Engine、Swift Core、Swift CLI/MCP 读取一致 catalog。
- 保证 package 仍从 Rust source 打包。
- 保证 user overrides 不参与内置同步。

涉及：

- `Tests/HomieCoreTests/AgentKindTests.swift`
- `Tests/HomieCLITests/CommandGrammarTests.swift`
- `homie/crates/homie-engine/tests/*`
- `homie/scripts/package.sh`

关联 Case：FC-04、FC-05、FC-06、FC-07、FC-08。

## 4. 依赖图

```text
T1 drift check RED baseline
  -> T2 sync script
  -> T3 check script GREEN
  -> T4 local/CI gate
  -> T5 docs/comments
  -> T6 verification execution
```

T4 依赖 T3；T5 可与 T4 并行，但进入功能验证前必须全部完成。

## 5. 风险控制

| 风险 | 控制 |
|------|------|
| 直接删除 Swift resource 导致 CLI/Core 断裂 | 第一阶段保留 Swift mirror，只改为生成产物 |
| check 只比较数量，漏掉行为 drift | FC-03 修改 `agent.shortLabel` 强制验证内容级比较 |
| sync 误处理 user overrides | sync/check 只操作两个 repo 内置目录，FC-08 验证 HOME override 不参与 |
| package 来源混淆 | FC-07 验证 bundle manifest 从 Rust source 复制 |
| 文档继续误导贡献者 | FC-09 扫描 README/CONTRIBUTING/docs/源码注释 |

## 6. 验收引用

- P0 准出：FC-01、FC-02、FC-03、FC-04、FC-06 必须通过。
- P1 准出：FC-05、FC-07、FC-08、FC-09、FC-10 必须通过，若本机缺少 package 工具导致 FC-07 不能执行，必须记录环境阻塞和替代 package structure 检查。

## 7. 输出物

- 同步脚本与 drift check 脚本。
- 本地/CI gate 更新。
- 文档和注释更新。
- 功能验证执行报告。
- 两轮 code review 报告。
- E2E/package 验证报告。
