# AGENTS.md

This file defines the working rules for AI agents contributing to this repository.

## Project Context

Homie is a Rust + GPUI desktop application inspired by Zed's high-performance native architecture. It will manage multiple background coding agents, including Codex, Claude Code, OpenCode, and future compatible agents.

Homie owns the unified LLM configuration entrypoint. Real provider credentials live in Homie's local configuration. Managed agents receive virtual keys and call Homie's OpenAI-compatible proxy. Homie then applies policy, records usage, and forwards requests to the configured provider.

Homie also owns global context, agent sessions, memory, tasks, intent recognition, and orchestration.

## Repository Rules

1. Do not preserve backward compatibility by default. Delete obsolete code directly. Do not add compatibility layers, migration paths, or fallbacks unless the user explicitly asks for them.

2. Choose the simplest implementation that satisfies the current requirement. Do not add preventive abstractions or unnecessary configuration layers.

3. Grow system layering gradually. First make a minimal end-to-end version work, then add structure on top. Do not break working behavior to make room for unfinished complexity.

4. Keep components modular with clear separation of concerns.

5. Prefer mature, actively maintained libraries. Do not rewrite existing proven functionality without a clear reason.

6. Inspect the project's existing dependencies and capabilities before adding a new package or implementing something from scratch.

7. Make architecture decisions for the long term. Do not accept temporary designs justified only by "we can replace it later."

8. Study how mature products solve the same problem and use validated patterns. Do not invent from zero when there is a proven approach.

## Required Development Workflow

Meaningful changes must follow this sequence:

1. Create or confirm a Beads issue with a stable `change_id`.
2. Write or update a Chinese PRD/spec under `prd-spec/`.
3. Evaluate affected long-lived component specs under `specs/`.
4. Read the project layout, development standards, quality gates, and package research before implementation.
5. Complete spec review and record evidence under `docs/verification/<change-id>/`.
6. Create or update `openspec/changes/<change-id>/plan.md` and `tasks.md`.
7. Prove PRD/spec to OpenSpec alignment in `openspec/changes/<change-id>/alignment-report.md` or `docs/verification/<change-id>/openspec-alignment-report.md`.
8. Implement with SDD/TDD from the OpenSpec tasks.
9. Run local verification and record results under `docs/verification/<change-id>/`.
10. Update or close the Beads issue only after evidence matches the delivered state.

Small documentation-only changes may use the lightweight version of this workflow, but they still need a clear source document and Beads linkage when they establish or change project process.

## Worktree Build Cache Rules

Homie worktrees must share one project-level Cargo build output directory.
Do not let every worktree create its own `homie/target` tree.

The local project-level shared target directory is:

```text
/Users/bytedance/workspace/github/homie-worktrees/.shared/homie-target
```

When creating a new Homie worktree, create `homie/target` as a symlink to that
shared directory before running Cargo or Homie packaging scripts:

```bash
mkdir -p /Users/bytedance/workspace/github/homie-worktrees/.shared
mkdir -p <new-worktree>/homie
ln -s /Users/bytedance/workspace/github/homie-worktrees/.shared/homie-target <new-worktree>/homie/target
```

If a worktree already has a real `homie/target` directory, inspect it first. If
it is disposable build output, move or remove it, then replace it with the
symlink. Never commit the symlink or the shared target directory; keep
`homie/target` ignored through local exclude rules when needed.

This symlink rule is preferred over writing `CARGO_TARGET_DIR` into tracked
repository configuration because existing Homie scripts default to
`homie/target`, and the symlink keeps those scripts and direct Cargo commands on
the same shared build cache.

## Beads Requirements Management

This repository uses Beads for local issue and dependency tracking. Use `bd` commands from the repository root.

Essential commands:

```bash
bd status
bd list
bd ready
bd show <bead-id> --long
bd create "<title>" --type feature --priority P0 --spec-id prd-spec/features/<change-id>/YYYY-MM-DD-<change-id>-design.md --metadata '{"change_id":"<change-id>"}'
bd update <bead-id> --claim
bd update <bead-id> --status blocked --notes "<reason>"
bd close <bead-id> --reason "Implemented and verified. See docs/verification/<change-id>/release-readiness-report.md."
```

Beads stores status, priority, ownership, dependencies, and links. It does not replace PRD/spec content. The canonical requirement text belongs in `prd-spec/`; long-lived engineering contracts belong in `specs/`; execution plans belong in `openspec/changes/`.

## Documentation Boundaries

| Layer | Location | Purpose |
|-------|----------|---------|
| Product overview | `README.md` | Project vision and top-level architecture |
| Requirement design | `prd-spec/` | Chinese PRD/spec for a feature, refactor, or bugfix |
| Component contract | `specs/` | Long-lived interfaces, data models, state machines, security, recovery, and tests |
| Change execution | `openspec/changes/<change-id>/` | Per-change plan, task breakdown, and alignment |
| Evidence | `docs/verification/<change-id>/` | Spec review, tests, E2E, security review, code review, and release readiness |
| Issue state | Beads (`bd`) | Status, dependencies, priority, assignee, and spec links |

Before implementation, read:

- `docs/architecture/project-layout.md`
- `docs/development/standards.md`
- `docs/development/quality-gates.md`
- `docs/research/rust-package-selection.md`

## PRD Spec Rules

All feature, refactor, and bugfix PRD/spec documents live under:

```text
prd-spec/
├── features/
├── refactors/
└── bugfixes/
```

Rules:

- Write PRD/spec documents in Chinese.
- Use kebab-case topic directories.
- Use filenames in the form `YYYY-MM-DD-<description>.md`.
- Do not overwrite historical design docs; create a new dated file for a new iteration.
- Include background, goals, non-goals, user scenarios, requirements, affected specs, test plan, acceptance criteria, and Beads tracking.

## Component Spec Rules

Use `specs/` for durable engineering contracts. Update component specs whenever a change affects:

- Rust module boundaries;
- GPUI state and interaction contracts;
- agent adapter interfaces;
- runtime supervisor behavior;
- LLM proxy protocol;
- credential or virtual key handling;
- context, memory, task, or orchestration state;
- storage schemas or indexes;
- logging, metrics, tracing, or recovery behavior.

## OpenSpec Rules

Each implementation change uses:

```text
openspec/changes/<change-id>/
├── plan.md
├── tasks.md
└── alignment-report.md
```

OpenSpec must map every PRD requirement to executable tasks and verification. Do not implement from chat context alone.

## Implementation Guidance

- Keep changes scoped to the current task.
- Read the existing code and dependencies before designing a solution.
- Prefer one complete vertical slice over disconnected partial systems.
- Keep agent adapters isolated from shared systems such as LLM proxying, context, memory, storage, and orchestration.
- Treat credential custody and virtual key issuance as security-sensitive code.
- Avoid leaking real provider keys into managed agent configuration.
- Add tests around behavior that affects agent launch, request proxying, persistence, credentials, or orchestration.

## Security Baseline

- Never commit real provider keys, virtual key signing secrets, Authorization headers, cookies, private keys, local agent credentials, raw prompts, or full tool arguments containing sensitive data.
- Keep real local configuration in ignored files such as `.env`, `*.local.toml`, `providers.local.*`, or `homie.local.*`.
- Commit only sanitized examples such as `.env.example` or `*.example.toml`.
- Use the repository hook path before committing:

```bash
git config core.hooksPath .githooks
```

- The hook in `.githooks/pre-commit` is a local safety net. If a secret is ever committed, rotate it immediately even if the commit is removed later.
