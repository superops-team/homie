# Code Review Round 1: Diri Storage/Indexing Parity Phase 1

```yaml
change_id: diri-storage-indexing
beads: homie-q7n
round: 1
status: pass_after_fix
```

## 1. 审查范围

- `crates/homie-storage/src/lib.rs`
- `crates/homie-storage/tests/diri_storage_indexing.rs`
- `crates/homie-storage/tests/storage_bootstrap.rs`
- `specs/storage-indexing/README.md`
- PRD/OpenSpec/evidence files under `diri-storage-indexing`

## 2. Findings

| 严重度 | 类别 | 位置 | 证据与影响 | 处置 |
|---|---|---|---|---|
| medium | Correctness | `crates/homie-storage/src/lib.rs` `query_usage_totals` | Empty usage queries made `MAX(CASE...)` return NULL. Reading it as `i64` would fail instead of returning zero totals. | Fixed with `COALESCE(MAX(...), 0)` and added `usage_repository_handles_empty_queries_and_rejects_negative_tokens`. |
| medium | Correctness | `crates/homie-storage/src/lib.rs` `record_usage` | Diri ledger rejects negative token/cost values, but storage accepted negative usage values before insert. This could corrupt totals. | Added `validate_usage`, non-negative decimal validation and regression test. |

## 3. 修复摘要

- Added `StorageError::InvalidInput`.
- Added negative usage/token/cost validation before insert.
- Added empty-query behavior to return zero totals.
- Added a focused regression test covering both issues.

## 4. 验证结果

| Command | Result |
|---------|--------|
| `cargo fmt -p homie-storage -- --check` | pass |
| `cargo check -p homie-storage` | pass |
| `cargo clippy -p homie-storage --all-targets -- -D warnings` | pass |
| `cargo test -p homie-storage` | pass |

## 5. 剩余风险

- No P0/P1 issues remain after round 1.
- This round did not inspect UI/runtime behavior because those are out of scope for this lane.
