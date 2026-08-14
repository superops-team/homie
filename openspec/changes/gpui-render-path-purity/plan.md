# GPUI Render Path Purity Plan

## Scope

Extract Sidebar shortcut rank derivation from render into a pure helper and
cover it with tests.

## Out Of Scope

- Moving `sidebar_projection()` out of render.
- Moving glyph lifecycle out of render.
- Sidebar file split.
