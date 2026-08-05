# Homie V1 Architecture OpenSpec Plan

> Change ID: `homie-v1-architecture`
> Source PRD: `prd-spec/features/homie-v1-architecture/2026-08-05-homie-v1-architecture-design.md`
> Beads: `homie-9c9`
> Status: design

## 1. Summary

This change defines Homie's first implementable architecture spec. It does not implement runtime code yet. The output is a PRD/spec that can drive follow-up component specs and implementation OpenSpec changes.

## 2. Goals

| Goal | Source requirement | Acceptance |
|------|--------------------|------------|
| G-1 | FR-1 | Rust workspace and crate boundaries are specified |
| G-2 | FR-2 | App/runtime process model is specified |
| G-3 | FR-3 | Control protocol and event model are specified |
| G-4 | FR-4 | Agent adapter contract and Codex as the default V1 runtime are specified |
| G-5 | FR-4A | Agent profile registry for runtime, skills, MCP config, permissions, and LLM profile is specified |
| G-6 | FR-5 | PTY/session runtime responsibilities are specified |
| G-7 | FR-7 | LLM provider config, virtual key, and proxy model are specified |
| G-7A | FR-7A | Token usage, cache hit rate, estimated cost, request latency, and tool-call latency metrics are specified |
| G-8 | FR-8/FR-9 | Context, memory, task, and orchestrator V1 boundaries are specified |
| G-9 | FR-10 | SQLite local storage, relationships, migrations, and output-log indexing are specified |
| G-10 | FR-11/FR-12 | Desktop UI and CLI boundaries are specified |

## 3. Non-Goals

- No Rust workspace implementation in this change.
- No GPUI UI code.
- No agent runtime code.
- No provider credentials or local secrets.

## 4. Affected Component Specs

| Component spec | Impact | Required update |
|----------------|--------|-----------------|
| `specs/desktop-shell/README.md` | yes | Create before UI implementation |
| `specs/runtime-supervisor/README.md` | yes | Create before runtime implementation |
| `specs/agent-adapter-contract/README.md` | yes | Create before first agent adapter |
| `specs/llm-proxy/README.md` | yes | Create before proxy implementation |
| `specs/virtual-key-credentials/README.md` | yes | Create before credential implementation |
| `specs/session-context-store/README.md` | yes | Create before context implementation |
| `specs/storage-indexing/README.md` | yes | Create before SQLite schema implementation |
| `specs/memory-controller/README.md` | yes | Create before memory implementation |
| `specs/task-controller/README.md` | yes | Create before task implementation |
| `specs/intent-orchestrator/README.md` | yes | Create before orchestrator implementation |

## 5. Implementation Scope

| Area | Files/modules | Reason |
|------|---------------|--------|
| PRD spec | `prd-spec/features/homie-v1-architecture/...` | First-version architecture source of truth |
| OpenSpec | `openspec/changes/homie-v1-architecture/` | Trace spec requirements to next tasks |

## 6. Test Strategy

| Layer | Required cases | Command or evidence |
|-------|----------------|---------------------|
| Terminology check | Reference project names do not appear in Homie spec | `rg` check |
| Markdown hygiene | No trailing whitespace | `git diff --check` |
| Secret scan | No staged secret pattern | `.githooks/pre-commit` |
| Dependency research | Mature reusable Rust crates are identified before implementation | `docs/research/rust-package-selection.md` |
| Project standards | Swift + Rust layout, development standards, and quality gates are defined | `docs/architecture/project-layout.md`, `docs/development/standards.md`, `docs/development/quality-gates.md` |
| Reference feature coverage | Reference feature surface is mapped into Homie V1/V1.x scope | `docs/research/reference-feature-coverage.md` |

## 7. Release Gates

- PRD/spec exists and is self-contained.
- Beads issue `homie-9c9` points to the PRD/spec.
- Dependency research document exists and is referenced by the PRD/spec.
- Layout, development standards, and quality gates exist and are referenced by the PRD/spec.
- Reference feature coverage matrix exists and missing feature surfaces are represented in PRD/spec or V1.x roadmap.
- Terminology check passes.
- Markdown hygiene passes.
