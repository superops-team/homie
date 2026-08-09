# Release Readiness Report: Diri Host Prefs Sync

```yaml
change_id: diri-host-prefs-sync
beads: homie-cue
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## 1. Source

- PRD: `prd-spec/features/diri-host-prefs-sync/2026-08-08-diri-host-prefs-sync-design.md`
- OpenSpec: `openspec/changes/diri-host-prefs-sync/`
- Functional cases: `docs/verification/diri-host-prefs-sync/functional-cases.md`
- Beads: `homie-cue`

## 2. Delivered

- Secretless prefs sync include list for Claude and Codex.
- Present-item discovery.
- mkdir/rsync argv generation.
- Missing remote rsync error mapping.
- Parity lock updated from `missing` to `partial` for `REM-003`.

## 3. Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Prefs sync tests | `cargo test -p homie-remote --test prefs_sync -- --nocapture` | pass |
| Build | `cargo check -p homie-remote` | pass |
| Lint | `cargo clippy -p homie-remote --all-targets -- -D warnings` | pass |

## 4. Remaining Work

- `host.locate_repo` model and tests.
- Real ssh/rsync integration smoke.
- Remote settings UI and companion access.
