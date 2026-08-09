# OpenSpec Plan: Diri Remote Companion Config

> Change ID: `diri-remote-companion-config`  
> Beads: `homie-0gh`

## Scope

Add the first remote companion config model to `homie-remote`, matching Diri's remote config file semantics without starting a listener or discovering Tailscale.

## Modules

| Module | Change |
|--------|--------|
| `crates/homie-remote/src/lib.rs` | Add `RemoteCompanionConfig` load/save/remove, endpoint/pairing helpers |
| `crates/homie-remote/tests/companion_config.rs` | Config roundtrip, owner-only mode, token redaction tests |

## Functional Cases

| Case | Command |
|------|---------|
| FC-DRCC-001 | `cargo test -p homie-remote --test companion_config -- --nocapture` |
| FC-DRCC-002 | `cargo check -p homie-remote` |
| FC-DRCC-003 | `cargo clippy -p homie-remote --all-targets -- -D warnings` |
| FC-DRCC-004 | scoped `git diff --check`; `make parity-lock` |

