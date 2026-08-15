# GPUI Large Module Test Boundaries Code Review Round 2

## 1. Scope

Second-pass review focused on hidden overlap, render and testing risks.

## 2. Hidden Risk Review

| Risk | Result | Evidence / Handling |
|---|---|---|
| Helper extraction could create a private API that cannot be reused | acceptable: helpers are `pub(super)` inside sidebar, enough for first-slice boundary without broadening public API |
| Tests might still require GPUI runtime | pass: tests live in `picker_logic.rs` and are ordinary Rust tests |
| Render path could still own too much decision logic | partially improved: target/shortcut/repo decision helpers moved; remaining host click effects stay in render by design |
| New module could be confused with completed shortcut-rank work | pass: module name and tests are picker-specific; shortcut-rank remains untouched |
| Visual behavior could change | low risk: call sites use the same helper outputs; no element tree or style code changed |

## 3. Not Changed

- No `homie-ui` primitive changes.
- No RootView lifecycle changes.
- No UtilitySurfaces changes.
- No terminal or inspector changes.
- No GPUI visual tree restructuring.

## 4. Conclusion

No P0/P1 hidden risks remain for this first slice.
