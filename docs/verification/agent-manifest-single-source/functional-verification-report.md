# Agent Manifest 单源化功能验证执行报告

## 1. 总体结论

- 结论：功能验证 Case 已全部通过；app 可完整编译打包并启动随包 Engine，启动 smoke 未发现 error/failed/panic 日志。
- 通过项：manifest drift 检测、同步生成镜像、手工漂移阻断、Swift Core/CLI/Protocol focused 回归、Rust Engine manifest focused 回归、user overrides 边界、文档路径扫描、完整 package、全量本地质量门禁、启动 smoke。
- 阻塞项：无。

按 dev-loop 门禁，本阶段可进入两轮 code review、最终 E2E 和提交推送。

## 2. Case 执行结果

| Case | 状态 | 证据 | 说明 |
|------|------|------|------|
| FC-01 当前双源漂移可被检测 | 通过 | `fc-01-drift-detected.log` | 当前 20 个 manifest drift 被 check 捕获，非 0 退出并提示 sync 脚本。 |
| FC-02 同步脚本能生成 Swift mirror | 通过 | `fc-02-sync-green.log` | `scripts/sync-agent-manifests.sh` 同步 20 个 manifest，drift check 通过。 |
| FC-03 手工改 Swift mirror 被阻断 | 通过 | `fc-03-manual-swift-drift-blocked.log` | 修改 `codex.agent.shortLabel` 后 check 失败，恢复后通过。 |
| FC-04 Swift AgentCatalog 继续加载 mirror | 通过 | `fc-04-swift-core-tests.log` | `HomieCoreTests` 22 个测试通过。执行时需要显式 Testing framework/rpath。 |
| FC-05 Swift CLI/MCP vocabulary 不回退 | 通过 | `fc-05-swift-cli-protocol-tests.log` | `HomieCLITests` 20 个测试、`HomieProtocolTests` 14 个测试通过。 |
| FC-06 Rust Engine manifest decode 不回退 | 通过 | `fc-06-rust-engine-manifest-tests.log` | manifest decode、agent authority、MCP tools 测试通过。 |
| FC-07 package 仍从 Rust source catalog 复制 | 通过 | `fc-07-package-manifest-source.log` | 完整 package 成功，universal app、CLI、Engine、3 个 remote helpers、20 个 manifest 和 ad-hoc codesign 均完成。 |
| FC-08 user overrides 不参与同步/check | 通过 | `fc-08-user-overrides-ignored.log` | 临时 HOME override 不影响 drift check。 |
| FC-09 文档不再指向 Swift mirror 作为人工源 | 通过 | `fc-09-doc-source-path-scan.log` | README/CONTRIBUTING 无误导；仅脚本路径与 generated mirror 注释保留 Swift mirror 路径。 |
| FC-10 全量本地质量门禁 | 通过 | `fc-10-full-check.log` | Shell/release guards、Swift tests、Rust fmt/clippy/tests、license policy 全部通过。 |

## 3. 关键修复记录

1. 新增 `scripts/check-agent-manifest-drift.sh`。
2. 新增 `scripts/sync-agent-manifests.sh`。
3. 将 `Sources/HomieCore/Resources/manifests/` 同步为 Rust Engine manifest 的生成镜像。
4. 将 drift check 接入 `scripts/check.sh` 和 `.github/workflows/ci.yml`。
5. 更新 README、CONTRIBUTING、Package.swift 与 Swift Core 注释，明确 Rust manifest 是唯一人工源。
6. 修复 Swift `AgentDescriptor.Resume` 对 `style: "flag"` 的语义解释，使其与 Rust Engine `resume_args(None)` 对齐：无 id 时可生成 bare token，有 id 时生成 token + id。
7. 更新 `Tests/HomieCoreTests/AgentKindTests.swift`，使测试断言以 Rust 权威 manifest 语义为准。
8. 补充 `scripts/check.sh` 的 Swift Testing framework/rpath 参数，使 Swift test 阶段能在当前 CLT/Xcode 布局下运行。

## 4. 阻塞详情

### 4.1 FC-07 package 验证

命令：

```sh
env PATH=/Users/bytedance/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin \
  HOMIE_DIST_DIR=/private/tmp/homie-agent-manifest-package \
  homie/scripts/package.sh
```

结果：

- 完整 package 成功，产物：`/private/tmp/homie-agent-manifest-package/homie.app`。
- 主 app 为 universal binary：x86_64 + arm64。
- `homied-rs` 与 `homie-holder` 为 universal binary：x86_64 + arm64。
- remote helper catalog 构建 3 个 artifact。
- bundled agent manifests 数量为 20。
- `codesign --verify --deep --strict /private/tmp/homie-agent-manifest-package/homie.app` 通过。

环境修复：

- 安装 `x86_64-apple-darwin`、`x86_64-unknown-linux-musl`、`aarch64-unknown-linux-musl` targets。
- 安装 `zig` 与 `cargo-zigbuild`，用于 remote helper cross build。

### 4.2 FC-10 全量 gate 修复记录

命令：

```sh
./scripts/check.sh
```

结果：

- Shell/release guards 通过。
- Agent manifest mirror check 通过。
- Swift CLI/protocol support 通过：`Test run with 56 tests in 2 suites passed`。
- Rust fmt/clippy/tests 通过。
- Dependency license policy 通过。

修复内容：

- `WorkbenchInspector::set_visible(false)` 显式取消 passive `refresh_task` 与 `review_task`，防止 GPUI test scheduler 跨线程析构本地任务。
- `legacy_remote::is_homie_generated_name` 收紧为 `homie-` + 正好 8 位 lowercase hex，符合旧 session id 截断事实并避免误认 `homie-1`。
- 移除 license policy 中已经不存在于 `Package.resolved` 的 stale `SwiftTerm` 手工 review 条目。

### 4.3 启动 Smoke

命令：

```sh
HOMIE_APP_SUPPORT=/private/tmp/homie-startup-smoke \
  /private/tmp/homie-agent-manifest-package/homie.app/Contents/Resources/bin/homied-rs
HOMIE_SOCKET=/private/tmp/homie-startup-smoke/daemon.sock \
  /private/tmp/homie-agent-manifest-package/homie.app/Contents/Resources/bin/homie status
HOMIE_SOCKET=/private/tmp/homie-startup-smoke/daemon.sock \
  /private/tmp/homie-agent-manifest-package/homie.app/Contents/Resources/bin/homie doctor
```

结果：

- Engine 启动成功并创建 `/private/tmp/homie-startup-smoke/daemon.sock`。
- `homie status` 返回 `No active sessions.`。
- `homie doctor` 返回 daemon reachable，并识别本机 `claude` 与 `codex`。
- 启动日志扫描 `error|failed|panic|cannot|refused|missing` 无命中。

## 5. 后续处理建议

1. 进入两轮 code review。
2. 执行最终 E2E。
3. E2E 通过后再提交/推送。
