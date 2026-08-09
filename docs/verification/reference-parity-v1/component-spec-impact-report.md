# Homie Reference Parity V1 Component Spec Impact Report

```yaml
change_id: reference-parity-v1
report_type: component-spec-impact
status: pass
beads: homie-h7n
source_prd: prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md
reviewed_at: 2026-08-05
```

## 1. Decision

Decision: pass for design phase, pass for component-spec creation; implementation still requires child Beads and task-specific SDD/TDD gates.

Reason:

- The PRD correctly identifies every long-lived component affected by Reference parity.
- Implementation must not start from the PRD alone because many affected components define protocol, credential, process, storage, update, remote, and MCP contracts.
- The OpenSpec tasks assign each component update to an execution owner.

## 2. Required Component Spec Updates

| Component spec | Required state before implementation | Owning OpenSpec task |
|----------------|--------------------------------------|----------------------|
| `specs/desktop-shell/README.md` | Define GPUI shell, sidebar, terminal pane, floating surfaces, settings, inspector, menu bar, notifications, keyboard map, UI fidelity gates | T-007, T-008, T-009 |
| `specs/runtime-supervisor/README.md` | Define PTY/process ownership, holder-equivalent, output log, headless screen, session lifecycle, resource governor, recovery | T-004 |
| `specs/agent-adapter-contract/README.md` | Define manifest schema, Reference agent catalog parity, status authority, approval/deny, resume, hook/notify contracts | T-003 |
| `specs/llm-proxy/README.md` | Define OpenAI-compatible proxy, streaming, usage metrics, cost snapshots, safe errors | T-012 |
| `specs/virtual-key-credentials/README.md` | Define real provider key custody, virtual key scope, revocation, audit, remote/node credential policy | T-012, T-014 |
| `specs/session-context-store/README.md` | Define session context, lineage, history, artifacts, summaries, task/memory references | T-015 |
| `specs/storage-indexing/README.md` | Extend schema, migration, preferences, output index, usage ledger, host/node config boundaries | T-005 |
| `specs/observability/README.md` | Define safe logging, metrics, events, trace, evidence helpers, redaction | T-011, T-012, T-017 |
| `specs/task-controller/README.md` | Define task model, agent claim/update/block/return, Beads boundary | T-015 |
| `specs/memory-controller/README.md` | Define durable memory candidate writes, source attribution, redaction, retrieval boundary | T-015 |
| `specs/intent-orchestrator/README.md` | Define New Agent, command palette, MCP spawn, task routing, permission-aware orchestration | T-015 |
| `specs/packaging-updater/README.md` | Create bundle, signing, notarization, updater trust, helper swap, rollback, perf gate contract | T-016 |
| `specs/remote-node-handoff/README.md` | Create host catalog, node, accounts, remote spawn, sync prefs, locate repo, move/fork/handoff contract | T-014 |
| `specs/mcp-automation/README.md` | Create CLI, hook/notify, MCP tools, browser/test_run, lineage and permission contract | T-013 |

## 3. Blocking Rules

| Rule | Applies to | Enforcement |
|------|------------|-------------|
| No code before component contract | Any task touching public protocol, runtime lifecycle, credentials, storage, remote/node, MCP or updater | Task cannot move past RED until component spec exists |
| No raw Reference compatibility fallback | Storage/protocol/runtime | Obsolete compatibility is forbidden unless user explicitly requests migration |
| No credential custody regression | Agent adapter, remote/node, MCP, proxy | Security tests must prove real provider key is absent from env/log/event/artifact |
| No UI-only state ownership | Desktop shell | UI state persists through storage/client APIs, not direct runtime mutation |
| No release evidence substitution | Packaging/updater/perf | Development-binary checks cannot replace packaged artifact gates |

## 4. Residual Risks

| Risk | Mitigation |
|------|------------|
| Many component specs are not yet present as files | T-001 must create or update them before implementation branches start |
| Reference's node-local provider model conflicts with Homie proxy custody | `virtual-key-credentials` and `remote-node-handoff` specs must resolve policy first |
| UI fidelity scope is large | Deterministic preview fixtures and screenshot gates are mandatory |
| Updater requires external signing assets | Release gate remains blocked until real notarized artifacts are tested |

