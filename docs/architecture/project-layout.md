# Homie 大型项目目录结构规范

## 1. 目标

Homie 是 Rust + GPUI 为主体、必要时包含 Swift/macOS native host 的桌面应用。目录结构必须支持长期演进：

- Rust 负责产品核心、runtime、LLM proxy、SQLite、agent adapter、context/memory/task/orchestrator。
- Swift 只负责 macOS 原生集成边界，例如 app bundle host、系统权限、Keychain 后续集成、通知、菜单栏、系统服务桥接。
- Swift 不承载 Homie 业务事实源，不直接写 SQLite 业务表，不绕过 Rust protocol。
- Rust 与 Swift 通过稳定 FFI、CLI 或 IPC 边界交互，不共享隐式全局状态。

## 2. 目标目录

```text
homie/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── Package.swift                     # Swift package, only when native host is introduced
├── crates/
│   ├── homie-app/                    # GPUI desktop binary
│   ├── homie-ui/                     # GPUI design system and shared widgets
│   ├── homie-term/                   # terminal grid, input, selection, search
│   ├── homie-proto/                  # protocol DTOs, events, error envelope
│   ├── homie-client/                 # async client for runtime/proxy
│   ├── homie-runtime/                # process/session/PTY/status/event runtime
│   ├── homie-agents/                 # runtime descriptor, agent profile, adapter contract
│   ├── homie-llm/                    # provider config, virtual key, OpenAI-compatible proxy
│   ├── homie-context/                # session context and workspace facts
│   ├── homie-memory/                 # durable memory boundary
│   ├── homie-task/                   # task model and local task state
│   ├── homie-storage/                # SQLite schema, migrations, repositories
│   ├── homie-observability/          # logs, metrics projection, evidence helpers
│   └── homie-cli/                    # doctor, runtime status, smoke commands
├── swift/
│   ├── HomieNativeHost/              # macOS app/native integration host
│   ├── HomieSystemBridge/            # permissions, notification, menu bar, keychain bridge
│   └── Tests/
├── assets/
│   ├── app/
│   ├── icons/
│   └── agent-descriptors/
├── migrations/
│   └── sqlite/
├── scripts/
│   ├── dev/
│   ├── quality/
│   ├── package/
│   └── release/
├── tests/
│   ├── fixtures/
│   ├── e2e/
│   └── smoke/
├── docs/
│   ├── architecture/
│   ├── development/
│   ├── research/
│   ├── security/
│   └── verification/
├── prd-spec/
├── specs/
├── openspec/
└── .githooks/
```

## 3. Rust Workspace 分层

| Crate | 允许依赖 | 禁止依赖 | 职责 |
|-------|----------|----------|------|
| `homie-proto` | `serde`, `time`, ID/error 基础包 | UI/runtime/storage 实现 | 稳定 wire model、事件、错误 |
| `homie-storage` | `homie-proto`, `rusqlite` | UI/runtime process | SQLite schema、migration、repository |
| `homie-agents` | `homie-proto`, `homie-storage`, schema 包 | UI | runtime descriptor、profile、permission config |
| `homie-llm` | `homie-proto`, `homie-storage`, HTTP 包 | UI | provider、virtual key、proxy、usage metrics |
| `homie-context` | `homie-proto`, `homie-storage` | UI | session context、workspace facts |
| `homie-runtime` | proto/storage/agents/llm/context | UI | PTY/process/session/event runtime |
| `homie-client` | `homie-proto`, `tokio` | runtime internals | app/CLI 调 runtime 的 client |
| `homie-term` | `homie-proto`, GPUI | runtime internals | terminal grid 渲染 |
| `homie-ui` | GPUI | runtime/storage direct | UI tokens/components |
| `homie-app` | ui/term/client/proto | runtime/storage direct | 桌面应用入口 |
| `homie-cli` | client/proto | UI/runtime direct state | doctor/status/smoke |

规则：

1. UI 只能通过 `homie-client` 访问 runtime/proxy。
2. runtime 不依赖 app/ui。
3. storage 不依赖 runtime，避免 migration 和业务生命周期耦合。
4. protocol DTO 不能引用具体 repository、GPUI 类型、tokio channel 类型。
5. agent/profile/permission schema 必须可由 tests 单独加载和验证。

## 4. Swift 边界

Swift 只在确有必要时加入，默认不承载业务逻辑。

允许：

- macOS app bundle host。
- NSApplication / NSWindow / menu bar / notification bridge。
- macOS permission prompt 或系统服务桥接。
- 后续 Keychain bridge，如果 encrypted envelope 决策升级。

禁止：

- Swift 直接实现 agent session runtime。
- Swift 直接修改 Homie SQLite 业务表。
- Swift 直接持有 provider raw key 进行 LLM 请求。
- Swift 与 Rust 共享未版本化的 ad hoc JSON。

Swift 与 Rust 交互方式优先级：

1. IPC/protocol：适合 runtime/control/diagnostic。
2. CLI：适合 packaging、doctor、一次性命令。
3. FFI：只用于高频 native UI/system API 且必须有明确 ABI wrapper。

## 5. 数据目录

运行时数据不放仓库。

```text
macOS: ~/Library/Application Support/Homie/
Linux: ~/.local/share/homie/
Windows: %APPDATA%/Homie/
```

V1 数据：

```text
homie.sqlite
homie.sqlite-wal
homie.sqlite-shm
secrets/
runtime/output/<session>.log
```

## 6. 测试目录

| 路径 | 用途 |
|------|------|
| crate 内 `tests/` | crate 级集成测试 |
| crate 内 `src/**` test module | 单元测试 |
| `tests/fixtures/` | protocol、SQLite、agent descriptor、LLM provider fixtures |
| `tests/e2e/` | app/runtime/proxy 端到端脚本或 harness |
| `tests/smoke/` | release 前最小真实路径验证 |

## 7. 文档落点

| 文档 | 路径 |
|------|------|
| 单次需求 | `prd-spec/` |
| 长期组件合同 | `specs/` |
| 单次执行计划 | `openspec/changes/<change-id>/` |
| 调研 | `docs/research/` |
| 开发规范 | `docs/development/` |
| 架构规则 | `docs/architecture/` |
| 验证证据 | `docs/verification/<change-id>/` |

## 8. 命名规则

- Rust crate：`homie-*`。
- Swift target：`Homie*`。
- SQLite table：snake_case plural 或领域名，如 `agent_profiles`。
- protocol method：dot-separated，如 `session.spawn`。
- event name：dot-separated past tense，如 `session.updated`。
- Beads change id / OpenSpec id：kebab-case。
