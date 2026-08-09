# Release Readiness Report: Diri Remote Companion Config

```yaml
change_id: diri-remote-companion-config
beads: homie-0gh
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## 1. Source

- PRD: `prd-spec/features/diri-remote-companion-config/2026-08-08-diri-remote-companion-config-design.md`
- OpenSpec: `openspec/changes/diri-remote-companion-config/`
- Functional cases: `docs/verification/diri-remote-companion-config/functional-cases.md`
- Beads: `homie-0gh`

## 2. Delivered

- Diri-compatible remote companion config model.
- Owner-only save/remove/load.
- Explicit pairing URL helper.
- Redacted Debug output.
- Parity lock updated from `missing` to `partial` for `REM-002`.

## 3. Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Config tests | `cargo test -p homie-remote --test companion_config -- --nocapture` | pass |
| Build | `cargo check -p homie-remote` | pass |
| Lint | `cargo clippy -p homie-remote --all-targets -- -D warnings` | pass |

## 4. Remaining Work

- App Settings Remote tab must wire to this model.
- Tailscale discovery and remote listener startup are pending.
- Real companion pairing E2E is pending.
