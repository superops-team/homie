# agent-manifest-single-source OpenSpec Tasks

## 1. 任务清单

### T1: 建立 drift check RED 基线

- 优先级：P0
- 类型：TDD RED
- 描述：在当前双源不一致状态下，先创建 `scripts/check-agent-manifest-drift.sh` 的最小失败实现或测试入口，证明现有 drift 会被检测到。
- 交付物：
  - `scripts/check-agent-manifest-drift.sh`
  - RED 证据：`docs/verification/agent-manifest-single-source/fc-01-drift-detected.log`
- 验收标准：
  - 当前不一致状态下命令非 0。
  - 输出 drift 文件列表。
  - 输出修复提示 `scripts/sync-agent-manifests.sh`。
- 关联 Case：FC-01
- 预估工时：0.5h
- 前置依赖：无

### T2: 实现 Rust source -> Swift mirror 同步脚本

- 优先级：P0
- 类型：TDD GREEN
- 描述：新增 `scripts/sync-agent-manifests.sh`，从 `homie/crates/homie-engine/manifests/` 同步到 `Sources/HomieCore/Resources/manifests/`，确保新增、删除、修改都同步。
- 交付物：
  - `scripts/sync-agent-manifests.sh`
  - 同步后的 Swift mirror manifest 文件
  - 证据：`docs/verification/agent-manifest-single-source/fc-02-sync-green.log`
- 验收标准：
  - sync 后 drift check 0 退出。
  - Swift mirror 文件集合等于 Rust source。
  - 多余 Swift mirror 文件会被删除。
- 关联 Case：FC-02
- 预估工时：0.75h
- 前置依赖：T1

### T3: 强化 drift check 内容级比较

- 优先级：P0
- 类型：TDD RED/GREEN
- 描述：保证 `scripts/check-agent-manifest-drift.sh` 不是只比较数量，而是能检测行为字段变化。
- 交付物：
  - 强化后的 `scripts/check-agent-manifest-drift.sh`
  - 证据：`docs/verification/agent-manifest-single-source/fc-03-manual-swift-drift-blocked.log`
- 验收标准：
  - 手工修改 Swift mirror 中 `codex.agent.shortLabel` 后 check 非 0。
  - 恢复或重新 sync 后 check 0。
  - 脚本结束不残留临时 drift。
- 关联 Case：FC-03
- 预估工时：0.5h
- 前置依赖：T2

### T4: 接入本地 contributor gate 与 CI

- 优先级：P0
- 类型：SDD/TDD
- 描述：将 drift check 接入 `scripts/check.sh` 和 `.github/workflows/ci.yml`，保证本地和 CI 都能阻断漂移。
- 交付物：
  - `scripts/check.sh`
  - `.github/workflows/ci.yml`
  - 证据：`docs/verification/agent-manifest-single-source/fc-10-full-check.log`
- 验收标准：
  - `./scripts/check.sh` 会执行 drift check。
  - CI 的 Swift/Rust 早期 job 中至少有一个执行 drift check。
  - drift check 失败时 CI 失败。
- 关联 Case：FC-01、FC-03、FC-10
- 预估工时：0.5h
- 前置依赖：T3

### T5: 更新文档与源码注释

- 优先级：P1
- 类型：文档/一致性
- 描述：更新 README、CONTRIBUTING、Package.swift 与 Swift source 注释，明确 Rust manifest 是唯一人工源，Swift resource 是生成镜像。
- 交付物：
  - `README.md`
  - `CONTRIBUTING.md`
  - `Package.swift`
  - `Sources/HomieCore/AgentCatalog.swift`
  - `Sources/HomieCore/AgentDescriptor.swift`
  - `Sources/HomieCore/ResourceBundle.swift`
  - 证据：`docs/verification/agent-manifest-single-source/fc-09-doc-source-path-scan.log`
- 验收标准：
  - README/CONTRIBUTING 不再指导编辑 `Sources/HomieCore/Resources/manifests/`。
  - 源码注释若提到 Swift path，必须称其为 generated mirror。
- 关联 Case：FC-09
- 预估工时：0.5h
- 前置依赖：T2

### T6: Swift catalog 回归

- 优先级：P0
- 类型：回归测试
- 描述：验证 Swift Core/CLI/MCP 仍可通过 generated mirror 获取 agent catalog。
- 交付物：
  - 必要时更新 Swift tests
  - 证据：
    - `docs/verification/agent-manifest-single-source/fc-04-swift-core-tests.log`
    - `docs/verification/agent-manifest-single-source/fc-05-swift-cli-protocol-tests.log`
- 验收标准：
  - `swift test --package-path . --filter HomieCoreTests` 通过。
  - `swift test --package-path . --filter HomieCLITests` 通过。
  - `swift test --package-path . --filter HomieProtocolTests` 通过。
- 关联 Case：FC-04、FC-05
- 预估工时：0.5h
- 前置依赖：T2、T5

### T7: Rust Engine catalog 回归

- 优先级：P0
- 类型：回归测试
- 描述：验证 Rust Engine 仍从 Rust source manifest 加载，runtime/MCP tests 不回退。
- 交付物：
  - 证据：`docs/verification/agent-manifest-single-source/fc-06-rust-engine-manifest-tests.log`
- 验收标准：
  - `cargo test -p homie-engine --lib detect::tests::every_bundled_manifest_decodes` 通过。
  - `cargo test -p homie-engine --lib agent::tests::every_shipped_manifest_declares_an_authority` 通过。
  - `cargo test -p homie-engine --test mcp_tools` 通过。
- 关联 Case：FC-06
- 预估工时：0.5h
- 前置依赖：T2

### T8: User overrides 边界验证

- 优先级：P1
- 类型：边界测试
- 描述：验证 sync/check 只处理 repo 内置 Rust source 与 Swift mirror，不读取或改写用户 override。
- 交付物：
  - 必要时更新 check 脚本边界
  - 证据：`docs/verification/agent-manifest-single-source/fc-08-user-overrides-ignored.log`
- 验收标准：
  - 临时 HOME 下存在 `Library/Application Support/Homie/manifests/overrides` 不影响 drift check。
  - sync/check 不写入 HOME。
- 关联 Case：FC-08
- 预估工时：0.25h
- 前置依赖：T3

### T9: Package manifest 来源验证

- 优先级：P1
- 类型：E2E/package
- 描述：验证发布包仍从 Rust source catalog 复制 manifest，bundle 结构不回退。
- 交付物：
  - 证据：`docs/verification/agent-manifest-single-source/fc-07-package-manifest-source.log`
- 验收标准：
  - package 成功，或记录明确环境阻塞。
  - `Contents/Resources/bin/manifests` 存在并数量等于 Rust source。
  - bundle codesign 验证通过。
- 关联 Case：FC-07
- 预估工时：1h
- 前置依赖：T2、T4

### T10: 全量验证与报告

- 优先级：P1
- 类型：准出
- 描述：按功能验证 Case 顺序执行全部 Case，生成执行报告。
- 交付物：
  - `docs/verification/agent-manifest-single-source/functional-verification-report.md`
  - 各 FC 日志
- 验收标准：
  - P0 Case 全部通过。
  - P1 Case 通过；如环境阻塞，必须记录风险和替代验证。
- 关联 Case：FC-01 至 FC-10
- 预估工时：1h
- 前置依赖：T1-T9

## 2. Task ↔ Case 映射表

| Task | FC-01 | FC-02 | FC-03 | FC-04 | FC-05 | FC-06 | FC-07 | FC-08 | FC-09 | FC-10 |
|------|-------|-------|-------|-------|-------|-------|-------|-------|-------|-------|
| T1 | 是 |  |  |  |  |  |  |  |  |  |
| T2 |  | 是 |  |  |  |  |  |  |  |  |
| T3 |  |  | 是 |  |  |  |  |  |  |  |
| T4 | 是 |  | 是 |  |  |  |  |  |  | 是 |
| T5 |  |  |  |  |  |  |  |  | 是 |  |
| T6 |  |  |  | 是 | 是 |  |  |  |  |  |
| T7 |  |  |  |  |  | 是 |  |  |  |  |
| T8 |  |  |  |  |  |  |  | 是 |  |  |
| T9 |  |  |  |  |  |  | 是 |  |  |  |
| T10 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 |

## 3. 开发门禁

- 未完成 T1-T3，不得改 CI。
- 未完成 T4，不得进入功能验证执行。
- 未完成 T6/T7，不得进入 code review。
- 未完成 T10，不得进入 E2E 和提交。
