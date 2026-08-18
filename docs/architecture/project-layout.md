# Homie Project Layout

## 1. Purpose

This document is the required orientation map for Homie development. Read it
before changing Rust modules, GPUI surfaces, Swift protocol glue, packaging, or
worktree setup.

## 2. Repository Roots

| Path | Purpose |
|------|---------|
| `README.md` | Product overview and top-level usage |
| `AGENTS.md` | Required AI-agent workflow, Beads, PRD/spec, OpenSpec, security, and worktree build cache rules |
| `Package.swift` | Swift CLI/protocol/core package |
| `homie/` | Rust workspace and desktop app |
| `prd-spec/` | Chinese feature/refactor/bugfix PRD/spec documents |
| `specs/` | Long-lived engineering contracts |
| `openspec/changes/` | Per-change execution plans and task breakdowns |
| `docs/verification/` | Evidence, functional cases, code review, E2E, release readiness |
| `.beads/` | Local Beads issue database |

## 3. Rust Workspace

`homie/Cargo.toml` owns the Rust workspace.

| Crate | Responsibility |
|-------|----------------|
| `homie-app` | GPUI desktop app, window shell, sidebar, terminal, inspector, settings and app UI orchestration |
| `homie-ui` | Shared GPUI visual tokens and reusable UI components |
| `homie-engine` | Local daemon/runtime, session supervision, control protocol serving |
| `homie-client` | Client API for app/CLI to talk to the daemon |
| `homie-proto` | Rust protocol DTOs and paths |
| `homie-term` | GPUI terminal rendering support |
| `homie-terminal-state` | Terminal state model shared by local/remote runtimes |
| `homie-pty` | PTY abstraction |
| `homie-node` | Remote node service |
| `homie-remote` | Remote helper package |
| `homie-mcp` | MCP bridge binary |
| `homie-updater` | Update/install support |
| `homie-usage` | Usage domain types and helpers |
| `homie-gateway` | Local LLM gateway: virtual keys, OpenAI/Anthropic-compatible proxy, upstream forwarding, per-key usage |

## 4. Swift Package

Swift is retained for CLI/protocol/core/MCP glue and shared package resources.
Rust Engine is the daemon/runtime authority. Do not reintroduce Swift daemon or
holder paths without a new PRD/spec.

| Target Area | Responsibility |
|-------------|----------------|
| `Sources/homie-cli` | CLI entrypoint and user-facing commands |
| `Sources/HomieProtocol` | Swift protocol DTOs used by CLI surfaces |
| `Sources/HomieCore` | Swift core types and generated/packaged resources |
| `Sources/HomieMCP` | Swift MCP support |
| `Tests/` | Swift CLI/protocol/core tests |

## 5. GPUI App Layout

Important GPUI files:

| Path | Current Role |
|------|--------------|
| `homie/crates/homie-app/src/main.rs` | App startup, runtime setup, menus, window creation |
| `homie/crates/homie-app/src/root.rs` | Top-level GPUI root view and current composition root |
| `homie/crates/homie-app/src/workbench.rs` | Pure workbench split layout state |
| `homie/crates/homie-app/src/sidebar/` | Sidebar state, fixtures, and view rendering |
| `homie/crates/homie-app/src/surface_shell.rs` | Utility overlays: history, worktrees, settings, remote host editor |
| `homie/crates/homie-app/src/terminal_pane.rs` | Terminal GPUI pane |
| `homie/crates/homie-app/src/inspector.rs` | Inspector and artifact/review surfaces |
| `homie/crates/homie-ui/src/tokens.rs` | Shared type, color, radius and spacing concepts |
| `homie/crates/homie-ui/src/components.rs` | Shared reusable GPUI components |

New GPUI work should respect `specs/gpui-shell.md`,
`specs/gpui-interaction-contract.md`, and `specs/ui-components.md`.

## 6. Worktree Layout And Build Cache

Homie uses sibling worktrees for parallel development. Current known locations:

```text
/Users/bytedance/workspace/github/homie
/Users/bytedance/workspace/github/homie-gpui-audit-20260814
/Users/bytedance/workspace/github/homie-worktrees/<task-name>
```

All Homie worktrees on this machine must share one Cargo target directory via
`homie/target` symlink. The authoritative rule is in `AGENTS.md`.

Current shared target instance:

```text
/Users/bytedance/workspace/github/homie-worktrees/.shared/homie-target
```

Do not add the symlink or shared target directory to tracked files.

## 7. Documentation Boundaries

| Layer | Location | Rule |
|-------|----------|------|
| Product overview | `README.md` | Top-level product narrative only |
| Requirement design | `prd-spec/` | Chinese feature/refactor/bugfix documents |
| Durable contract | `specs/` | Component state machines, module contracts, security/recovery/testing contracts |
| Execution plan | `openspec/changes/<change-id>/` | Per-change plan, tasks, alignment report |
| Evidence | `docs/verification/<change-id>/` | Functional cases, command logs, review reports, release readiness |

Do not replace durable specs with chat context or implementation comments.
