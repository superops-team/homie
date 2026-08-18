# OpenSpec Plan — homie-cli-config-ops

## 1. 变更概述

为 Homie 落地 LLM 配置的 CLI 操作面：新增 `homie config`（`show`/`get`/`set`/`agent`）、增强
`homie doctor`（LLM 网关四项检查）、新增 `homie fix`（幂等修复），并提供 `homie` skill。核心
是让「人」和「AI agent」用一条命令查看、录入、诊断、修复网关配置与注入结果。

关键调整：将网关配置规范从 PRD1 的 `gateway.local.toml` 统一为 `homie.local.json`（JSON），
使 Swift CLI 与 Rust 网关共享同一文件。`config agent` 复用
`homie-engine::inject::injection_args()`，保证「预览 = 真实注入」。

## 2. 模块划分与依赖

```text
Sources/homie-cli/
├── Homie.swift                # 注册 Config / Fix，增强 Doctor
├── ConfigCommand.swift        # config show/get/set/agent
├── FixCommand.swift           # fix
├── DoctorCommand.swift        # doctor（增强）
└── HomieConfigStore.swift     # homie.local.json 读写 + SQLite 只读 + 脱敏

homie/crates/homie-gateway/
└── src/inject.rs              # inject --agent 子命令（复用 homie-engine::inject）

homie/.agents/skills/homie/SKILL.md
```

依赖：`llm-gateway-virtual-keys`（T1 网关 + T8 注入先落地）。无新增第三方 Swift 包（用系统
`SQLite3` C 库只读查询）；Rust 侧无新增依赖（复用 `serde_json`/`rusqlite`）。

## 3. 层级关系

| 层 | 产物 |
|----|------|
| 需求 | `prd-spec/features/homie-cli-config-ops/2026-08-18-homie-cli-config-ops-design.md` |
| 规范 | `specs/homie-cli-config-ops.md`（配置格式、命令、注入一致性、安全） |
| 执行 | 本 OpenSpec + `Sources/homie-cli/*` + `homie-gateway/src/inject.rs` + skill |
| 证据 | `docs/verification/homie-cli-config-ops/` |

## 4. 与既有权威的关系

- `homie-engine/src/inject.rs::injection_args()` 保持注入逻辑唯一事实来源；`config agent` 只
  委托，不复制。
- 不改变 PTY/holder 生命周期权威（`specs/engine-session-runtime.md`）。
- 不改 `homie-node` 远程节点与 `homie-mcp` MCP 代理。
- 网关 SQLite 由 `homie-gateway` 独占写，CLI 只读（`specs/llm-gateway.md` §7 用量合同不变）。

## 5. 安全边界（Tier 3 域）

配置录入与回显涉及 credential custody（真实 api_key/master key/虚拟 key），属 security-sensitive
Tier 3 域。要求：

- 脱敏函数、JSON 读写、`config set` 的 stdin/env 录入路径新增/改动行逐行覆盖。
- 手工 mutation：对脱敏、原子写、损坏文件重建 3–5 个真实 bug，套件必须击杀并恢复。

## 6. 后续 child Bead（本变更只声明，不实现）

- per-agent 模型映射图形 UI（`llm-gateway-model-routing`）。
- 配额/限流/策略/审计（`llm-gateway-policy-quota`）。
- Claude/Codex 登录凭证接入网关上游（`llm-gateway-credential-login`）。
- 交互式 TUI 配置向导（后续独立变更）。
