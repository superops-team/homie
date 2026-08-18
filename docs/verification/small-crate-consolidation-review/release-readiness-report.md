# 评估就绪报告 — small-crate-consolidation-review

## 变更概述

评估单文件小 crate `homie-mcp` / `homie-usage` / `homie-pty` 是否应合并、保留或重组（审计 finding F11，Accidental Complexity，Suggestion）。本切片为评估性，结论为**全部保留，不合并**，无代码落地。

- change_id：`small-crate-consolidation-review`
- Beads：`homie-ubu.9`
- 类型：evaluation（评估性切片，产出决策表与证据）

## 事实基线

| crate | 行数 | 类型 | 依赖 | 消费者 | 语义 |
|-------|------|------|------|--------|------|
| `homie-mcp` | 241（`src/main.rs`） | 二进制 | `serde_json`（唯一） | 0（独立进程） | 把 `homie` CLI 暴露为 MCP stdio server |
| `homie-usage` | 162（`src/lib.rs`） | 库 | 无（零依赖） | `homie-app` + `homie-node`（2） | 共享定价表（`ModelPricing`/`match_claude` 等） |
| `homie-pty` | 430（`unix.rs` 367 + `lib.rs` 63） | 库 | `libc`（仅 unix） | `homie-engine` + `homie-remote`（2） | PTY 平台封装 |

## 决策表

| crate | 决策 | 理由 |
|-------|------|------|
| `homie-usage` | **保留** | 零依赖共享定价表 leaf crate，2 个消费者（app + node）。合并到任一方都会迫使另一方依赖一个远超其需求的更大 crate，破坏依赖方向隔离。单文件 162 行属内聚，非 accidental complexity。 |
| `homie-pty` | **保留** | 平台门控（`libc` unix-only）leaf crate，2 个消费者（engine + remote）。合并会把 unix 平台细节泄漏进 engine/remote，且两边共同依赖，保留是最小耦合。 |
| `homie-mcp` | **保留** | 独立部署产物（stdio MCP 桥进程，仅 `serde_json` 依赖），非可并入库的「库」。其存在意义是把 `homie` CLI 适配为 MCP server，属独立二进制 artifact，无合并对象。 |

## 结论

三小 crate 的「单文件」形态并非代码异味：crate 边界提供的是真实的依赖隔离（usage/pty 各 2 消费者）、平台门控（pty）与独立制品分离（mcp）价值，而非 accidental complexity。F11 判定为**无需合并**，评估完成，零代码改动。

## 验证证据

- 评估为静态分析，无代码移动，故无 `cargo test`/`cargo check` 新增运行要求；现有 workspace 在 `terminal-state-module-split`（v0.1.15）已 `cargo check --workspace` 0 警告、`cargo fmt --all` 通过。
- 依赖方向与消费者数量经 `grep -rl '<crate>' homie/crates/*/Cargo.toml` 核实。

## 已知限制与后续

- 无已知限制。
- 本切片为决策文档，无代码变更、无 tag 需要（无功能变化）。
