# OpenSpec Plan: Diri Host Prefs Sync

> Change ID: `diri-host-prefs-sync`  
> Beads: `homie-cue`

## Scope

Add Homie remote prefs sync planning models matching Diri's fixed include-list behavior. This slice generates safe sync plans and command argv only; it does not execute remote commands.

## Modules

| Module | Change |
|--------|--------|
| `crates/homie-remote/src/lib.rs` | Add prefs sync tool specs, present-item discovery, mkdir/rsync argv, failure mapping |
| `crates/homie-remote/tests/prefs_sync.rs` | Diri-equivalent include/exclude/argv/error tests |

## Functional Cases

| Case | Command |
|------|---------|
| FC-DHPS-001 | `cargo test -p homie-remote --test prefs_sync -- --nocapture` |
| FC-DHPS-002 | `cargo check -p homie-remote` |
| FC-DHPS-003 | `cargo clippy -p homie-remote --all-targets -- -D warnings` |
| FC-DHPS-004 | scoped `git diff --check`; `make parity-lock` |

