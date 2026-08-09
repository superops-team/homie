# Diri UI Screenshot Verification Report

```yaml
change_id: diri-ui-screenshot
status: pass
validated_at: 2026-08-07
artifact: docs/verification/diri-ui-screenshot/homie-window-2026-08-07.png
```

## Summary

`homie-app` was launched from the current worktree with `cargo run -p homie-app`, activated as a foreground macOS window, and a real macOS screenshot was captured with `screencapture`.

## Evidence

| Gate | Result |
|------|--------|
| App launch | `cargo run -p homie-app` built and launched `target/debug/homie-app` |
| First screenshot capture | `docs/verification/diri-ui-screenshot/homie-workbench-2026-08-07.png` captured a black screen and is not accepted as visual evidence |
| Accepted screenshot capture | `screencapture -x docs/verification/diri-ui-screenshot/homie-workbench-2026-08-07-light.png` |
| Accepted window capture | `screencapture -x -l 24016 docs/verification/diri-ui-screenshot/homie-window-2026-08-07.png` |
| Image format | PNG, 3456 x 2234, RGBA |
| File size | 2041501 bytes |
| PNG decode smoke | zlib IDAT decode succeeded, raw image payload present, nonzero raw bytes = 2834594 |
| Window image format | PNG, 2336 x 1602, RGBA |
| Window file size | 534141 bytes |
| Window PNG decode smoke | zlib IDAT decode succeeded, raw image payload present, nonzero raw bytes = 546361 |
| Automated gate | `make ui-screenshot-gate` validates Homie and Diri PNGs, rejects blank captures, and checks this report records the black-screen rejection |
| Structural comparison gate | `make ui-screenshot-gate` decodes Homie and Diri PNG scanlines, computes left/center/right luma, verifies sidebar/workbench/inspector contrast, and requires vertical edge peaks for workbench separators |
| Current screenshot refresh attempt | `white-screen-diagnosis-2026-08-07.md` records rejected debug/package screenshots caused by local macOS capture/frontmost-window issues; these files are not accepted as parity evidence |
| Cleanup | launched app session was interrupted and no `target/debug/homie-app` process remained |

## Structural Metrics

The screenshot gate is intentionally structural rather than pixel-perfect. Homie and Diri can be captured at different window sizes, but both must show the Diri-style workbench skeleton: left sidebar, center terminal/workbench, and right inspector.

| Screenshot | Size | left/center/right luma | Luma gaps | vertical edge peaks |
|------------|------|------------------------|-----------|---------------------|
| Homie window | 2336 x 1602 | 183.21 / 65.94 / 148.79 | left-center 117.28, right-center 82.86 | 0.269, 0.711, 0.950 |
| Diri reference | 3456 x 2234 | 166.21 / 65.83 / 224.21 | left-center 100.37, right-center 158.38 | 0.745, 0.144, 0.425, 0.575 |

This gate prevents a blank capture or single-pane/static page from being accepted as UI evidence. It does not prove full interaction parity, visual polish, or pixel-level equality.

## Parity Impact

This is visual evidence for the current Diri-style workbench shell. It shows the live Homie window with sidebar, terminal pane, inspector, artifacts and notifications sections. The Diri reference screenshot `diri-reference-2026-08-07.png` is also stored in this directory and shows the target light right inspector, Info/Changes/Artifacts cards, permission modal, and toolbar structure.

Follow-up implementation after the comparison changed Homie’s right inspector to use a Diri-style light panel variant with light Info/Changes/Artifacts tabs and card-like rows, changed the left sidebar to a Diri-style light panel treatment, and added a visible permission modal projection for needs-input sessions. It does not by itself close UI parity rows because automated Diri side-by-side screenshot assertions and interaction automation remain pending.

The 2026-08-07 screenshot refresh after sidebar visible interactions did not produce valid evidence. The diagnosis report rejects those captures and keeps the accepted `homie-window-2026-08-07.png` / `diri-reference-2026-08-07.png` pair as the current evidence baseline.

