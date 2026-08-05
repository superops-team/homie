# Local Basic V1 OpenSpec Plan

> Change ID: `local-basic-v1`
> Source PRD: `prd-spec/features/local-basic-v1/2026-08-05-local-basic-v1-design.md`
> Beads: `homie-54y`
> Status: implementation-ready

## 1. Summary

Complete the local basic V1 slice on top of workspace-storage-bootstrap: seed default Codex profile config, create/list session records through CLI, expose runtime status, add install and package scripts, and wire smoke/full-check gates.

## 2. Goals

| Goal | Requirement | Functional cases |
|------|-------------|------------------|
| G-1 | FR-1 seed defaults | FC-001 |
| G-2 | FR-2 session repository | FC-002 |
| G-3 | FR-3 CLI commands | FC-001, FC-002, FC-003 |
| G-4 | FR-4 install-local | FC-004 |
| G-5 | FR-5 package tarball | FC-005 |
| G-6 | FR-6 full gate | FC-006 |

## 3. Non-Goals

- No GPUI app.
- No Codex process spawn.
- No PTY.
- No LLM proxy.
- No MCP proxy.

## 4. Implementation Scope

| Area | Files |
|------|-------|
| Storage | `crates/homie-storage/` |
| CLI | `crates/homie-cli/` |
| Scripts | `scripts/dev/install-local.sh`, `scripts/package/package.sh` |
| Gates | `Makefile` |
| Evidence | `docs/verification/local-basic-v1/` |

## 5. Release Gates

- `make full-check` passes.
- `scripts/dev/install-local.sh --prefix <tmp>` installs a runnable CLI.
- `scripts/package/package.sh` creates a tarball.
- unpacked tarball binary runs `doctor`.
