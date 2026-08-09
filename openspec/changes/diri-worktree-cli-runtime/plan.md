# OpenSpec Plan: Diri Worktree CLI Runtime

```yaml
change_id: diri-worktree-cli-runtime
beads: homie-ye8
prd: prd-spec/features/diri-worktree-cli-runtime/2026-08-08-diri-worktree-cli-runtime-design.md
```

## Scope

Implement the runtime/client/CLI path for Diri-equivalent `worktree.list/create/remove` using real git and local fixture E2E.

## Module Boundaries

| Layer | Files | Responsibility |
|-------|-------|----------------|
| Protocol | `homie-proto` | DTOs and method dispatch payloads |
| Runtime | `homie-runtime` | Git worktree subprocess operations and porcelain parser |
| Client | `homie-client` | Public methods and protocol dispatch |
| CLI | `homie-cli` | User-facing `worktree` commands |
| Evidence | `docs/verification/diri-worktree-cli-runtime` | Cases, verification, review |

## Acceptance

The worktree create/list/remove E2E must use a real temporary git repository and prove the created path exists then is removed.
