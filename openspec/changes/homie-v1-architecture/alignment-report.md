# Homie V1 Architecture OpenSpec Alignment Report

```yaml
change_id: homie-v1-architecture
report_type: openspec-alignment
status: pass
beads: homie-9c9
source_prd: prd-spec/features/homie-v1-architecture/2026-08-05-homie-v1-architecture-design.md
openspec_plan: openspec/changes/homie-v1-architecture/plan.md
openspec_tasks: openspec/changes/homie-v1-architecture/tasks.md
```

## 1. Requirement To Task Mapping

| PRD requirement | Priority | OpenSpec task | Component spec | Verification | Status |
|-----------------|----------|---------------|----------------|--------------|--------|
| FR-1 Rust workspace 与 crate 边界 | P0 | T-001 | `specs/storage-indexing/README.md`, component-specific specs | document review | pass |
| FR-2 双进程运行模型 | P0 | T-001 | `specs/runtime-supervisor/README.md` | document review | pass |
| FR-3 控制协议与事件模型 | P0 | T-001 | `specs/runtime-supervisor/README.md` | document review | pass |
| FR-4 Agent adapter contract | P0 | T-001 | `specs/agent-adapter-contract/README.md` | document review | pass |
| FR-4A Agent profile 配置中心 | P0 | T-001 | `specs/agent-adapter-contract/README.md`, `specs/llm-proxy/README.md`, `specs/storage-indexing/README.md` | document review | pass |
| FR-5 PTY/session runtime | P0 | T-001 | `specs/runtime-supervisor/README.md` | document review | pass |
| FR-6 Terminal grid 与 GPUI 渲染 | P1 | T-001 | `specs/desktop-shell/README.md` | document review | pass |
| FR-7 LLM provider config、virtual key 与 proxy | P0 | T-001 | `specs/llm-proxy/README.md`, `specs/virtual-key-credentials/README.md` | document review | pass |
| FR-7A LLM 统一流量指标、token 成本、缓存命中率与工具调用耗时 | P0 | T-001 | `specs/llm-proxy/README.md`, `specs/storage-indexing/README.md`, `specs/observability/README.md` | document review | pass |
| FR-8 Context store | P0 | T-001 | `specs/session-context-store/README.md` | document review | pass |
| FR-9 Memory、Task、Orchestrator 骨架 | P1 | T-001 | `specs/memory-controller/README.md`, `specs/task-controller/README.md`, `specs/intent-orchestrator/README.md` | document review | pass |
| FR-10 Storage | P0 | T-001 | `specs/storage-indexing/README.md` | document review | pass |
| FR-11 Desktop V1 UI | P0 | T-001 | `specs/desktop-shell/README.md` | document review | pass |
| FR-12 CLI 与诊断 | P1 | T-001 | future CLI component spec | document review | pass |

## 2. Component Spec Impact

| Component spec | Impact | Evidence | Status |
|----------------|--------|----------|--------|
| `specs/desktop-shell/README.md` | yes | UI shell, GPUI, terminal pane, session list | pending follow-up |
| `specs/runtime-supervisor/README.md` | yes | runtime process, session, PTY, socket, event model | pending follow-up |
| `specs/agent-adapter-contract/README.md` | yes | Codex default runtime, runtime descriptor, agent profile, skills/MCP config bindings, env scrub, status authority | pending follow-up |
| `specs/llm-proxy/README.md` | yes | OpenAI-compatible proxy, token usage, cache hit rate, estimated cost, request latency, and tool metrics | pending follow-up |
| `specs/virtual-key-credentials/README.md` | yes | encrypted local secret envelope, provider key custody, and virtual key lifecycle | pending follow-up |
| `specs/session-context-store/README.md` | yes | context events and session context | pending follow-up |
| `specs/storage-indexing/README.md` | yes | SQLite schema, migrations, WAL, foreign keys, uniqueness constraints, pricing snapshots, output-log indexing | pending follow-up |
| `specs/observability/README.md` | yes | LLM traffic metrics, token/cache/cost aggregation, request latency, and tool-call latency summaries | pending follow-up |
| `specs/memory-controller/README.md` | yes | V1 memory boundary | pending follow-up |
| `specs/task-controller/README.md` | yes | task model and Beads boundary | pending follow-up |
| `specs/intent-orchestrator/README.md` | yes | V1 intent routing | pending follow-up |

## 3. Beads Alignment

| Bead | Title | Status | Spec ID | Expected state |
|------|-------|--------|---------|----------------|
| `homie-9c9` | Draft Homie v1 architecture spec | open | `prd-spec/features/homie-v1-architecture/2026-08-05-homie-v1-architecture-design.md` | stay open until review or follow-up split |

## 4. Coverage Checks

| Check | Result | Evidence |
|-------|--------|----------|
| Every PRD FR has at least one task | pass | `openspec/changes/homie-v1-architecture/tasks.md` |
| Every affected component spec is identified | pass | PRD section 5 and this report |
| Reference project terms are absent from Homie spec | pass | `rg` check |
| Markdown hygiene passes | pass | `git diff --check` |
| No runtime code or secrets introduced | pass | PRD/OpenSpec docs only |
| Mature reusable package research exists | pass | `docs/research/rust-package-selection.md` |
| Swift + Rust project layout exists | pass | `docs/architecture/project-layout.md` |
| Development standards exist | pass | `docs/development/standards.md` |
| Quality gates exist | pass | `docs/development/quality-gates.md` |

## 5. Risks And Follow-Ups

| Risk | Source | Mitigation | Follow-up bead |
|------|--------|------------|----------------|
| V1 scope can become too broad if all crates are implemented in one change | PRD FR-1 through FR-12 | Split implementation into separate Beads and OpenSpec changes | pending |
| Component specs do not exist yet | PRD section 5 | Create P0 component specs before code implementation | pending |
| Cross-platform runtime is non-trivial | PTY and storage sections | Implement macOS/Unix first with named Windows seams | pending |
| Profile edits could accidentally mutate running sessions | PRD FR-4A/FR-5 | PRD now requires session-time `EffectiveAgentConfig` freezing | component spec follow-up |
| Historical cost could drift after price updates | PRD FR-7A/FR-10 | PRD now requires pricing snapshots and currency on usage records | component spec follow-up |
| MCP server proxy could expand V1 scope | PRD FR-4A/FR-7A | PRD now defers MCP server proxy execution; V1 only stores config/profile binding | follow-up MCP proxy bead |

## 6. Gate Decision

Decision: pass

Reason:

- The first-version architecture PRD/spec is self-contained and mapped to follow-up component specs.
- This change is design-only; implementation requires subsequent component specs and OpenSpec tasks.
