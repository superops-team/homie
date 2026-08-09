# Diri Terminal Find Surface Release Readiness

```yaml
change_id: diri-terminal-find-surface
beads: homie-42v
status: ready_for_next_loopx_slice
```

## Delivered

- App-visible terminal Find surface.
- Command palette `OpenFind` entry.
- `TerminalFindModel` backed query and highlight sync.

## Parity Impact

| Row | Decision | Reason |
|-----|----------|--------|
| UI-003 | partial | Find is visible and model-backed; full GPUI terminal interaction E2E remains pending. |
| TERM-004 | partial | Find/key model tests and app wiring exist; real PTY interaction E2E remains pending. |

