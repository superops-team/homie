# Workspace Storage Bootstrap OpenSpec Plan

> Change ID: `workspace-storage-bootstrap`
> Source PRD: `prd-spec/features/workspace-storage-bootstrap/2026-08-05-workspace-storage-bootstrap-design.md`
> Beads: `homie-mgl`
> Status: implementation-ready

## 1. Summary

Initialize the first executable Homie vertical slice: Rust workspace, `homie-proto` ID types, `homie-storage` SQLite migration and health check, `homie-cli doctor`, and basic quality gate entrypoints.

This change intentionally excludes GPUI app, runtime socket, PTY, Codex adapter, LLM proxy, virtual key issuance, encrypted secret envelope, and MCP proxy implementation.

## 2. Goals

| Goal | Source requirement | Functional cases |
|------|--------------------|------------------|
| G-1 | FR-1 Rust workspace | FC-005 |
| G-2 | FR-2 proto IDs | FC-005 |
| G-3 | FR-3 storage open/migrate/health | FC-001, FC-002, FC-003 |
| G-4 | FR-4 SQLite schema | FC-003, FC-004 |
| G-5 | FR-5 CLI doctor | FC-001, FC-002 |
| G-6 | FR-6 quality entrypoints | FC-005, FC-006 |

## 3. Non-Goals

- No GPUI app.
- No background runtime socket.
- No PTY/session process.
- No LLM proxy.
- No secret encryption implementation.
- No MCP proxy.

## 4. Affected Component Specs

| Component spec | Impact | Required update |
|----------------|--------|-----------------|
| `specs/storage-indexing/README.md` | yes | Create storage contract before implementation |
| `specs/virtual-key-credentials/README.md` | no | Secret envelope only referenced as future |
| `specs/agent-adapter-contract/README.md` | no | Adapter not implemented |
| `specs/llm-proxy/README.md` | no | Proxy not implemented |

## 5. Implementation Scope

| Area | Files/modules | Reason |
|------|---------------|--------|
| Rust workspace | `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` | Build foundation |
| Proto | `crates/homie-proto/` | Shared ID and error types |
| Storage | `crates/homie-storage/` | SQLite migration and health |
| CLI | `crates/homie-cli/` | `doctor` smoke command |
| Quality | `Makefile` | repeatable gates |
| Component spec | `specs/storage-indexing/README.md` | long-lived storage contract |

## 6. Test Strategy

| Layer | Required cases | Evidence |
|-------|----------------|----------|
| Unit | ID serialization, storage path/report | test report |
| Integration | SQLite migration, constraints, usage schema | FC-003, FC-004 |
| Functional | CLI doctor create/idempotent | FC-001, FC-002 |
| Quality | make pre-commit and secret scan | FC-005, FC-006 |

## 7. Release Gates

- All functional cases pass or are honestly marked blocked.
- `cargo fmt --all -- --check` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
- `cargo test --workspace` passes.
- `.githooks/pre-commit` passes.
- Evidence is recorded under `docs/verification/workspace-storage-bootstrap/`.
