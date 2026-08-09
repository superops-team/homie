# OpenSpec Plan: Diri Agent Detection

```yaml
change_id: diri-agent-detection
beads: homie-v4b
source_prd: prd-spec/features/diri-agent-detection/2026-08-07-diri-agent-detection-design.md
functional_cases: docs/verification/diri-agent-detection/functional-cases.md
status: ready_for_tasks
```

## 1. 目标

将 Diri M16 agent catalog/detection 第一阶段落到 Homie 的 agent adapter contract：

- `assets/agent-descriptors` 成为 combined manifest 单一事实源。
- `crates/homie-agents` 提供 catalog、readiness、manifest detection、hook redaction 的纯函数合同。
- `specs/agent-adapter-contract/README.md` 明确 Diri parity mapping、failure modes 和 verification gates。

## 2. 模块边界

| 模块 | 本次职责 | 不做 |
|------|----------|------|
| `specs/agent-adapter-contract` | 长期合同：catalog/readiness/golden/redaction/failure mode | runtime supervisor 细节 |
| `assets/agent-descriptors` | 19-agent combined manifest | 用户 override/hot reload |
| `crates/homie-agents` | manifest decode、catalog projection、readiness projection、golden detection、redaction | process spawn、PTY、storage、UI |
| `docs/verification/diri-agent-detection` | spec review、functional cases、reports | 修改其他 lane evidence |

## 3. 数据流

```text
assets/agent-descriptors/<id>.json
  ├─ top-level detection manifest
  │    └─ ManifestEngine -> ScreenSnapshot -> ScreenObservation
  └─ agent descriptor block
       └─ AgentCatalog -> AgentManifest/AgentReadinessResult

Claude/Codex hook payload
  -> parse_claude_hook / parse_codex_notify
  -> ParsedHook/ParsedNotify + NeedsInputDetail + safe_summary
  -> StatusReducer signal input in later runtime lane
```

## 4. 依赖与阻塞

- 已确认 Beads: `homie-v4b`。
- 已读 Diri source/test baseline: `diri/Sources/DirijorCore/Resources/manifests`、`diri/Tests/DirijorDetectionTests`、`diri/Sources/DirijorDaemonKit/AgentReadiness.swift`、`HookParsing.swift`。
- 不依赖 storage/observability implementation；只在 spec 中引用后续 gate。

## 5. 验收 Gate

| Gate | 命令 | Functional Case |
|------|------|-----------------|
| Catalog | `cargo test -p homie-agents --test manifest_catalog` | FC-DA-001, FC-DA-002, FC-DA-003, FC-DA-006 |
| Golden | `cargo test -p homie-agents --test golden_screens` | FC-DA-004 |
| Hooks | `cargo test -p homie-agents --test hook_parser` | FC-DA-005 |
| Quality | `cargo fmt --all -- --check`, `cargo check -p homie-agents`, `cargo clippy -p homie-agents --all-targets -- -D warnings`, `cargo test -p homie-agents`, `git diff --check` | FC-DA-007 |
| Security | `.githooks/pre-commit` | FC-DA-008 |

## 6. 风险控制

- 如果 golden tests 暴露 manifest parser 不支持 Diri rule 字段，优先补 parser，不降低 fixture 断言。
- 如果 `.githooks/pre-commit` 受全仓未跟踪/外部状态影响失败，release report 记录实际原因；本 change 仍必须通过 focused Rust gates。
- 不写 `.beads` 或其他 lane 文件，避免越过用户写入范围。
