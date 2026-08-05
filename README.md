# Homie

Homie is a Rust + GPUI desktop application for running, supervising, and coordinating multiple local coding agents from one high-performance cross-platform workspace.

The project takes inspiration from Zed's Rust-native desktop architecture: keep the UI fast, keep core state local-first, and use explicit systems for process supervision, context, memory, tasks, and model access instead of scattering those concerns across individual agent CLIs.

## What Homie Is Building

Homie is designed to be the control plane for agentic development work.

It will manage background agents such as Codex, Claude Code, OpenCode, and future compatible agents. Each agent can keep its native runtime model, but Homie owns the surrounding environment: configuration, credentials, session records, context policy, memory, task state, and orchestration.

The long-term goal is not to wrap one assistant UI. The goal is to provide a single local desktop system where multiple agents can be launched, observed, interrupted, delegated to, and composed into useful workflows.

## Core Capabilities

- Cross-platform desktop application built with Rust and GPUI.
- Background process management for multiple agent runtimes.
- Unified LLM configuration entrypoint.
- Local credential custody with virtual keys issued to managed agents.
- OpenAI-compatible proxy for agent model requests.
- Central session log for every agent conversation and execution.
- Shared context store across agents, tasks, and workspaces.
- Unified memory layer for durable project and user knowledge.
- Task management that can be read and updated by agents.
- Built-in intent recognition and orchestration engine.

## LLM Access Model

Homie owns the real model provider configuration.

Instead of asking every agent to store and manage its own API keys, Homie will keep the real credentials in its own local configuration and issue virtual keys to managed agents. Agents call an OpenAI-compatible endpoint exposed by Homie. Homie validates the virtual key, resolves the configured provider, applies policy, records usage, and forwards the request.

This gives the application one place to handle:

- provider selection;
- model aliases;
- request tracing;
- token and cost accounting;
- rate limits;
- workspace policy;
- redaction and audit rules;
- future routing across local and remote models.

The virtual key is not the source of authority. It is a scoped handle that lets Homie attribute and control requests from each managed agent.

## Agent Management

Homie will run agents as supervised background processes. The first implementation should focus on a minimal end-to-end path:

1. Register an agent profile.
2. Launch the agent with a Homie-managed environment.
3. Provide the agent with a virtual OpenAI-compatible key.
4. Capture its session metadata and output.
5. Store the conversation and execution context in Homie's state.
6. Show the running session in the desktop UI.

Agent integrations should stay modular. Each adapter should be responsible for one agent runtime's process model, configuration shape, and I/O protocol. Shared concerns such as credentials, proxying, session storage, tasks, and memory belong in Homie's core systems.

## Context, Memory, and Tasks

Homie maintains global context as a first-class system. Individual agents should not become isolated islands of state.

The context layer will track active workspaces, files, sessions, prompts, tool results, task state, and useful long-term memory. The memory layer should distinguish between short-lived session context and durable facts that should survive across sessions.

Tasks are also part of the shared state. Agents should be able to receive tasks, update progress, emit blockers, and hand work back to the orchestrator without each agent inventing its own task database.

## Intent Recognition and Orchestration

Homie will include an intent recognition and orchestration engine. Its job is to decide what kind of work the user is asking for, which agent or workflow should handle it, what context should be attached, and what permissions or confirmations are required.

The orchestrator should start simple:

- classify the user request;
- choose a target agent or internal workflow;
- attach the minimum useful context;
- supervise execution;
- persist the result.

More complex multi-agent planning should be added only after the minimal end-to-end flow works reliably.

## Architecture Direction

The project is expected to grow around a few durable boundaries:

- `desktop`: GPUI application shell, windows, panes, and user interaction.
- `core`: domain types, application state, task model, and orchestration contracts.
- `agents`: runtime adapters for Codex, Claude Code, OpenCode, and other tools.
- `llm`: provider configuration, virtual key issuance, OpenAI-compatible proxying, and request policy.
- `context`: session records, workspace context, prompt history, and retrieval APIs.
- `memory`: durable user and project memory.
- `storage`: local persistence, migrations when needed, and indexing.

These names are architectural direction, not a promise that every module must exist on day one. The first working version should prove the full loop before adding more layers.

## Development Workflow

Homie uses a document-first workflow for meaningful changes:

1. Track the requirement in Beads.
2. Write the Chinese PRD/spec under `prd-spec/`.
3. Update long-lived component contracts under `specs/` when interfaces, state, data, credentials, security, recovery, or observability are affected.
4. Break the change into executable OpenSpec tasks under `openspec/changes/<change-id>/`.
5. Record verification evidence under `docs/verification/<change-id>/`.

Key entrypoints:

- [AGENTS.md](./AGENTS.md): repository rules for AI agents and contributors.
- [prd-spec/README.md](./prd-spec/README.md): PRD/spec directory and document templates.
- [specs/README.md](./specs/README.md): long-lived component specification model.
- [openspec/README.md](./openspec/README.md): per-change planning and task workflow.
- [docs/workflows/requirements-management.md](./docs/workflows/requirements-management.md): Beads + PRD/spec + OpenSpec requirements management.
- [docs/verification/report-templates/README.md](./docs/verification/report-templates/README.md): verification report templates.
- [docs/security/pre-commit.md](./docs/security/pre-commit.md): local pre-commit secret scanning baseline.
- [docs/architecture/project-layout.md](./docs/architecture/project-layout.md): Swift + Rust large-project layout and ownership boundaries.
- [docs/development/standards.md](./docs/development/standards.md): Rust/Swift development standards.
- [docs/development/quality-gates.md](./docs/development/quality-gates.md): evidence-first quality gates and release criteria.
- [docs/research/rust-package-selection.md](./docs/research/rust-package-selection.md): reusable Rust package selection research.

Beads is initialized with the `homie` issue prefix. Use `bd status`, `bd list`, `bd ready`, and `bd show <bead-id> --long` from the repository root to inspect work.

Enable the repository hooks once per clone:

```bash
git config core.hooksPath .githooks
```

## Development Principles

- Build the smallest complete vertical slice first.
- Keep components modular and responsibilities explicit.
- Prefer mature maintained Rust crates and existing project dependencies.
- Study how mature products solve the same problem before inventing new patterns.
- Avoid compatibility layers for obsolete internal designs.
- Make long-lived architecture decisions deliberately.
- Do not add speculative configuration or abstraction.

## Current Status

Homie is at the project initialization stage. The README defines the intended product and architecture direction; implementation details will evolve as the first end-to-end agent management flow is built.

## License

MIT
