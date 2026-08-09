# Code Review Round 2: Diri Storage/Indexing Parity Phase 1

```yaml
change_id: diri-storage-indexing
beads: homie-q7n
round: 2
status: pass_after_fix
```

## 1. 审查范围

Second-pass adversarial review of:

- v1/v2 -> v3 migration path.
- usage repository validation and aggregate behavior.
- history/project/worktree/session repository boundaries.
- scope compliance against lane write constraints.

## 2. Findings

| 严重度 | 类别 | 位置 | 证据与影响 | 处置 |
|---|---|---|---|---|
| low | Correctness | `crates/homie-storage/src/lib.rs` `record_usage` | `source_event_id` and other identity fields could be blank. That weakens Diri-style source event dedupe and makes later usage parser evidence hard to audit. | Added required-field validation for usage identity fields and regression assertion for empty `source_event_id`. |

## 3. 对抗式复盘

- Migration from an empty DB applies v1, v2 and v3 inside one transaction and records all versions.
- Existing v2 `history_entries` rows are copied into the v3 table with `external_id = id`, preserving old rows while allowing the new `(agent_kind, external_id)` contract.
- `usage_records` old rows get `source_event_id = request_id` before the unique index is created.
- Repository tests exercise public APIs; SQLite metadata assertions are limited to schema inventory and constraints.
- No UI/runtime/git/parser behavior was added.

## 4. 验证结果

| Command | Result |
|---------|--------|
| `cargo fmt -p homie-storage -- --check` | pass |
| `cargo check -p homie-storage` | pass |
| `cargo clippy -p homie-storage --all-targets -- -D warnings` | pass |
| `cargo test -p homie-storage` | pass |

## 5. 剩余风险

- No P0/P1 issues remain.
- Usage pricing remains stored as decimal strings in SQLite, matching existing schema style; precise money arithmetic belongs to the LLM/usage lane.
