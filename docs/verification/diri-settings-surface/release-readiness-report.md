# Diri Settings Surface Release Readiness

```yaml
change_id: diri-settings-surface
beads: homie-s0w
status: ready_for_next_loopx_slice
```

## Delivered

- Typed `SettingsPreferences` model.
- `preferences` JSON get/set API.
- `homie-app` settings surface with General/Terminal/Resources/Remote tabs.
- Real `OpenSettings` command wiring.
- Persistent terminal font size and remote companion access toggles.

## Parity Impact

| Row | Decision | Reason |
|-----|----------|--------|
| UI-006 | partial | Settings surface and persistence exist; screenshot/interaction E2E remains pending. |

## Verification

See `docs/verification/diri-settings-surface/functional-verification-report.md`.

