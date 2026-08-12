# startup-background-shell-invisibility OpenSpec Tasks

## 1. P0-A Rust 启动无感

### T1: 建立 Rust environment resolver seam

- 描述：从 `homied-rs.rs` 中拆出可测试环境解析边界，启动阶段只能读取 login shell/fallback/cache，不执行 shell。
- 涉及文件：
  - `homie/crates/homie-engine/src/bin/homied-rs.rs`
  - `homie/crates/homie-engine/src/lib.rs`
  - 新建 `homie/crates/homie-engine/src/environment.rs` 或等价模块
- 验收：
  - FC-01 可执行且先红后绿。
  - `homied-rs` startup 不调用 path capture executor。
- 关联 Case：FC-01、FC-07
- 优先级：P0

### T2: 实现 fallback/cache PATH 启动策略

- 描述：启动阶段使用 fallback/cache PATH；不依赖交互 shell。缓存格式保持简单、可删除、无 secret。
- 涉及文件：
  - `homie/crates/homie-engine/src/environment.rs`
  - `homie/crates/homie-engine/src/bin/homied-rs.rs`
  - app support path helpers
- 验收：
  - 无 cache 时使用 fallback。
  - 有 cache 时优先 cache。
  - cache 读取失败回退 fallback。
- 关联 Case：FC-01、FC-02、FC-03、FC-07
- 优先级：P0

### T3: 实现 lazy PATH refresh

- 描述：将 PATH refresh 移到用户触发或延迟触发路径；refresh 静默、限时、可取消，输出只提取 PATH-like 行。
- 涉及文件：
  - `homie/crates/homie-engine/src/environment.rs`
  - `homie/crates/homie-engine/src/control.rs`
  - 如需：新增 control method 或 internal refresh trigger
- 验收：
  - refresh 成功后更新 readiness。
  - refresh timeout 不阻塞 startup。
  - refresh stdout/stderr 不进入 UI payload。
- 关联 Case：FC-02、FC-03、FC-04、FC-07
- 优先级：P0

### T4: agent readiness 从启动关键路径解耦

- 描述：确保 `agent.readiness` 不触发 startup eager shell；必要时支持 checking/unknown 或 fallback/cache readiness。
- 涉及文件：
  - `homie/crates/homie-engine/src/control.rs`
  - `homie/crates/homie-proto/src/methods.rs`
  - `homie/crates/homie-app/src/store/mod.rs`
  - `homie/crates/homie-app/src/sidebar/view.rs`
- 验收：
  - 启动首帧不等待 readiness shell。
  - agent picker 可在 refresh 后局部更新。
- 关联 Case：FC-02、FC-04、FC-07
- 优先级：P0

### T5: 普通启动 remote/browser 任务按需化

- 描述：检查并修正普通启动阶段触发 remote restore、browser sidecar、ssh/node/rsync 的路径；保留用户触发语义。
- 涉及文件：
  - `homie/crates/homie-engine/src/bin/homied-rs.rs`
  - `homie/crates/homie-engine/src/browser.rs`
  - `homie/crates/homie-engine/src/remote/*`
  - `homie/crates/homie-engine/src/pr_monitor.rs`
  - `homie/crates/homie-engine/src/governor.rs`
- 验收：
  - FC-04 无普通启动阶段 user-triggered exec。
  - remote/browser 仍可在用户触发路径运行。
- 关联 Case：FC-04、FC-07
- 优先级：P0

## 2. P0-B Swift daemon 清理

### T6: 绘制 Swift target 保留/删除清单

- 描述：梳理 `Package.swift` target 依赖，明确保留 Swift protocol/core/CLI/MCP/macOS glue，删除 daemon/holder。
- 交付物：
  - `docs/verification/startup-background-shell-invisibility/swift-target-cleanup-inventory.md`
- 验收：
  - 每个 Swift target 标记 keep/delete/rehome。
  - 删除不会依赖 fallback 或 legacy。
- 关联 Case：FC-05、FC-06
- 优先级：P0

### T7: 删除 Swift daemon/holder source、targets、tests

- 描述：删除 `Sources/homied/`、`Sources/homied-holder/`、`Sources/HomieDaemonKit/`、`Sources/HomieHolderKit/` 及 daemon tests；更新 `Package.swift`。
- 涉及文件：
  - `Package.swift`
  - `Sources/`
  - `Tests/`
- 验收：
  - `swift package dump-package` 不含 daemon/holder target。
  - `swift build` 仍通过。
- 关联 Case：FC-05、FC-06、FC-08
- 优先级：P0

### T8: 清理 docs/scripts 中 Swift daemon 叙述和 fallback

- 描述：更新 README、CONTRIBUTING、scripts，移除 Swift daemon/engine/fallback 描述。
- 涉及文件：
  - `README.md`
  - `CONTRIBUTING.md`
  - `Package.swift` 注释
  - `homie/scripts/dev.sh`
  - `homie/scripts/package.sh`
  - 其他 grep 命中的 docs/scripts
- 验收：
  - FC-08 无未解释残留。
  - dev/package scripts 不再查找或复制 Swift daemon/holder。
- 关联 Case：FC-05、FC-06、FC-08
- 优先级：P0

## 3. 验证与评审

### T9: 启动 exec probe 与 heavy rc 验证脚本

- 描述：实现或补充功能验证脚本，记录启动阶段 shell/exec 调用。
- 涉及文件：
  - `docs/verification/startup-background-shell-invisibility/run-heavy-rc-smoke.sh`
  - `docs/verification/startup-background-shell-invisibility/run-startup-exec-probe.sh`
- 验收：
  - FC-03、FC-04 可执行并留证。
- 关联 Case：FC-03、FC-04
- 优先级：P0

### T10: 功能验证 Case 执行

- 描述：逐条执行 FC-01 至 FC-08，记录实际输出、退出码和证据路径。
- 交付物：
  - `docs/verification/startup-background-shell-invisibility/functional-verification-report.md`
- 验收：
  - P0 Case 全部 pass。
- 关联 Case：FC-01 至 FC-08
- 优先级：P0

### T11: 两轮代码审查

- 描述：执行显性问题审查与隐性问题复审，覆盖实现与 Case 对齐。
- 交付物：
  - `docs/verification/startup-background-shell-invisibility/code-review-round-1.md`
  - `docs/verification/startup-background-shell-invisibility/code-review-round-2.md`
- 验收：
  - 无 P0/P1 未处理问题。
- 关联 Case：FC-01 至 FC-08
- 优先级：P0

### T12: release readiness report

- 描述：汇总 spec、OpenSpec、功能验证、代码审查、编译门禁、残余风险。
- 交付物：
  - `docs/verification/startup-background-shell-invisibility/release-readiness-report.md`
- 验收：
  - Beads `homie-f21` 可基于报告关闭。
- 关联 Case：FC-01 至 FC-08
- 优先级：P0
