# <Report Title>

```yaml
change_id: <change-id>
report_type: spec-review | openspec-alignment | sdd-tdd-task | test | e2e | security-review | code-review | release-readiness
status: pass | blocked | not_run | partial
beads: <bead-id>
generated_at: YYYY-MM-DD HH:mm:ss <timezone>
executor: TRAE CLI
```

## 1. Scope

| Item | Value |
|------|-------|
| Source PRD | `prd-spec/...` |
| Component specs | `specs/...` |
| OpenSpec change | `openspec/changes/<change-id>/` |
| Diff/branch | ... |

## 2. Commands Or Review Inputs

| Command/input | Exit/status | Summary |
|---------------|-------------|---------|
| `cargo test` | not_run | Rust project not initialized yet |

## 3. Findings

| ID | Severity | Location | Finding | Action | Status |
|----|----------|----------|---------|--------|--------|
| F-1 | P0/P1/P2 | `file:line` | ... | ... | open/fixed/accepted |

## 4. Verification Result

| Check | Result | Evidence |
|-------|--------|----------|
| ... | pass/blocked/not_run | ... |

## 5. Risks And Follow-Ups

| Risk | Impact | Mitigation | Beads |
|------|--------|------------|-------|
| ... | ... | ... | ... |

## 6. Gate Decision

Decision: pass | blocked | not_run | partial

Reason:

- ...
