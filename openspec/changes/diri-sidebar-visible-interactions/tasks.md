# OpenSpec Tasks: Diri Sidebar Visible Interactions

| Task | Description | Acceptance | Cases |
|------|-------------|------------|-------|
| T-001 | Add app sidebar model state and sync helper | `refresh_sessions_from_client` updates `SidebarSessionModel` from runtime sessions while preserving local pin/archive/multi-select state | FC-DSVI-001, FC-DSVI-003 |
| T-002 | Add visible row controls | Session rows render pin, multi-select, and archive controls with mouse handlers | FC-DSVI-001, FC-DSVI-003 |
| T-003 | Preserve selection and archive semantics | Selecting a row still attaches runtime session; archiving only hides local sidebar row | FC-DSVI-001, FC-DSVI-002 |
| T-004 | Verify and record evidence | Functional report, code review, release readiness, parity lock honesty | FC-DSVI-001..005 |

## Notes

- This slice intentionally uses source regression plus compile/lint gates because full GPUI pointer E2E is still not available in this repo.
- Do not mark `UI-002` implemented.
