# Homie V1 Architecture OpenSpec Tasks

> Change ID: `homie-v1-architecture`
> Source PRD: `prd-spec/features/homie-v1-architecture/2026-08-05-homie-v1-architecture-design.md`
> Beads: `homie-9c9`

## Tasks

### T-001: Draft V1 architecture PRD/spec

| Field | Value |
|-------|-------|
| Status | green |
| Source requirement | FR-1 through FR-12 |
| Beads | `homie-9c9` |
| Files | `prd-spec/features/homie-v1-architecture/2026-08-05-homie-v1-architecture-design.md` |

Objective:

- Write Homie's first-version architecture spec with process model, crate boundaries, protocol, runtime, agent adapter, agent profile registry, SQLite storage, LLM proxy, token/cache/cost/tool metrics, context, memory, task, UI, and CLI boundaries.

Acceptance:

- Spec is self-contained.
- Spec uses Homie terminology.
- Spec does not mention the reference project's name or internal names.
- Spec states that SQLite is the V1 local storage source of truth.
- Spec defines how agent profiles bind runtime, LLM profile, skills, MCP servers, permission profile, and workspace scope.
- Spec defines session-time `EffectiveAgentConfig` freezing so profile edits do not mutate running sessions.
- Spec defines token usage, cache hit rate, estimated cost, request latency, and tool-call latency metrics for Homie's unified LLM traffic entrypoint.
- Spec defines pricing snapshots and currency for historical cost interpretation.
- Spec states Codex is the default V1 real runtime.
- Spec states secrets use encrypted local secret envelope in V1.
- Spec defers MCP server proxy execution to follow-up work while keeping config/schema boundaries.
- Spec references the Rust package selection research and requires reuse of mature crates before implementation.
- Spec references Swift + Rust large-project layout, development standards, and quality gates before implementation.

Evidence:

- Run a forbidden-term scan against `prd-spec/features/homie-v1-architecture/2026-08-05-homie-v1-architecture-design.md` to confirm reference-project names and internal names are absent.
- `git diff --check -- prd-spec/features/homie-v1-architecture/2026-08-05-homie-v1-architecture-design.md`
- `docs/architecture/project-layout.md`
- `docs/development/standards.md`
- `docs/development/quality-gates.md`
- `docs/research/rust-package-selection.md`

### T-002: Map PRD requirements to follow-up component specs

| Field | Value |
|-------|-------|
| Status | green |
| Source requirement | Section 5 |
| Beads | `homie-9c9` |
| Files | `openspec/changes/homie-v1-architecture/alignment-report.md` |

Objective:

- Identify which long-lived component specs must be created before implementation.

Acceptance:

- Every major architecture area has a component spec target or an explicit no-impact reason.

### T-003: Prepare implementation follow-up split

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | Section 9 |
| Beads | `homie-9c9` |
| Files | future Beads issues |

Objective:

- Split implementation into separate Beads issues and OpenSpec changes for runtime, LLM proxy, desktop shell, agent adapter, context store, and workspace bootstrap.

Acceptance:

- Follow-up issues are created before code implementation begins.
