# Local Basic V1 Release Readiness Report

```yaml
change_id: local-basic-v1
report_type: release-readiness
status: pass
beads: homie-54y
```

## 1. Scope

| Item | Value |
|------|-------|
| Source PRD | `prd-spec/features/local-basic-v1/2026-08-05-local-basic-v1-design.md` |
| OpenSpec | `openspec/changes/local-basic-v1/` |
| Functional cases | `docs/verification/local-basic-v1/functional-cases.md` |
| Beads | `homie-54y` |

## 2. Gate Summary

| Gate | Evidence | Status |
|------|----------|--------|
| doctor seeds defaults | `artifacts/fc-001-doctor.json` | pass |
| session create/list | `artifacts/fc-002-session.json` | pass |
| runtime status | `artifacts/fc-003-runtime-status.json` | pass |
| install local | `artifacts/fc-004-install-local.txt` | pass |
| package tarball | `artifacts/fc-005-package.txt` | pass |
| full-check | `artifacts/fc-006-full-check.txt` | pass |

## 3. Commands

| Command | Result |
|---------|--------|
| `make full-check` | pass |
| `scripts/dev/install-local.sh --prefix <tmp>` | pass |
| installed `homie doctor --data-dir <tmp> --json` | pass |
| `scripts/package/package.sh` | pass |
| unpacked tarball `bin/homie doctor --data-dir <tmp> --json` | pass |

## 4. Package

Generated artifact:

```text
dist/homie-0.1.0-aarch64-apple-darwin.tar.gz
```

Tarball contents:

```text
homie-0.1.0-aarch64-apple-darwin/
homie-0.1.0-aarch64-apple-darwin/LICENSE
homie-0.1.0-aarch64-apple-darwin/README.md
homie-0.1.0-aarch64-apple-darwin/bin/homie
```

## 5. Not Run

| Gate | Reason |
|------|--------|
| GPUI app package | GPUI app not in this slice |
| Codex process runtime | Runtime process not in this slice |
| LLM proxy E2E | LLM proxy not in this slice |
| Swift build/test | Swift package not introduced |

## 6. Gate Decision

Decision: pass

Reason:

- The local basic V1 CLI can initialize storage, seed default Codex profile configuration, create/list sessions, report runtime status, install locally, and package as a tarball.
- All required gates for this slice passed.
