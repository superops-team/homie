# Code Review Round 2

## 1. 复核重点

- 是否引入隐性兼容层；
- 是否让后台 shell 仍在启动阶段执行；
- 是否误删仍被真实路径使用的 Swift code；
- 是否降低用户创建 agent 的成功率；
- 是否有测试/验证口径绕过真实路径。

## 2. 隐性风险检查

| 风险 | 结论 | 处理/说明 |
|------|------|-----------|
| Rust Engine startup 仍可能触发 shell | pass | `homied-rs` startup 仅调用 account metadata/cache/fallback，不执行 `Command::new(shell)` |
| PATH refresh 被自动启动触发 | pass | app 启动连接路径不触发 refresh；new-agent picker 用户触发时才触发 `RefreshAgentCatalog` |
| PATH refresh 高频重复 | mitigated | refresh 使用 cache TTL，默认 300s 内复用 cache |
| Swift daemon fallback 残留 | pass | Package/scripts/product docs 扫描无残留；Swift daemon/holder sources 和 tests 删除 |
| Swift target 误删导致 CLI 不可用 | pass | `swift build` 通过，保留 `HomieCore`、`HomieProtocol`、`HomieMCP`、`homie-cli` |
| 启动 exec probe 漏掉进程 | partial | wrapper PATH 捕获 `ssh/node/rsync/gh/lsof/open/osascript`，可证明普通启动不调用这些工具；无法证明系统内核层所有 exec，但满足当前 P0 验收 |
| full Rust test suite 未运行 | accepted risk | 本变更执行了 targeted TDD、cargo check 和 functional cases；全 workspace test 可作为后续质量 gate 单独稳定化 |

## 3. SDD/TDD 合规性

- TDD：先补 `environment` 单测，再实现 startup fallback/cache/override 和 refresh 行为。
- SDD：Rust Engine 作为唯一 daemon/supervisord 的架构约束已写入 PRD、OpenSpec 和 docs。
- 功能验证：FC-01 至 FC-08 均有实际命令和证据文件。

## 4. 二轮结论

无 P0/P1 代码问题。可以进入 E2E/准出验证与 release readiness 汇总。
