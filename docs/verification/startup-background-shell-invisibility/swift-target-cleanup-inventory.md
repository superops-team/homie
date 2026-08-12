# Swift Target Cleanup Inventory

## 1. 决策

Rust Engine 是 Homie 唯一 daemon/supervisord。Swift 侧不保留后台 daemon、holder、parallel supervisor、fallback 或 compatibility adapter。

## 2. 保留目标

| Target | 类型 | 原因 |
|--------|------|------|
| `HomieCore` | library | Swift CLI/MCP/protocol 共享 agent manifest、session/domain 类型 |
| `HomieProtocol` | library | Swift CLI/MCP 与 Rust wire DTO 对齐所需 |
| `HomieMCP` | library | Swift CLI 的 MCP stdio 工具实现仍在真实路径使用 |
| `homie-cli` | executable | 打包脚本仍将本地自动化 CLI 放入 app bundle |
| `HomieCoreTests` | test | 保留 core schema/manifest 测试 |
| `HomieProtocolTests` | test | 保留 wire/protocol 测试 |
| `HomieCLITests` | test | 保留 CLI command grammar 测试 |

## 3. 删除目标

| Target | 类型 | 删除原因 |
|--------|------|----------|
| `CHomiePTY` | C target | Swift daemon PTY seam，Rust `homie-pty` 已承接 |
| `HomieHolderKit` | library | Swift holder 实现，不保留 legacy |
| `HomieDaemonKit` | library | Swift daemon 实现，不保留 legacy |
| `HomieClient` | library | Swift app-side daemon client，不在当前产品路径使用 |
| `HomieDetection` | library | Swift daemon status detection，不在当前产品路径使用 |
| `HomieGit` | library | Swift daemon git worktree helper，不在当前产品路径使用 |
| `homied` | executable | Swift daemon executable，不保留 fallback |
| `homied-holder` | executable | Swift holder executable，不保留 fallback |
| `HomieDaemonKitTests` | test | 仅覆盖删除的 Swift daemon/holder |
| `HomieDetectionTests` | test | 仅覆盖删除的 Swift detection engine |

## 4. 脚本和文档清理点

- `Package.swift` 不再声明 Swift daemon/holder/detection/client/git targets。
- `README.md`、`CONTRIBUTING.md` 不再描述 Swift daemon 为 engine。
- `homie/scripts/dev.sh` 不再寻找 installed Swift `homied` fallback。
- `homie/scripts/package.sh` 只构建 Swift CLI，不复制 Swift daemon/holder。

## 5. 验证

- `swift package dump-package` 不应包含 `homied`、`homied-holder`、`HomieDaemonKit`、`HomieHolderKit`。
- `swift build` 只构建保留 Swift targets。
- `rg -n "HomieDaemonKit|HomieHolderKit|homied-holder|Swift daemon|Swift engine"` 不应在产品 docs/scripts/source 中出现未解释残留。
