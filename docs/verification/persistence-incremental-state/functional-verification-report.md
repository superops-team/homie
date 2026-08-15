# Persistence Incremental State Functional Verification Report

## 1. 结论

`persistence-incremental-state` 首阶段功能验证通过。

已完成：

- `PersistenceStore` trait。
- `JsonEnvelopeStore` wrapper for the existing envelope format。
- `SplitJsonStore` for `projects.json` and `sessions/<id>.json`。
- Envelope to split migration with dry-run and backup。
- Corrupt single-session file quarantine。

未做：

- 不默认切换 Registry production path。
- 不引入 SQLite。
- 不改 OutputLog、remote bindings、provider config/credentials。

## 2. Case 执行结果

| Case | 状态 | 证据 |
|---|---|---|
| FC-01 PRD/spec 和 review 风险已收敛 | pass | `fc-01-spec-review.log` |
| FC-02 OpenSpec 三件套完整并覆盖 Case | pass | `fc-02-openspec-alignment.log` |
| FC-03 split store 和 dry-run migration | pass | `fc-03-dry-run.log` |
| FC-04 apply migration 保留 backup 并可加载 split store | pass | `fc-04-apply-migration.log` |
| FC-05 单 session corruption quarantine | pass | `fc-05-quarantine.log` |
| FC-06 静态门禁和范围守卫 | pass | `fc-06-static-gates.log` |

## 3. 关键证据

- `split_store_dry_run_migration_writes_nothing` passed。
- `split_store_migration_preserves_backup_and_loads_records` passed。
- `split_store_quarantines_one_corrupt_session_file` passed。
- `cargo fmt --manifest-path homie/Cargo.toml --all -- --check` passed。
- `git diff --check` passed。

## 4. 残余风险

- Production `Registry` still defaults to the envelope store. This is intentional: split store default enablement should be a later change after real data fixtures and performance evidence.
- Performance baseline for 100/1000/5000 sessions is deferred because this slice only lands store and migration primitives.
