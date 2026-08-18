# Spec Review Report — homie-cli-config-ops

## 1. 范围

本报告记录 `homie-cli-config-ops`（Beads `homie-ys0`）实现前的 spec 与可行性评审结论，覆盖
配置格式统一、命令合同、注入一致性、跨语言边界与安全分级。

## 2. 技术选型评审（可行性）

| 决策 | 结论 | 依据 |
|------|------|------|
| 配置格式 JSON（`homie.local.json`）而非 TOML | **采纳** | Swift CLI 无一等 TOML 解析器；Rust 已有 `serde_json`。跨语言共享同一文件时 JSON 最小成本 |
| `config agent` 委托 Rust 注入而非重写 | **采纳** | `injection_args()` 已是纯函数，暴露为子命令成本低，且消除「预览≠真实注入」漂移 |
| 虚拟 key 状态读网关 SQLite（只读） | **采纳** | 避免另开管理端点；macOS 系统 `SQLite3` C 库可读，CLI 只读不写 |
| 无新增第三方 Swift 包 | **采纳** | 用系统 `SQLite3` + `Foundation` JSON，符合「不新增包」 |
| `fix` 有限动作表，非通用迁移框架 | **采纳** | 符合 AGENTS.md「不过度设计」 |

## 3. 依赖评估

| 依赖 | 状态 | 理由 |
|------|------|------|
| `serde_json` | workspace 已有 | Rust 网关读 JSON 配置 |
| `rusqlite` | workspace 已有 | 网关写、CLI 侧 Rust 可读 |
| SQLite3（系统 C 库） | macOS 内建 | Swift 只读查询虚拟 key |
| `Foundation` JSONEncoder/Decoder | 系统框架 | Swift 读写 `homie.local.json` |
| 无新增包 | — | 符合依赖添加政策 |

## 4. 组件合同评审

`specs/homie-cli-config-ops.md` 定义了配置文件（§3）、命令（§4）、注入一致性（§5）、虚拟 key
只读（§6）、安全/恢复（§7）、skill（§8）。评审结论：合同与 PRD FR 一一对应，无缺口。

## 5. 跨语言一致性风险与对策

风险：Swift CLI 与 Rust 网关对 `homie.local.json` 的 schema 认知漂移。对策：schema 在
`specs/homie-cli-config-ops.md` §3 定死；Swift/Rust 各写 round-trip 单测，且 `config set` 写后
Rust 网关读同文件启动作为集成测试（FC-1）。

风险：`config agent` 预览与 spawn 注入漂移。对策：单一事实来源 `injection_args()`，单测断言
输出形状相等（FC-4）。

## 6. 安全 Tier 分级

配置录入/回显涉及 credential custody（真实 api_key/master key/虚拟 key），属 Tier 3
security-sensitive 域。`docs/development/quality-gates.md` §4.2 要求 coverage + mutation gate：

- 脱敏函数、JSON 原子写、stdin/env 录入路径逐行覆盖。
- 手工 mutation：对脱敏、原子写、损坏文件重建注入 3–5 个真实 bug，套件必须击杀并恢复。

## 7. 与既有权威的一致性

- `homie-engine::inject::injection_args()` 保持注入唯一事实来源，`config agent` 只委托不复制。
- 不改 PTY/holder 生命周期权威（`specs/engine-session-runtime.md`）。
- 网关 SQLite 由 `homie-gateway` 独占写，CLI 只读（`specs/llm-gateway.md` §7 用量合同不变）。
- 决策 A（JSON 统一）仅调整 `specs/llm-gateway.md` §6 的配置文件名措辞（`gateway.local.toml`
  → `homie.local.json`），不改其协议/虚拟 key/用量语义；已在 `llm-gateway-virtual-keys` T2 实现
  时同步。

## 8. 结论

方案可行，规格齐备，跨语言边界已明确，可进入实现阶段。
