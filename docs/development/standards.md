# Homie 开发规范

## 1. 总原则

1. PRD/spec 先行，组件 spec 影响分析先行。
2. 先跑通最小端到端闭环，再扩展复杂能力。
3. 优先复用成熟包，新增依赖必须有 spec 或 research 依据。
4. Rust 是产品核心事实源，Swift 是平台集成边界。
5. SQLite 是 V1 本地事实源，禁止模块各自维护独立状态文件。
6. 安全数据默认敏感，所有日志、事件、指标和报告必须脱敏。

## 2. Rust 规范

### 2.1 Toolchain

- 使用 `rust-toolchain.toml` pin Rust 版本和 `rustfmt`、`clippy`。
- `Cargo.lock` 必须提交，因为 Homie 是应用项目。
- workspace 使用 resolver 2。

### 2.2 Crate 设计

- crate 边界按职责，不按技术偏好。
- crate public API 只暴露必要类型，内部实现默认 private。
- domain 类型放 `homie-proto` 或对应领域 crate，不能从 UI 层反向定义。
- `homie-storage` 提供 repository API，不把裸 SQL 泄漏到 UI/runtime 各处。

### 2.3 Error

- library crate 使用 `thiserror` 定义可匹配错误。
- binary/CLI 使用 `anyhow` 添加上下文。
- protocol error 必须映射到稳定 safe error code。
- 不用 `unwrap()`/`expect()` 处理用户输入、IO、网络、SQLite、secret、agent process。

### 2.4 Async

- async runtime 统一使用 `tokio`。
- 阻塞 PTY/read/process wait 不得占用 async worker；用 dedicated thread 或 `spawn_blocking`，并记录理由。
- long-running task 必须支持 cancel/shutdown。
- runtime shutdown 必须 flush SQLite、usage/context 和必要 output index。

### 2.5 SQLite

- 所有连接启用 `PRAGMA foreign_keys = ON`。
- 默认 WAL 模式。
- migration forward-only，不写兼容旧 schema 的 fallback。
- 写操作使用事务。
- high-volume output bytes 不进 SQLite blob。
- schema 变更必须有 migration test 和 repository test。

### 2.6 LLM / Secret

- provider raw key 只存在 encrypted local secret envelope 和短期内存中。
- 内存中的 secret 使用 `secrecy`/`zeroize` 或等价模式。
- agent env/config 只能拿 virtual key 和 local proxy URL。
- logs/events/context/memory/metrics/report 禁止 raw key、Authorization、cookie、完整 tool args/result。

### 2.7 Tests

- 新功能先写可失败测试或 contract fixture。
- Bug 修复必须先有能复现问题的 RED test。
- Storage、protocol、credential、LLM proxy、permission、runtime shutdown 属于 P0 测试范围。
- 测试禁止依赖真实 provider key 和真实用户 HOME，除非是显式 smoke/E2E。

## 3. Swift 规范

Swift 只用于 macOS native integration。

### 3.1 Target 边界

- `HomieNativeHost`：app host、bundle、窗口生命周期、系统菜单。
- `HomieSystemBridge`：通知、权限、Keychain 后续桥接、menu bar 等。
- Swift target 不实现 agent runtime、LLM proxy、SQLite repository。

### 3.2 Interop

- Swift 调 Rust 优先通过 IPC/CLI。
- FFI 必须有薄 wrapper，并有 ABI 边界测试。
- Swift 与 Rust 共享的数据必须来自 `homie-proto` 生成或手写稳定 schema，不用临时 JSON。

### 3.3 Swift Quality

- Swift 代码必须通过 `swift build`、`swift test`。
- macOS-only API 必须隔离到 Swift 或 Rust `cfg(target_os = "macos")` 模块。
- Swift package 不得引入业务层第三方库，除非组件 spec 批准。

## 4. Dependency Policy

新增依赖必须满足：

- 在 PRD/spec 或 research 文档中有用途说明。
- 说明为什么不用标准库或已有依赖。
- 说明 license 是否可用于桌面应用分发。
- 说明是否新增 network/filesystem/process/secret 能力。

优先顺序：

1. 已在本项目使用的依赖。
2. 已在相近 Rust + GPUI 桌面工程验证过的依赖。
3. Rust 生态成熟维护依赖。
4. 自研，仅在以上都不满足且 spec 写明原因时允许。

## 5. Commit / Evidence

- 每个实质变更必须关联 Beads issue。
- 提交前运行对应 quality gate。
- evidence report 记录实际命令、退出码、结果和未运行原因。
- 不得把未运行的门禁写成 pass。

## 6. 禁止事项

- 禁止为过时接口写兼容层，除非用户明确要求。
- 禁止在实现中静默扩大 scope。
- 禁止降低测试断言来让实现通过。
- 禁止提交真实 secret 或本机 `.beads/`、runtime 数据、SQLite 数据库。
- 禁止 UI 直接写 runtime/storage 状态。
