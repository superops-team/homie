# Code Review Report: Diri MCP Runtime-backed Tool Surface

```yaml
change_id: diri-mcp-tool-surface
beads: homie-0pd
status: pass
reviewed_at: 2026-08-08
```

## 1. 审查范围

- 文件/模块：`crates/homie-cli/src/main.rs`, `crates/homie-cli/tests/mcp_stdio_runtime_cli.rs`, `specs/mcp-automation/README.md`, `docs/research/diri-parity-lock.md`。
- 变更类型：新增 runtime-backed MCP stdio tool dispatch。
- 调用链：MCP JSON-RPC line -> tool dispatch -> `HomieClient` -> runtime session APIs。

## 2. 旧问题复核

| ID/标题 | 位置 | 状态 | 依据 |
|---|---|---|---|
| RED-001 no `--data-dir` | `homie mcp-stdio` | fixed | added `McpStdioArgs` and runtime context |
| RED-002 static tools only | `mcp_stdio_response` | fixed | runtime-backed list/status/read/send/spawn tests pass |

## 3. Findings

| 严重度 | 类别 | 位置 | 证据与影响 | 处置 |
|---|---|---|---|---|
| medium | Correctness | `crates/homie-cli/src/main.rs` | Initial implementation left no-runtime helper as dead code under `clippy -D warnings`. | fixed: added unit test for no-runtime fallback. |
| low | Scope | parity lock | Runtime-backed basic tools do not complete full Diri MCP lineage. | accepted: API-004/API-005 remain `partial`. |

## 4. 对抗式复盘

- No `--data-dir`: should not create real HOME state. Covered by existing `mcp_stdio_cli` and new no-runtime unit test.
- Missing runtime args: should return safe JSON-RPC error, not panic. Covered by unsupported future tool test and invalid param branches.
- Real runtime path: test creates a real session, reads status/output, sends prompt, then spawns another session.

## 5. 修复摘要

- Added `mcp-stdio --data-dir/--session-id/--parent-session-id`.
- Added `McpRuntimeContext` and runtime-backed tool dispatch.
- Added runtime MCP integration tests.
- Updated MCP spec and parity lock evidence.

## 6. 验证结果

| 命令 | 结果 |
|------|------|
| `cargo test -p homie-cli --test mcp_stdio_runtime_cli -- --nocapture` | pass |
| `cargo test -p homie-cli --test mcp_stdio_cli -- --nocapture` | pass |
| `cargo check -p homie-cli` | pass |
| `cargo clippy -p homie-cli --all-targets -- -D warnings` | pass |

## 7. 剩余风险

- Lineage/children/wait/release/worktree/browser/test_run still require dedicated lanes.
