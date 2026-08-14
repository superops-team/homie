# Code Review Report

## 1. 审查范围

- 文件/模块：
  - `scripts/check-agent-manifest-drift.sh`
  - `scripts/sync-agent-manifests.sh`
  - `scripts/check.sh`
  - `.github/workflows/ci.yml`
  - `README.md`
  - `CONTRIBUTING.md`
  - `Package.swift`
  - `Sources/HomieCore/AgentCatalog.swift`
  - `Sources/HomieCore/AgentDescriptor.swift`
  - `Sources/HomieCore/ResourceBundle.swift`
  - `Sources/HomieCore/Resources/manifests/*.json`
  - `Tests/HomieCoreTests/AgentKindTests.swift`
  - `homie/crates/homie-app/src/inspector.rs`
  - `homie/crates/homie-engine/src/legacy_remote.rs`
  - `license-policy.json`
- 变更类型：manifest 镜像同步、脚本新增、CI gate、Swift/Rust 语义对齐、测试稳定性修复、license policy 清理。
- 调用链/数据流：
  - Rust source manifest -> sync script -> Swift resource mirror -> Swift `AgentCatalog`
  - Rust source manifest -> package bundle `Contents/Resources/bin/manifests`
  - `scripts/check.sh` -> manifest drift gate + Swift/Rust full checks + license policy
  - App package -> bundled `homied-rs` -> temp `HOMIE_APP_SUPPORT` startup smoke
- 参考规则：
  - `AGENTS.md`：不保留无意义兼容层、最小实现、按 PRD/OpenSpec/验证证据推进。
  - `code-review` skill：两轮审查、发现即修复、验证诚实。

## 2. 旧问题复核

| ID/标题 | 位置 | 状态 | 依据 |
|---|---|---|---|
| FC-10 `homie-app` GPUI scheduler cleanup abort | `homie/crates/homie-app/src/inspector.rs` | fixed | `WorkbenchInspector::set_visible(false)` 现在取消 `refresh_task` 和 `review_task`；`./scripts/check.sh` 已通过。 |
| FC-07 universal package toolchain blockers | package 工具链 | fixed | 已安装缺失 targets、`zig` 和 `cargo-zigbuild`；完整 package 通过并生成 `/private/tmp/homie-agent-manifest-package/homie.app`。 |

## 3. Findings

| 严重度 | 类别 | 位置 | 证据与影响 | 处置 |
|---|---|---|---|---|
| medium | Correctness | `Sources/HomieCore/AgentDescriptor.swift` | 第一轮复查发现 Swift `Resume.argv(id: nil)` 将 `.subcommand` 无 id 情况视为可生成 bare token，但 Rust Engine `resume_args(None)` 只支持 `.flag` bare token；若后续出现 `subcommand` manifest，会让 Swift UI/CLI 暴露 Rust 无法执行的 resume 语义。 | fixed：`.subcommand` + nil id 改为 `nil`，并更新 `AgentKindTests.resumeSpecsBuildTheRightArgv`。 |
| medium | Error Handling | `scripts/sync-agent-manifests.sh` | 原实现先删除 mirror 再逐个 copy；若复制过程中中断或失败，会留下半同步 Swift resource mirror，随后 Swift build/CLI 可能读取不完整 catalog。 | fixed：改为 staging 目录复制并校验数量后，再替换 mirror 内 JSON 文件。 |
| medium | Portability | `scripts/check.sh` | 第一轮新增 Swift Testing flags 时写死 CLT 路径；在 GitHub Actions 或用户选择完整 Xcode 时，`Testing.framework` 与 `lib_TestingInterop.dylib` 可能位于 `xcode-select -p` 下的 MacOSX platform 路径，导致 `swift test` 找不到测试运行时。 | fixed：按 `xcode-select -p` 派生 CLT/Xcode 两组候选目录，只在实际存在时添加 `-F`/`-rpath`。 |
| high | Correctness | `scripts/check.sh` | 第二轮验证发现 shell 语法错误：`for interop_dir ...; do` 以 `fi` 结束，`./scripts/check.sh` 立即退出。 | fixed：改为 `done`；`bash -n scripts/*.sh homie/scripts/*.sh` 通过。 |

## 4. 对抗式复盘

- 反例/边界：
  - Swift mirror 被手工改一个行为字段：`scripts/check-agent-manifest-drift.sh` 必须失败。已由 FC-03 覆盖。
  - `scripts/sync-agent-manifests.sh` 中途失败：不能留下部分 mirror。已改 staging 目录。
  - Xcode runner 而非 CLT runner：Swift Testing framework 路径不能写死。已改为 `xcode-select` 派生。
  - `.subcommand` 无 id：Rust 不支持 bare subcommand resume，Swift 不能错误展示可 resume。已修复。
  - package 完整路径：必须验证 remote helpers、manifest 数量、codesign 和启动 smoke。已由 FC-07 和启动 smoke 覆盖。
- 撤回或降级：
  - 未把 `.agents/skills/build-gpui-apps/**` 和 `skills-lock.json` 纳入本次 review；它们是当前工作树未跟踪项，但与本次 Homie 代码变更无直接关系。
- 新增修复：
  - `legacy_remote::is_homie_generated_name` 收紧为 8 位 hex，是 FC-10 全量 gate 暴露出的真实安全边界问题。
  - `license-policy.json` 移除 stale `SwiftTerm` entry，是全量 gate 暴露出的供应链策略问题。

## 5. 修复摘要

- 新增并验证 agent manifest drift gate 与 sync 脚本。
- Swift resource manifest 同步为 Rust Engine manifest 生成镜像。
- 文档和 Swift 注释改为 Rust manifest 是唯一人工源。
- Swift resume 语义对齐 Rust Engine。
- `scripts/check.sh` 接入 manifest drift gate，并修复 Swift Testing framework/rpath。
- 修复 inspector 测试清理，避免 GPUI scheduler cleanup abort。
- 修复 legacy remote generated-name 判定。
- 清理 stale SwiftTerm license policy entry。

## 6. 验证结果

| 命令 | 结果 | 说明 |
|---|---|---|
| `bash -n scripts/*.sh homie/scripts/*.sh` | pass | Shell 语法通过。 |
| `scripts/check-agent-manifest-drift.sh` | pass | Rust source 与 Swift mirror 一致。 |
| `swift test --package-path . --filter HomieCoreTests ...` | pass | 22 tests passed。 |
| `./scripts/check.sh` | pass | `All contributor checks passed`。 |
| `HOMIE_DIST_DIR=/private/tmp/homie-agent-manifest-package homie/scripts/package.sh` | pass | 生成 signed universal `.app`，包含 20 manifests 和 3 remote helpers。 |
| `codesign --verify --deep --strict /private/tmp/homie-agent-manifest-package/homie.app` | pass | Bundle 签名验证通过。 |
| startup smoke: bundled `homied-rs` + bundled CLI `status`/`doctor` | pass | daemon reachable；status 返回 `No active sessions.`；启动日志无 `error/failed/panic/cannot/refused/missing`。 |

## 7. 剩余风险

- 当前尚未执行最终 E2E 和提交推送；这属于 dev-loop 后续步骤。
- 工作树存在与本次任务无关的未跟踪 `.agents/skills/build-gpui-apps/**` 和 `skills-lock.json`，未纳入本次 review，也不应随本次变更提交。
