# Local Basic V1 OpenSpec Tasks

> Change ID: `local-basic-v1`
> Source PRD: `prd-spec/features/local-basic-v1/2026-08-05-local-basic-v1-design.md`
> Functional Cases: `docs/verification/local-basic-v1/functional-cases.md`
> Beads: `homie-54y`

## Tasks

### T-001: Seed default Codex configuration

| Field | Value |
|-------|-------|
| Status | todo |
| Requirement | FR-1 |
| Functional cases | FC-001 |
| Files | `crates/homie-storage/` |

Acceptance:

- `doctor` seeds provider, LLM profile, runtime descriptor, permission profile, default agent profile.
- Seed is idempotent.

### T-002: Session repository

| Field | Value |
|-------|-------|
| Status | todo |
| Requirement | FR-2 |
| Functional cases | FC-002 |
| Files | `crates/homie-storage/` |

Acceptance:

- `create_session` uses enabled default profile.
- `list_sessions` returns stable summaries.

### T-003: CLI commands

| Field | Value |
|-------|-------|
| Status | todo |
| Requirement | FR-3 |
| Functional cases | FC-001, FC-002, FC-003 |
| Files | `crates/homie-cli/` |

Acceptance:

- `doctor`, `runtime status`, `session create`, `session list` all support `--json`.

### T-004: Install script

| Field | Value |
|-------|-------|
| Status | todo |
| Requirement | FR-4 |
| Functional cases | FC-004 |
| Files | `scripts/dev/install-local.sh` |

Acceptance:

- Installs release CLI to `<prefix>/bin/homie`.

### T-005: Package script

| Field | Value |
|-------|-------|
| Status | todo |
| Requirement | FR-5 |
| Functional cases | FC-005 |
| Files | `scripts/package/package.sh` |

Acceptance:

- Produces `dist/homie-<version>-<target>.tar.gz`.
- Tarball contains `bin/homie`, `README.md`, `LICENSE`.

### T-006: Full gate

| Field | Value |
|-------|-------|
| Status | todo |
| Requirement | FR-6 |
| Functional cases | FC-006 |
| Files | `Makefile` |

Acceptance:

- `make full-check` passes.
