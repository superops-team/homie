# Diri Agent Detection Functional Cases

```yaml
change_id: diri-agent-detection
beads: homie-v4b
status: designed_before_implementation
source_prd: prd-spec/features/diri-agent-detection/2026-08-07-diri-agent-detection-design.md
```

## 1. 验证原则

- 每个 P0 需求至少有一个可执行 Case 覆盖。
- 本阶段只验证 `homie-agents` 与 `assets/agent-descriptors` 的真实代码路径，不使用 runtime mock 替代。
- Readiness 使用 resolver 注入来避免依赖本机真实 PATH；这验证的是 `homie-agents` readiness 合同，不验证 runtime PATH 解析。
- 所有命令输出和结果写入 `docs/verification/diri-agent-detection/release-readiness-report.md`。

## 2. Functional Case 清单

| Case ID | 覆盖需求 | 前置环境 | 执行命令 | 预期结果 | 证据路径 | 失败处理 |
|---|---|---|---|---|---|---|
| FC-DA-001 | FR-1, FR-2, FR-3 | Rust toolchain 可用 | `cargo test -p homie-agents --test manifest_catalog bundled_catalog_projects_diri_manifest_fields -- --nocapture` | 19 个 manifest 均加载；Claude/Codex/Cursor/Gemini/Amp descriptor 字段与 Diri source 一致 | `release-readiness-report.md#fc-da-001` | 回到 manifest loader 或 asset 数据修复 |
| FC-DA-002 | FR-2, FR-3 | Rust toolchain 可用 | `cargo test -p homie-agents --test manifest_catalog catalog_resolves_aliases_and_falls_back_for_unknown_ids -- --nocapture` | alias/id/shortLabel 可解析；unknown id fallback 为 process/no binary/no quick action | `release-readiness-report.md#fc-da-002` | 回到 catalog API 修复 |
| FC-DA-003 | FR-4 | Rust toolchain 可用 | `cargo test -p homie-agents --test manifest_catalog readiness_projects_launchable_agents_only -- --nocapture` | readiness 只包含有 binary 的 launchable agent；`shell`/`generic` 不探测；缺失 binary 不影响其他 agent | `release-readiness-report.md#fc-da-003` | 回到 readiness projection 修复 |
| FC-DA-004 | FR-5, FR-6 | Rust toolchain 可用 | `cargo test -p homie-agents --test golden_screens -- --nocapture` | Claude/Codex/Cursor/Gemini golden screen 的 state/rule/options/excerpt 与 Diri tests 对齐 | `release-readiness-report.md#fc-da-004` | 回到 manifest rules、region extraction 或 predicate engine 修复 |
| FC-DA-005 | FR-7, FR-8, FR-9 | Rust toolchain 可用 | `cargo test -p homie-agents --test hook_parser -- --nocapture` | Claude/Codex stable events、needs-input、subagent isolation、nested/URL/header secret redaction 全部通过 | `release-readiness-report.md#fc-da-005` | 回到 hook parser/redaction 修复 |
| FC-DA-006 | FR-5, FR-10 | Rust toolchain 可用 | `cargo test -p homie-agents --test manifest_catalog every_bundled_manifest_decodes_strictly -- --nocapture` | bundled manifest 严格解码；full manifest 必须有 rules；process-only 可为空 | `release-readiness-report.md#fc-da-006` | 回到 asset/parser 修复 |
| FC-DA-007 | AC-7 | Git/Rust toolchain 可用 | `cargo fmt --all -- --check && cargo check -p homie-agents && cargo clippy -p homie-agents --all-targets -- -D warnings && cargo test -p homie-agents && git diff --check` | 相关 crate 格式、编译、lint、测试、diff whitespace 通过 | `release-readiness-report.md#quality-gates` | 修复代码或记录非本 change 阻塞 |
| FC-DA-008 | Security baseline | `.githooks/pre-commit` 可执行 | `.githooks/pre-commit` | hook 未发现 secret 或安全违规；若因仓库全局状态失败，记录 stdout/stderr 和原因 | `release-readiness-report.md#security-gate` | 修复本 change 或标明非本 change 阻塞 |

## 3. 覆盖矩阵

| PRD 需求 | Functional Cases |
|---|---|
| FR-1 catalog loader 读取 combined manifest | FC-DA-001, FC-DA-006 |
| FR-2 19-agent id/alias catalog | FC-DA-001, FC-DA-002 |
| FR-3 descriptor 表达 Diri 能力字段 | FC-DA-001, FC-DA-002 |
| FR-4 readiness projection | FC-DA-003 |
| FR-5 full/process-only manifest 规则 | FC-DA-004, FC-DA-006 |
| FR-6 golden screen parity | FC-DA-004 |
| FR-7 stable hook/notify events | FC-DA-005 |
| FR-8 hook/screen/notify redaction | FC-DA-005, FC-DA-008 |
| FR-9 unknown hook fail-open | FC-DA-005 |
| FR-10 spec/test mapping | FC-DA-007, release-readiness-report |

## 4. 执行顺序

1. 先跑 targeted RED tests，确认新增测试在实现前能暴露缺口。
2. 完成实现后逐条执行 FC-DA-001 到 FC-DA-006。
3. 执行 FC-DA-007 quality gates。
4. 执行 FC-DA-008 security hook。
5. 将实际命令、退出码、摘要和未运行原因写入 release readiness report。
