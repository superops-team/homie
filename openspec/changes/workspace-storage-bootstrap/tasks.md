# Workspace Storage Bootstrap OpenSpec Tasks

> Change ID: `workspace-storage-bootstrap`
> Source PRD: `prd-spec/features/workspace-storage-bootstrap/2026-08-05-workspace-storage-bootstrap-design.md`
> Functional Cases: `docs/verification/workspace-storage-bootstrap/functional-cases.md`
> Beads: `homie-mgl`

## Tasks

### T-001: Create Rust workspace

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-1 |
| Functional cases | FC-005 |
| Files | `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, crate manifests |

RED:

- Before implementation, `cargo check --workspace` cannot run because no workspace exists.

GREEN:

- Create workspace and empty crates.
- `cargo check --workspace` succeeds.

Acceptance:

- Workspace resolver 2.
- `Cargo.lock` generated.
- Toolchain includes rustfmt/clippy.

### T-002: Implement proto ID types

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-2 |
| Functional cases | FC-005 |
| Files | `crates/homie-proto/` |

RED:

- Add tests for ID creation and serde roundtrip before implementing wrappers.

GREEN:

- Implement ID newtypes and shared error envelope basics.

Acceptance:

- Tests pass.
- `homie-proto` has no dependency on storage/runtime/UI.

### T-003: Implement SQLite storage migration

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-3, FR-4 |
| Functional cases | FC-001, FC-002, FC-003, FC-004 |
| Files | `crates/homie-storage/`, `specs/storage-indexing/README.md` |

RED:

- Add integration tests for migration idempotency, foreign keys, unique constraints, default profile constraint, and usage schema.

GREEN:

- Implement `Storage::open_or_create`, `migrate`, `health_check`.
- Implement schema version 1.

Acceptance:

- `cargo test -p homie-storage sqlite_constraints -- --nocapture` passes.
- `cargo test -p homie-storage usage_metrics_schema -- --nocapture` passes.

### T-004: Implement CLI doctor

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-5 |
| Functional cases | FC-001, FC-002 |
| Files | `crates/homie-cli/` |

RED:

- Add CLI integration test or command test for `doctor --data-dir <tmp> --json`.

GREEN:

- Implement clap-based `doctor`.
- Emit stable JSON and human-readable output.

Acceptance:

- Doctor creates SQLite in empty data dir.
- Doctor is idempotent.

### T-005: Add quality gate entrypoints

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-6 |
| Functional cases | FC-005, FC-006 |
| Files | `Makefile` |

RED:

- `make pre-commit` does not exist.

GREEN:

- Add `fmt`, `lint`, `test`, `security`, `pre-commit`.

Acceptance:

- `make pre-commit` passes after implementation.
- `.githooks/pre-commit` remains part of `make security`.
