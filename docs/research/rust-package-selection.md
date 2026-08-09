# Homie V1 Rust 包选型调研

## 1. 结论

V1 应优先复用成熟 crate，避免重复造轮子。推荐基线：

| 领域 | 推荐包 | 状态 | 说明 |
|------|--------|------|------|
| Desktop UI | `gpui`, `gpui_platform` | 采用 | 与项目方向一致，固定 git revision，升级必须显式评估 |
| Async/runtime | `tokio` | 采用 | control socket、process、HTTP proxy、后台任务统一基座 |
| 序列化 | `serde`, `serde_json` | 采用 | protocol、SQLite JSON 字段、config snapshot |
| SQLite | `rusqlite` + `bundled` | 采用 | 本地单用户数据库，WAL、migration、事务、外键可控 |
| HTTP server/proxy | `axum`, `tower`, `tower-http` | 采用 | OpenAI-compatible proxy、health、metrics endpoint |
| HTTP client | `reqwest` + `rustls-tls` + `json` + `stream` | 采用 | provider 转发、SSE/streaming 基础 |
| Terminal emulation | `alacritty_terminal` | 采用 | 避免自研 VT parser，runtime 维护 headless screen |
| PTY | `portable-pty` 或 Unix seam + `libc` | V1 评估后定 | macOS 可直接 unix seam；若要提前跨平台则选 `portable-pty` |
| File watching | `notify` | 采用 | transcript、config、workspace watch |
| Fuzzy search | `nucleo-matcher` | 采用 | command palette、quick open、session search |
| CLI | `clap` derive | 采用 | `homie doctor`、runtime status、session list |
| Logging/tracing | `tracing`, `tracing-subscriber` | 采用 | structured logs、safe fields、debug diagnostics |
| Error handling | `thiserror`, `anyhow` | 采用 | library 用 `thiserror`，binary/CLI 用 `anyhow` |
| IDs | `uuid` v7 或 `ulid` | 采用其一 | 需要可排序 ID 时优先 `uuid` v7；schema 中统一存 TEXT |
| Time | `time` | 采用 | `OffsetDateTime` + serde，避免 chrono 过重 |
| Decimal/cost | `rust_decimal` | 采用 | token cost、estimated_cost、pricing |
| Secret memory | `secrecy`, `zeroize` | 采用 | raw key 在内存中包装和 drop 清零 |
| Secret envelope | `age` 或 RustCrypto primitives | 先做 ADR | V1 已决策 encrypted local secret envelope，需在组件 spec 里定格式 |
| JSON schema | `schemars` | 采用 | agent/runtime/profile/permission schema 生成和校验 |
| Assets | `rust-embed` 或 `include_dir` | 采用其一 | 打包内置 assets、agent descriptors、default config |
| macOS glue | `objc2`, `objc2-app-kit`, `objc2-foundation` | macOS 专用 | 通知、菜单栏、窗口细节、Keychain 后续可扩展 |

## 2. 选型原则

1. 已被相近 Rust + GPUI agent 桌面工程验证过的依赖优先。
2. 能直接承载复杂领域逻辑的成熟库优先，例如 terminal emulation、SQLite、HTTP proxy、file watching。
3. 安全敏感能力不使用小众未审计封装做黑盒；encrypted secret envelope 需要在组件 spec 中明确格式、KDF、AEAD、AAD、version header 和 threat model。
4. V1 不引入 ORM。SQLite 关系复杂但可控，使用 `rusqlite` 手写 SQL + migration，避免 Diesel/SeaORM 在早期放大复杂度。
5. 不引入 MCP proxy 库到 V1 实现。MCP server 配置先入库，实际 proxy 后续单独选型。

## 3. 按组件选型

### 3.1 `homie-app` / `homie-ui`

推荐：

- `gpui`
- `gpui_platform`
- `image`
- `nucleo-matcher`
- `notify`
- `tokio`
- macOS-only: `objc2`, `objc2-app-kit`, `objc2-foundation`, `objc2-user-notifications`

原因：

- GPUI 是项目目标框架，也是 Zed 系列 Rust-native UI 的直接路径。
- `nucleo-matcher` 足够支撑 command palette、session search、quick open。
- `notify` 用于 config/profile/transcript/workspace 变化监听。

约束：

- `gpui` 使用 git revision pin，不用浮动分支。
- UI crate 不依赖 `homie-runtime`，只依赖 `homie-client` 和 `homie-proto`。

### 3.2 `homie-runtime`

推荐：

- `tokio`
- `alacritty_terminal`
- `regex`
- `serde`, `serde_json`
- Unix/macOS: `libc`
- PTY 备选：`portable-pty`

原因：

- `alacritty_terminal` 已是成熟终端 emulator 组件，比自研 VT parser 更安全。
- PTY 是平台 seam。若 V1 只做 macOS/Unix，直接 unix seam + tests 更可控；若 V1 要提前 Windows，优先评估 `portable-pty`。

约束：

- runtime 不依赖 UI。
- PTY trait 必须保留 Windows ConPTY seam。
- output bytes 不写 SQLite blob，写文件并由 SQLite 索引。

### 3.3 `homie-proto` / `homie-client`

推荐：

- `serde`
- `serde_json`
- `tokio`
- `sha2`
- `thiserror`

原因：

- Wave 1A transport 使用 fixed binary frame header；control/event/stream metadata payload 使用 bounded Serde JSON，terminal payload 使用 raw binary。
- client 需要 request correlation、watch/broadcast channel、reconnect/backoff。
- daemon hello 使用 SHA-256 标识实际 executable。

约束：

- frame length、kind、flags、stream ownership、pending request 和 queue 都必须有硬上限。
- unknown wire kind/version/flags 和损坏 payload fail closed；未来 control enum 只在 negotiated minor capability 内 additive decode。
- error envelope 必须稳定。
- protocol types 不依赖 app/runtime 实现。
- 不为固定 frame codec 引入额外 RPC/codec crate；使用 Tokio I/O 和标准 byte conversion。

### 3.4 `homie-storage`

推荐：

- `rusqlite` with `bundled`
- `serde_json`
- `time`
- `uuid` or `ulid`
- `thiserror`
- test: `tempfile`

原因：

- 本地单用户 SQLite 是当前事实源，`rusqlite` 简洁、可控、已被相近工程使用。
- `bundled` 降低 macOS/Linux/Windows SQLite 链接差异。

约束：

- `PRAGMA foreign_keys = ON`
- WAL 模式
- forward-only migration
- no ORM in V1
- schema migration 必须有集成测试

### 3.5 `homie-llm`

推荐：

- `axum`
- `tower`
- `tower-http`
- `reqwest`
- `serde`
- `serde_json`
- `rust_decimal`
- `tokio`
- `thiserror`

原因：

- `axum` 位于 Tokio/Tower 生态，适合本机 proxy 和 middleware。
- `reqwest` 是成熟 high-level HTTP client，支持 JSON、stream、rustls。
- `rust_decimal` 用于 money/cost，避免 float。

约束：

- provider raw key 只在 secret resolution 后注入 upstream request，不进入 logs/events/metrics。
- streaming response 需要保留 passthrough 能力，同时采集 safe metrics。
- usage/cost/cache hit rate/tool latency 写 SQLite。

### 3.6 `homie-agents`

推荐：

- `serde`
- `serde_json`
- `schemars`
- `regex`
- `thiserror`

原因：

- runtime descriptor、agent profile、permission profile 都需要 schema 化。
- status detection 可用 declarative rules，避免每个 agent 写硬编码逻辑。

约束：

- V1 默认真实 runtime 是 Codex。
- OpenCode、Claude Code 后续通过同一 descriptor/adapter contract 接入。
- MCP server proxy 不在 V1 实现，只保存配置与 profile 绑定。

### 3.7 `homie-context` / `homie-memory` / `homie-task`

推荐：

- `serde`
- `serde_json`
- `rusqlite` via `homie-storage`
- `thiserror`

原因：

- V1 只做结构化事实和最小边界，不需要引入向量库、搜索引擎或外部 memory framework。

约束：

- context/memory/task 不直接开自己的数据库连接策略；通过 storage repository API。
- memory 禁止写 raw request、raw response、raw secret、完整 tool args/result。

### 3.8 `homie-cli`

推荐：

- `clap` with `derive`
- `anyhow`
- `serde_json`
- `tokio`

原因：

- CLI 是诊断和 smoke path，不应自研 arg parser。

命令范围：

```text
homie doctor
homie runtime status
homie session list
homie llm proxy-status
```

## 4. Secret Envelope 选型建议

V1 已决策使用 encrypted local secret envelope，不直接依赖 macOS Keychain。

推荐在 `specs/virtual-key-credentials/README.md` 中二选一：

### 方案 A: `age` passphrase/identity envelope

优点：

- 格式成熟，生态认知度高。
- 自带版本化加密格式，避免自定义 envelope 格式太多。

风险：

- 与本地无交互桌面体验的密钥解锁流程需要单独设计。
- 若不使用用户 passphrase，需要定义本机 envelope key 来源。

### 方案 B: RustCrypto primitives 自定义 envelope

候选包：

- `chacha20poly1305`
- `argon2`
- `rand`
- `zeroize`
- `secrecy`

优点：

- 格式可完全贴合 Homie 的 secret ref 和 SQLite metadata。

风险：

- 自定义 crypto envelope 容易出错，必须明确 version header、salt、nonce、AAD、KDF params、rotation。

建议：

- V1 若没有强加密格式经验，优先 `age`。
- 若选择自定义 envelope，必须先写 `virtual-key-credentials` 组件 spec 和测试向量，再实现。

## 5. 暂不采用

| 包/方向 | 原因 |
|---------|------|
| Diesel / SeaORM | V1 schema 仍在快速演进，手写 SQL 更直接，避免 ORM 抽象成本 |
| 自研 terminal parser | VT/ANSI 细节复杂，直接用 `alacritty_terminal` |
| 自研 HTTP framework | `axum` 已满足 proxy/health/metrics |
| 自研 fuzzy matcher | `nucleo-matcher` 足够成熟 |
| 自研 CLI parser | `clap` 已成熟 |
| MCP proxy 库 | V1 不实现 MCP server proxy，后续单独评估 |
| OpenTelemetry 全套 | V1 本地应用先用 SQLite metrics + tracing，后续再考虑导出 |

## 6. V1 推荐依赖基线

```toml
[workspace.dependencies]
anyhow = \"1\"
axum = \"0.8\"
chacha20poly1305 = \"0.10\"
clap = { version = \"4\", features = [\"derive\"] }
futures-core = \"0.3\"
gpui = { git = \"https://github.com/zed-industries/zed.git\", rev = \"<pinned-rev>\" }
gpui_platform = { git = \"https://github.com/zed-industries/zed.git\", rev = \"<pinned-rev>\", features = [\"font-kit\"] }
image = { version = \"0.25\", default-features = false }
nucleo-matcher = \"0.3\"
notify = \"8\"
regex = \"1\"
reqwest = { version = \"0.12\", default-features = false, features = [\"json\", \"stream\", \"rustls-tls\"] }
rusqlite = { version = \"0.38\", features = [\"bundled\"] }
rust_decimal = { version = \"1\", features = [\"serde\"] }
schemars = \"1\"
secrecy = \"0.10\"
serde = { version = \"1\", features = [\"derive\"] }
serde_json = \"1\"
sha2 = \"0.10\"
tempfile = \"3\"
thiserror = \"2\"
time = { version = \"0.3\", features = [\"serde\", \"formatting\", \"parsing\"] }
tokio = { version = \"1\", features = [\"fs\", \"io-util\", \"macros\", \"net\", \"process\", \"rt-multi-thread\", \"signal\", \"sync\", \"time\"] }
tower = \"0.5\"
tower-http = { version = \"0.6\", features = [\"trace\", \"timeout\", \"cors\"] }
tracing = \"0.1\"
tracing-subscriber = { version = \"0.3\", features = [\"fmt\", \"json\", \"env-filter\"] }
uuid = { version = \"1\", features = [\"v7\", \"serde\"] }
zeroize = \"1\"
```

Notes:

- `gpui` revision must be pinned during workspace bootstrap.
- If encrypted envelope chooses `age`, add `age = \"0.12\"` and remove direct crypto primitives that are not used.
- If PTY uses `portable-pty`, add it in `homie-runtime` only after a short spike verifies resize, process kill, and macOS behavior.

## 7. Follow-up Spikes

| Spike | Decision |
|-------|----------|
| PTY seam | `portable-pty` vs Unix seam + future ConPTY |
| Secret envelope | `age` vs RustCrypto custom envelope |
| GPUI revision | exact pinned Zed revision and local patch policy |
| HTTP streaming | raw SSE passthrough strategy and metrics extraction boundary |
