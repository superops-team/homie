# OpenSpec Plan: Diri host.locate_repo

```yaml
change_id: diri-host-locate-repo
beads: homie-37j
prd: prd-spec/features/diri-host-locate-repo/2026-08-08-diri-host-locate-repo-design.md
```

## Scope

本 change 只实现 Diri `host.locate_repo` 的第一阶段本地可验证闭环：协议 DTO、origin 发现、候选路径匹配、storage-backed client handler 和 CLI fixture E2E。

## Module Boundaries

| Layer | Files | Responsibility |
|-------|-------|----------------|
| Protocol | `crates/homie-proto/src/lib.rs` | Diri-compatible method DTO and JSON spelling |
| Remote domain | `crates/homie-remote/src/lib.rs` | Read-only repo origin discovery and candidate matching |
| Storage/client | `crates/homie-storage`, `crates/homie-client` | Project fact lookup and `host.locate_repo` dispatch |
| CLI | `crates/homie-cli/src/main.rs` | User/automation entrypoint with JSON output |
| Evidence | `docs/verification/diri-host-locate-repo/` | Spec review, cases, verification, review, readiness |

## Dependencies

- Depends on existing storage project/worktree schema from `diri-storage-indexing`.
- Does not depend on macOS screenshots or real remote SSH.
- Does not introduce new third-party dependencies.

## Acceptance

- FR-1 covered by proto serialization test.
- FR-2/FR-3 covered by `homie-remote` fixture tests.
- FR-4/FR-5 covered by CLI integration test using temp git-like fixture.
- `REM-003` parity evidence updated from prefs-only to prefs plus locate-repo foundation.
