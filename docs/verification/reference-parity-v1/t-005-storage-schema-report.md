# T-005 Storage Schema And Preferences Report

```yaml
change_id: reference-parity-v1
openspec_task: T-005
beads: homie-tjy
status: pass
functional_cases:
  - FC-004
  - FC-007
  - FC-018
```

## 1. Summary

T-005 extended `homie-storage` from schema version 1 to schema version 2 with Reference parity product-state tables.

Implemented:

- Forward migration from v1 to v2.
- `preferences`.
- `projects`.
- `worktrees`.
- `session_artifacts`.
- `listening_ports`.
- `pull_request_statuses`.
- `history_entries`.
- `hosts`.
- `node_accounts`.
- `handoff_records`.
- `memory_candidates`.
- Unique artifact constraint by session/kind/url.

## 2. RED

Added failing schema tests:

- `crates/homie-storage/tests/reference_parity_schema.rs`

Initial failure:

- `preferences` and other Reference parity tables did not exist after migration.

## 3. GREEN

Implemented:

- `SCHEMA_VERSION = 2`.
- v1 then v2 forward migration path.
- `SCHEMA_V2` product-state schema.
- Updated storage bootstrap tests to expect schema version 2.

## 4. Verification

Focused command:

```bash
cargo test -p homie-storage
```

Result:

- Exit code: 0
- Local basic tests: 3 passed
- Reference parity schema tests: 2 passed
- Storage bootstrap tests: 4 passed

Workspace regression command:

```bash
cargo test --workspace
```

Result:

- Exit code: 0
- Homie agents/app/CLI/proto/storage tests passed.

Safety checks:

```bash
rg -n -i "<old-reference-name-pattern>" .
git diff --check
```

Result:

- Old reference name scan: no matches.
- Markdown/patch whitespace check: pass.

## 5. Remaining Scope

Still deferred:

- Repository APIs for the new tables.
- Runtime integration for worktrees/artifacts/history/hosts.
- Real FC-007 and FC-010 execution after runtime implementation.

