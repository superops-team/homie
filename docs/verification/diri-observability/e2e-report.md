# Diri observability 第一阶段 E2E Report

```yaml
change_id: diri-observability
beads: homie-wm7
report_type: e2e
status: pass_with_scope_limit
updated_at: 2026-08-07
```

## 1. Scope

本阶段是 observability foundation 纯模型实现，不接入 runtime socket、LLM proxy、storage repository、client protocol、CLI 或 UI。因此没有真实 app/runtime E2E 路径可执行。

## 2. Executed Substitute

| Gate | Command | Result |
|------|---------|--------|
| Model integration | `cargo test --manifest-path crates/homie-observability/Cargo.toml` | pass |
| Functional cases | FC-OBS-001 through FC-OBS-006 | pass |
| Security hook | `.githooks/pre-commit` after staging scoped files | pass |

## 3. Not Run

| E2E path | Status | Reason |
|----------|--------|--------|
| Runtime EventBus subscription | not_run | Runtime/client integration is out of scope |
| LLM proxy metrics sink failure E2E | not_run | LLM proxy integration is out of scope |
| Storage usage metrics persistence | not_run | Storage schema/repository integration is out of scope |
| CLI events wait/subscribe | not_run | CLI/client integration is out of scope |
| UI usage/evidence rendering | not_run | UI integration is out of scope |

## 4. Decision

Decision: pass_with_scope_limit

Reason: The first-stage contract and model are fully verified. Broader E2E paths are explicitly excluded from this change and must be covered by future lane-specific PRDs.
