# Diri Settings Surface Tasks

> Change ID: `diri-settings-surface`  
> Beads: `homie-s0w`

## Task Mapping

| Task | Scope | Functional cases | Beads |
|------|-------|------------------|-------|
| T-001 | Storage preferences API | FC-DSS-001 | `homie-s0w` |
| T-002 | App settings surface and command wiring | FC-DSS-002 | `homie-s0w` |
| T-003 | Evidence, clippy, parity lock, LoopX writeback | FC-DSS-003, FC-DSS-004 | `homie-s0w` |

## Tasks

### T-001: Storage preferences API

- Add persisted preference read/write helpers.
- Add typed settings preferences defaults and roundtrip tests.

### T-002: App settings surface

- Add settings UI state and tabs.
- Wire `PaletteCommand::OpenSettings` to `open_settings`.
- Render General/Terminal/Resources/Remote preference rows.

### T-003: Evidence and lock update

- Update parity lock `UI-006` from `missing` to `partial` with evidence.
- Keep `UI-006` partial until screenshot/interaction E2E exists.

