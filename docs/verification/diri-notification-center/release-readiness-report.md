# Diri Notification Center Release Readiness

```yaml
change_id: diri-notification-center
beads: homie-5pe
status: ready_for_next_loopx_slice
```

## Delivered

- Notification center model in `homie-ui`.
- Status rollup badge.
- Quick approve/deny descriptors gated by known agent capability.
- Safe macOS notification command builder.
- App inspector notification rollup.

## Parity Impact

| Row | Decision | Reason |
|-----|----------|--------|
| UI-008 | partial | Notification center model and app rollup exist; native delivery/menu bar/sounds/quick-action E2E remains pending. |

## Verification

See `docs/verification/diri-notification-center/functional-verification-report.md`.

