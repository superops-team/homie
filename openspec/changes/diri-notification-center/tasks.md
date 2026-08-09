# Diri Notification Center Tasks

> Change ID: `diri-notification-center`  
> Beads: `homie-5pe`

## Task Mapping

| Task | Scope | Functional cases | Beads |
|------|-------|------------------|-------|
| T-001 | `homie-ui` notification model and tests | FC-DNC-001 | `homie-5pe` |
| T-002 | App notification rollup rendering | FC-DNC-002 | `homie-5pe` |
| T-003 | Evidence, clippy, parity lock, LoopX | FC-DNC-003, FC-DNC-004 | `homie-5pe` |

## Tasks

### T-001: Notification model

- Add notification severity, item, action, rollup and macOS command builder.
- Cover blocked/running/exited rollup and quick approve/deny descriptors.

### T-002: App rollup

- Render notification rollup in inspector.
- Add regression test preventing notification surface from being absent.

### T-003: Evidence

- Update `UI-008` from `missing` to `partial`.
- Keep `UI-008` partial until native E2E exists.

