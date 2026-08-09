# OpenSpec Plan: Diri PR Monitor Parser

> Change ID: `diri-pr-monitor-parser`  
> Beads: `homie-jkj`

## Scope

Add first-stage PR monitor parsing models to `homie-runtime`. This brings `ART-003` from no implementation to parser/model foundation while avoiding network calls or UI claims.

## Modules

| Module | Change |
|--------|--------|
| `crates/homie-runtime/src/pr_monitor.rs` | New parser/model/rollup module |
| `crates/homie-runtime/src/lib.rs` | Export module |
| `crates/homie-runtime/tests/pr_monitor.rs` | Diri-equivalent parser fixtures |

## Functional Cases

| Case | Command |
|------|---------|
| FC-DPRM-001 | `cargo test -p homie-runtime --test pr_monitor -- --nocapture` |
| FC-DPRM-002 | `cargo check -p homie-runtime` |
| FC-DPRM-003 | `cargo clippy -p homie-runtime --all-targets -- -D warnings` |
| FC-DPRM-004 | scoped `git diff --check`; `make parity-lock` |

