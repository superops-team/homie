# Persistence Incremental State Plan

## 1. Scope

First slice for `persistence-incremental-state`.

## 2. In Scope

- Define a narrow `PersistenceStore` trait.
- Implement `JsonEnvelopeStore` for existing `{version, projects, sessions}` state files.
- Implement `SplitJsonStore` for `projects.json` + `sessions/<id>.json`.
- Implement dry-run and apply migration from envelope to split store.
- Keep the existing `Registry` default path on envelope persistence.
- Add tests for dry-run, apply migration, backup and single-session quarantine.

## 3. Out Of Scope

- SQLite.
- Default enablement of split store for real users.
- OutputLog changes.
- Remote binding changes.
- Provider config/credential storage.
- Cloud sync.

## 4. Design

The first slice provides the store and migration primitives without switching production Registry persistence.

`SplitJsonStore` layout:

```text
<root>/
├── projects.json
└── sessions/
    ├── s_1.json
    └── s_2.json
```

Migration:

- dry-run reads and validates source envelope but writes nothing;
- apply creates split files and copies the original envelope to a backup path;
- source `state.json` remains in place;
- a corrupt split session file is renamed with `.corrupt` suffix and skipped.

## 5. Evidence

- Spec review: `docs/verification/persistence-incremental-state/spec-review-report.md`
- Functional cases: `docs/verification/persistence-incremental-state/functional-cases.md`
- Functional verification: `docs/verification/persistence-incremental-state/functional-verification-report.md`
- Code review: `docs/verification/persistence-incremental-state/code-review-round-1.md`, `code-review-round-2.md`
- Release readiness: `docs/verification/persistence-incremental-state/release-readiness-report.md`

## 6. Risks

| Risk | Control |
|---|---|
| User data migration loss | Production Registry default remains envelope; migration tests use temp files |
| Dry-run writes accidentally | FC-03 asserts no target files |
| Split store hides corrupt file | FC-05 asserts quarantine path exists |
| Store trait grows too wide | First slice only covers projects/sessions CRUD |
