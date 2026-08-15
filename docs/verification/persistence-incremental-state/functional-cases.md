# Persistence Incremental State 功能验证 Case

## 1. 验证目标

本验证 Case 面向 `persistence-incremental-state` 首阶段，目标是证明：

- split JSON store 能加载/保存 projects 和 sessions；
- 旧 envelope `state.json` 可 dry-run 和 apply 迁移；
- apply 迁移保留旧 `state.json` backup；
- dry-run 不写目标文件；
- 单个 session 文件损坏只 quarantine 该 session；
- 本阶段不默认切换 Registry 主路径，不引入 SQLite。

## FC-01: PRD/spec 和 review 风险已收敛

```bash
test -s docs/verification/persistence-incremental-state/spec-review-report.md
rg -n "首阶段关闭口径|dry-run|backup|quarantine|不引入 SQLite|opt-in" \
  prd-spec/refactors/persistence-incremental-state/2026-08-13-persistence-incremental-state-design.md \
  docs/verification/persistence-incremental-state/spec-review-report.md
```

证据路径：`docs/verification/persistence-incremental-state/fc-01-spec-review.log`

## FC-02: OpenSpec 三件套完整并覆盖 Case

```bash
test -s openspec/changes/persistence-incremental-state/plan.md
test -s openspec/changes/persistence-incremental-state/tasks.md
test -s openspec/changes/persistence-incremental-state/alignment-report.md
rg -n "FC-01|FC-02|FC-03|FC-04|FC-05|FC-06" \
  openspec/changes/persistence-incremental-state/tasks.md \
  openspec/changes/persistence-incremental-state/alignment-report.md
```

证据路径：`docs/verification/persistence-incremental-state/fc-02-openspec-alignment.log`

## FC-03: split store 和 dry-run migration

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-engine split_store_dry_run -- --nocapture
```

通过标准：dry-run 读取旧 envelope 并报告项目/session 数量，但不创建 split store 目标文件。

证据路径：`docs/verification/persistence-incremental-state/fc-03-dry-run.log`

## FC-04: apply migration 保留 backup 并可加载 split store

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-engine split_store_migration -- --nocapture
```

通过标准：apply 迁移写入 `projects.json` 和 `sessions/*.json`，保留 backup，split store 可加载同等数量 records。

证据路径：`docs/verification/persistence-incremental-state/fc-04-apply-migration.log`

## FC-05: 单 session corruption quarantine

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-engine split_store_quarantine -- --nocapture
```

通过标准：一个坏 session 文件被 rename 为 `.corrupt`，其他 session 正常加载。

证据路径：`docs/verification/persistence-incremental-state/fc-05-quarantine.log`

## FC-06: 静态门禁和范围守卫

```bash
bash -n scripts/*.sh homie/scripts/*.sh
cargo fmt --manifest-path homie/Cargo.toml --all -- --check
git diff --check
git diff --name-only -- homie/crates/homie-engine/src/registry.rs docs/verification/persistence-incremental-state openspec/changes/persistence-incremental-state
```

证据路径：`docs/verification/persistence-incremental-state/fc-06-static-gates.log`
