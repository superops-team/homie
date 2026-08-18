# Homie CLI 配置操作面：`config` / `doctor` / `fix` + `homie` skill 设计文档

## 1. 概述

### 1.1 问题/背景

`AGENTS.md` 声明 Homie 是「统一 LLM 配置入口点」。LLM 网关（`llm-gateway-virtual-keys`，
Beads `homie-f91`）落地了**本地 HTTP 代理 + 虚拟 key + agent 配置自动注入**这条链路，但它
**没有给人/agent 一个可用的操作面**：

- 网关的上游凭证（`base_url` / `api_key`）与模型映射目前没有录入入口——用户只能手工编辑
  ignored 配置文件，易错且不可发现；
- `homie doctor` 只检查 3 项（daemon socket、`claude`/`codex` 二进制、state file），完全
  **不检查 LLM 网关**是否可达、上游凭证是否有效、虚拟 key 是否生效、agent 配置是否真的指向
  网关；
- 没有 `fix`：端口冲突、凭证缺失、配置漂移这类常见问题没有自动修复路径；
- 没有 `config agent` 预览：用户无法看到「实际注入到 Codex/Claude 启动 argv/env 的配置」长
  什么样，调试全靠猜；
- AI agent（Codex/Claude/OpenCode 等）没有一个 `homie` skill 来统一调用上述 CLI 命令。

本 PRD 补上这条操作面，让「人」和「AI agent」都能用一条命令查看、录入、诊断、修复 Homie 的
LLM 网关配置与注入结果。

### 1.2 目标

1. 新增 `homie config` 子命令组：`show` / `get` / `set` / `agent`，查看与录入网关配置、模型
   映射、虚拟 key 状态，并预览实际注入的 agent 配置。
2. 增强 `homie doctor`：新增 LLM 网关可达性、上游凭证、虚拟 key 生效、agent 配置指向正确四项
   检查。
3. 新增 `homie fix`：修复端口冲突、缺失凭证、配置漂移等常见问题。
4. 提供一个 `homie` skill（`SKILL.md`），让 AI agent 通过 CLI 操作 homie。

### 1.3 非目标

- 不实现 per-agent 模型映射的 UI 编辑面（只提供 CLI 录入与查看；图形 UI 属后续）。
- 不实现配额/限流/策略/审计（属 child Bead `llm-gateway-policy-quota`）。
- 不把 Claude/Codex 登录凭证接入网关上游（属 child Bead `llm-gateway-credential-login`）。
- 不改造 `homie-node` 远程节点协议与 `homie-mcp` MCP 代理。
- 不做交互式 TUI 配置向导（首版纯子命令 + 幂等参数；TUI 属后续）。

### 1.4 关键设计决策

#### 决策 A：统一网关配置为单一 JSON 文件（跨 Swift/Rust 共享）

LLM 网关 PRD1 曾将网关配置定为 `gateway.local.toml`（TOML）。但 `homie config` 是 **Swift CLI**
（`Sources/homie-cli/Homie.swift`），Swift 没有一等 TOML 解析器；而网关是 **Rust**
（`homie-gateway`，`serde_json` 已是 workspace 依赖）。二者要读写**同一个**配置文件，TOML 会
带来跨语言解析成本。

结论：将网关配置的**规范格式统一为 JSON**，文件名 `homie.local.json`，与 `.gitignore` 已忽略的
`homie.local.*` / `*.local.json` 规则一致。Rust 网关用 `serde_json` 读写，Swift CLI 用
`JSONEncoder`/`JSONDecoder` 读写。此决策**取代** PRD1 中 `gateway.local.toml` 的 TOML 选择，
是让「CLI 可录入」成立的最小必要调整（在 PRD1 实现 T2 config.rs 落地时同步采用 JSON）。

#### 决策 B：`config agent` 复用 Rust 注入逻辑，不重写

`config agent <codex|claude>` 要预览「实际注入的配置」，其权威来源是
`homie/crates/homie-engine/src/inject.rs::injection_args()`（Codex `-c` 覆盖、Claude
`ANTHROPIC_*` env）。Swift CLI **不重写**这套注入逻辑，而是调用一个 Rust 导出/CLI 入口得到
结果，保证「预览的就是真实注入的」。

MVP 落地方式（二选一，实现时取最小者）：

- **首选**：给 `homie-gateway` 二进制加一个 `inject --agent <codex|claude>` 子命令，内部复用
  `homie-engine::inject::injection_args()`，输出 JSON；Swift CLI 调用该二进制读取 stdout。
- 备选：若复用 `homie-engine` 作为库暴露纯函数代价过高，则 `config agent` 仅打印静态的
  「将要注入的 key/value 形状」（含网关地址与虚拟 key 占位），并在注释中标明与
  `injection_args()` 的一致性由单测锁定。

首版推荐**首选**，因为 `injection_args` 已是纯函数（输入 `InjectionSpec`/`inject_dir`/`cli_path`），
暴露为库或子命令成本低，且能保证单一事实来源。

#### 决策 C：`config show` 的虚拟 key 状态读网关 SQLite

虚拟 key 的持久化是网关 SQLite（PRD1 T3）。`config show` 需要展示「key 列表 + last_used」，
直接读网关 SQLite 文件（路径契约见 §4.3）。macOS 上 Swift 用系统 `SQLite3` C 库、Rust 用
`rusqlite`，二者读同一文件，避免再开一个管理 HTTP 端点。SQLite 文件为 append-only 只读查询，
CLI 只读不写（写只经网关）。

#### 决策 D：`fix` 幂等、最小侵入

`fix` 不引入通用配置迁移框架（违反「不过度设计」）。它是**有限的一组显式修复动作**，每个
动作：探测 → 若已健康则跳过（幂等）→ 否则修复并输出做了什么。动作表见 §4.6。

## 2. 用户场景

### 场景 1：人查看当前 LLM 配置

**Given** 用户已配置过网关。  
**When** 运行 `homie config show`。  
**Then** 显示网关监听地址、上游 base_url、api_key（脱敏）、模型映射、虚拟 key 列表与
last_used，全部来自本地 ignored 文件/SQLite，不回显明文 key。

### 场景 2：人录入上游凭证与模型映射

**Given** 用户有 OpenAI 兼容服务 base_url 与 api_key。  
**When** 运行 `homie config set --base-url ... --api-key ... --model-codex gpt-5.2-codex`。  
**Then** 写入 `homie.local.json`（ignored），凭证不进入 git；再次 `config show` 可见。

### 场景 3：人预览 agent 实际注入配置

**Given** 网关已配置。  
**When** 运行 `homie config agent codex`。  
**Then** 打印 Codex 实际会收到的 `-c model_provider=...` 等 argv 与虚拟 key env 指向，与
`injection_args()` 输出一致；`config agent claude` 同理打印 `ANTHROPIC_BASE_URL` /
`ANTHROPIC_AUTH_TOKEN`。

### 场景 4：人诊断 LLM 配置健康

**Given** 环境可能存在端口冲突或凭证缺失。  
**When** 运行 `homie doctor`。  
**Then** 除原有 3 项外，还检查网关可达、上游凭证有效、虚拟 key 生效、agent 配置指向正确，
每项 `✓`/`✗`，失败项给出原因。

### 场景 5：人自动修复常见问题

**Given** `homie doctor` 报告「网关端口被占用」或「上游凭证缺失」。  
**When** 运行 `homie fix`。  
**Then** 修复可自动处理的项（换端口、补占位/提示录入），跳过需人工的项，输出每个动作的
结果。

### 场景 6：AI agent 通过 skill 操作 homie

**Given** 一个 AI agent（如 Codex）需要查看/调整 Homie 配置。  
**When** 它读取并执行 `homie` skill。  
**Then** 按 skill 指引调用 `homie config show` / `config set` / `doctor` / `fix`，完成诊断或
录入，且不会泄露真实 key。

## 3. 功能需求

### FR-1: `homie config show` / `config get`

- `show`：汇总展示网关监听、上游 base_url、api_key 脱敏、模型映射、虚拟 key 列表（含
  last_used）。
- `get <key-path>`：读取单个字段，如 `homie config get upstream.base_url`，输出裸值。
- 输出一律**不打印真实 api_key / master key / 虚拟 key 明文**，统一脱敏（`sk-***` 后 4 位）。

### FR-2: `homie config set`

- 可设置 `base_url`、`api_key`、`listen`、`master_key`、模型映射（`codex` / `claude`）。
- 写入 `homie.local.json`（ignored），原子写、0600 权限。
- `--api-key` / `--master-key` 支持从环境变量或 stdin 读取，避免出现在 shell history。

### FR-3: `homie config agent <codex|claude>`

- 调用 Rust `injection_args()` 等价入口，输出 Codex `-c` argv 或 Claude `ANTHROPIC_*` env 的
  **真实注入结果**（网关地址 + 虚拟 key 引用）。
- 输出 JSON（可被 skill/脚本消费）与人类可读两种形式（`--json` 切换）。

### FR-4: 增强 `homie doctor`

- 保留原有 3 项检查，新增：
  1. 网关可达性（绑 127.0.0.1 端口，探测 `/v1/responses` 或专用 health）；
  2. 上游凭证有效（base_url/api_key 已录入且非空，可选做一次上游连通探测）；
  3. 虚拟 key 生效（SQLite 中至少一个 key 存在且未撤销）；
  4. agent 配置指向正确（`config agent` 结果指向当前网关地址而非真实 provider）。
- 退出码：任一检查失败返回非零。

### FR-5: `homie fix`

- 自动修复以下项（幂等）：
  1. 端口冲突：`config set --listen` 换可用端口并提示；
  2. 缺失上游凭证：写入提示/占位并引导 `config set`（不静默写入真实 key）；
  3. 配置漂移：`homie.local.json` 与网关实际运行状态不一致时，重建/对齐配置；
  4. 网关未运行：提示启动方式（不自动拉起守护进程，避免生命周期混淆）。
- 每个动作先探测、已健康则跳过；输出「做了什么 / 为什么跳过」。

### FR-6: `homie` skill（`SKILL.md`）

- 位于 `homie/.agents/skills/homie/SKILL.md`（项目 skill root `r3`）。
- 说明如何用 `homie config show/get/set/agent`、`doctor`、`fix` 查看/录入/诊断/修复 LLM 配置。
- 明确「真实 key 不回显、不进 git、不进 agent 可见配置」的红线，指导 agent 使用脱敏输出与
  stdin 录入。

### FR-7: 安全边界

- 真实 `api_key` / `master_key` / 虚拟 key 明文只存 ignored 文件与本地 SQLite。
- CLI 输出脱敏；`config set` 的 key 参数不进 shell history（支持 env/stdin）。
- `homie.local.json` 权限 0600，路径 `.gitignore` 已忽略。

## 4. 实现方案

### 4.1 配置规范（单一 JSON）

```jsonc
// homie.local.json（ignored，0600）
{
  "gateway": {
    "listen": "127.0.0.1:7338",
    "masterKey": null            // 可空
  },
  "upstream": {
    "baseUrl": "https://api.openai.com/v1",
    "apiKey": "sk-..."           // 仅本地
  },
  "models": {
    "codex": "gpt-5.2-codex",
    "claude": "claude-sonnet-4-5"
  }
}
```

### 4.2 模块边界

```text
Sources/homie-cli/
├── Homie.swift                 # 注册 Config/Doctor(增强)/Fix 子命令
├── ConfigCommand.swift         # config show/get/set/agent
├── FixCommand.swift            # fix
├── DoctorCommand.swift         # doctor（增强）
└── HomieConfigStore.swift      # homie.local.json 读写 + 脱敏 + SQLite 只读

homie/crates/homie-gateway/
└── src/inject.rs               # inject --agent 子命令（复用 homie-engine::inject）
```

### 4.3 文件与路径契约

| 内容 | 路径 | 读写方 |
|------|------|--------|
| 网关/模型配置 | `~/.config/homie/homie.local.json`（`HOMIE_CONFIG` 覆盖） | Rust 网关 + Swift CLI 均读写 |
| 虚拟 key + 用量 | 网关 SQLite（`~/.config/homie/gateway.sqlite3` 或 node 数据目录下） | 网关写，CLI 只读 |
| 注入逻辑 | `homie-engine::inject::injection_args()` | Rust（单一事实来源） |

> 注意：`homie.local.json` 默认路径与 PRD1 的 `gateway.local.toml` 不同，需在 PRD1 T2 实现时
> 统一到本文件的 JSON 规范（见决策 A）。

### 4.4 `config agent` 与注入一致性

`config agent` 的结果必须与 spawn 时 `injection_args()` 一致。通过在 `homie-gateway` 暴露
`inject --agent <codex|claude> --json`，内部调用 `homie-engine::inject::injection_args()`（读
`InjectionSpec` + manifest + 网关配置），Swift CLI 转发该 stdout。这样「预览」与「真实注入」
共享同一函数，杜绝漂移。单测断言二者输出形状相等。

### 4.5 脱敏规则

- `api_key` / `master_key` / 虚拟 key：显示 `sk-***` + 末 4 位（或全 `***`）。
- 任何错误信息、日志不包含完整 key、不包含敏感 prompt。

### 4.6 `fix` 动作表

| 探测 | 已健康 | 修复动作 |
|------|--------|----------|
| 网关端口被占用 | 跳过 | `config set --listen` 换可用端口，提示重启网关 |
| upstream api_key 缺失 | 跳过 | 引导 `config set --api-key`（stdin 录入），不静默填真实值 |
| 配置漂移（文件缺失/格式坏） | 跳过 | 重建最小合法 `homie.local.json`（保留可识别旧字段） |
| 网关未运行 | 跳过 | 打印启动命令提示（不自动拉起守护进程） |

## 5. 边界情况

| 场景 | 处理 |
|------|------|
| `homie.local.json` 不存在 | `config show` 输出空配置提示；`set` 首次创建 |
| `homie.local.json` 格式损坏 | `show`/`get` 报错并提示 `homie fix`；`fix` 重建 |
| 真实 key 出现在命令行参数 | 引导用 `--api-key-from-stdin` / env，避免 history |
| 网关 SQLite 不存在 | `config show` 的 key 列表显示「无（网关未初始化）」 |
| `config agent` 时网关未配置 | 输出占位（网关地址为空），不伪造真实 provider |
| 端口被占用 | `doctor` 报 `✗`，`fix` 换端口 |
| 上游不可达 | `doctor` 报 `✗`（可选探测超时脱敏），不泄露 key |

## 6. 涉及文件

- `Sources/homie-cli/Homie.swift`（注册 Config/Fix、增强 Doctor）
- `Sources/homie-cli/ConfigCommand.swift`（新增）
- `Sources/homie-cli/FixCommand.swift`（新增）
- `Sources/homie-cli/DoctorCommand.swift`（增强）
- `Sources/homie-cli/HomieConfigStore.swift`（新增：JSON 读写 + SQLite 只读 + 脱敏）
- `homie/crates/homie-gateway/src/inject.rs`（`inject --agent` 子命令，复用 homie-engine）
- `homie/crates/homie-engine/src/inject.rs`（若需暴露纯函数给 gateway 复用）
- `homie/.agents/skills/homie/SKILL.md`（新增 skill）
- `specs/homie-cli-config-ops.md`（组件合同）
- `.gitignore`（确认 `homie.local.*` / `*.local.json` 已忽略）

## 7. 验证计划

### 7.1 单元测试（Rust）

- `injection_args()` 输出在 gateway `inject --agent` 与 engine 单测中一致（防漂移）。
- 配置 JSON 序列化/反序列化 round-trip、脱敏函数。

### 7.2 单元测试（Swift）

- `HomieConfigStore`：读写、原子写、0600 权限、脱敏、损坏文件处理。
- `config get` 路径解析、`config set` 参数校验（stdin/env 录入）。

### 7.3 集成测试

- `config set` 写 `homie.local.json` 后，Rust 网关能读同一文件启动。
- `config agent codex` 输出与真实 spawn 注入 argv/env 一致。
- `doctor` 全项检查：健康环境全 `✓`、构造故障环境触发对应 `✗`。
- `fix`：构造端口冲突/凭证缺失/配置漂移，断言修复动作与幂等性。

### 7.4 门禁

- `cargo check --workspace`
- `cargo fmt --all --check`
- `cargo test -p homie-gateway`
- `cargo test -p homie-engine inject`
- `swift build`（`Sources/homie-cli` 编译通过）

## 8. 验收标准

1. `homie config show` / `get` / `set` / `agent` 可用，输出脱敏。
2. `homie config agent codex|claude` 与 `injection_args()` 真实注入一致。
3. `homie doctor` 新增 4 项 LLM 网关检查，失败返回非零。
4. `homie fix` 可修复端口冲突/凭证缺失/配置漂移且幂等。
5. `homie` skill 存在且可指导 AI agent 操作，不泄露真实 key。
6. `homie.local.json` 被 git 忽略、权限 0600、真实 key 不进 git/log/agent 可见配置。
7. OpenSpec alignment 对齐本 PRD，Beads `homie-ys0` 关闭。

## 9. Beads 追踪

- Beads: `homie-ys0`
- change_id: `homie-cli-config-ops`
- 类型: feature
- 优先级: P0
- 依赖: `llm-gateway-virtual-keys`（至少 T1 网关 + T8 注入先落地，`config agent` 与
  `doctor` 网关检查才可验证）
