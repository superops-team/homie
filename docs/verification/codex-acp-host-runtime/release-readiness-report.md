# Release Readiness Report — codex-acp-host-runtime

## 1. 变更类型

新运行时能力模块（Rust 代码实现）。change_id: `codex-acp-host-runtime`，Beads: `homie-skh`。
新增 ACP host harness（stdio JSON-RPC 2.0），属**新 runtime 能力**。

## 2. 交付物清单

| 文件 | 状态 |
|------|------|
| `prd-spec/features/codex-acp-host-runtime/2026-08-18-codex-acp-host-runtime-design.md` | 新增 |
| `homie/crates/homie-engine/src/acp/mod.rs` | 新增 |
| `homie/crates/homie-engine/src/acp/protocol.rs` | 新增 |
| `homie/crates/homie-engine/src/acp/frame.rs` | 新增 |
| `homie/crates/homie-engine/src/acp/host.rs` | 新增 |
| `homie/crates/homie-engine/src/acp/approval.rs` | 新增 |
| `homie/crates/homie-engine/src/acp/driver.rs` | 新增 |
| `homie/crates/homie-engine/src/lib.rs`（`pub mod acp;`） | 修改 |
| `homie/crates/homie-engine/tests/acp_host.rs` | 新增 |
| `homie/crates/homie-engine/Cargo.toml`（`[[test]] acp_host`） | 修改 |
| `specs/engine-session-runtime.md`（ACP/PTY 边界） | 修改 |
| `openspec/changes/codex-acp-host-runtime/{plan,tasks,alignment-report}.md` | 新增 |
| `docs/verification/codex-acp-host-runtime/{spec-review,functional-cases,functional-verification,release-readiness}.md` | 新增 |

## 3. 门禁检查

- `cargo check --workspace`：通过。
- `cargo fmt --all --check`：通过。
- `cargo test -p homie-engine`：296 passed / 0 failed / 3 ignored。
- `cargo test -p homie-engine --test acp_host`：E2E 通过。
- spec review：16 维度 PASS，无 P0-P3 问题。
- 功能验证：7/7 PASS。
- OpenSpec alignment：FR-1..FR-6 + 验收 §8 全覆盖，零漏项。

## 4. 版本标签

按 `AGENTS.md` 版本规则，本变更为"new runtime capabilities"（新增 ACP host harness 模块）
→ **minor** 递增。下一 tag：`v0.1.17`。

## 5. 后续 child Bead（不属本变更）

- session-driver 集成（把 `AcpDriver` 接入 `session.spawn`）。
- `fs/read_text_file` / `fs/update_text_file` 文件代理。
- GPUI chat canvas / composer / transcript（`chat-surface-gpui`）。
- 真实 provider 接入与模型发现（`available_commands_update`）。

## 6. 结论

实现完整、证据齐备，可提交、打 tag `v0.1.17` 并关闭 Beads `homie-skh`。
