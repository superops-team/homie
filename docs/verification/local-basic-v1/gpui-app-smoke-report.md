# GPUI App Smoke Report

```yaml
change_id: local-basic-v1
report_type: gpui-app-smoke
status: pass
beads: homie-54y
```

## Scope

This report verifies that `Homie.app` is a real GPUI application bundle, not a shell-script dialog wrapper.

## Evidence

| Check | Result |
|-------|--------|
| `cargo check -p homie-app` | pass |
| `make full-check` | pass |
| `make dmg` | pass |
| `file Homie.app/Contents/MacOS/Homie` | Mach-O 64-bit executable arm64 |
| `codesign --verify --deep --strict Homie.app` | pass |
| DMG `bin/homie doctor --data-dir <tmp> --json` | pass |
| `open -n <mounted>/Homie.app` then process check | GPUI app process stayed running |

## Artifact Paths

```text
APP_PATH=/Users/bytedance/workspace/github/homie/dist/homie-0.1.0-aarch64-apple-darwin/Homie.app
DMG_PATH=/Users/bytedance/workspace/github/homie/dist/homie-0.1.0-aarch64-apple-darwin.dmg
```

## Gate Decision

Decision: pass

Reason:

- `Homie.app` now launches a GPUI window process.
- CLI remains packaged separately under `Contents/Resources/bin/homie` and top-level `bin/homie`.
