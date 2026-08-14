# GPUI UtilitySurfaces First Slice Release Readiness Report

## 1. Conclusion

`gpui-utility-surfaces-first-slice` is ready to land. The change implements the
first code-bearing GPUI lifecycle hardening slice for UtilitySurfaces History
and Worktrees flows.

## 2. Delivered

- Added child PRD/spec and spec review evidence.
- Added functional verification cases and OpenSpec plan/tasks/alignment.
- Fixed clean-worktree icon asset baseline by narrowing `.gitignore` from
  `Icon?` to `/Icon?` and tracking existing `homie-ui` SVG icons.
- Added `history_generation`, `history_task`, `worktrees_generation`, and
  `worktrees_task` to `UtilitySurfaces`.
- Converted History load/resume and Worktrees refresh/cleanup to held lifecycle
  tasks.
- Added generation and surface guards before applying late async results.
- Cleared lifecycle tasks and advanced generation when closing History or
  Worktrees.
- Added focused tests for stale History, stale Worktrees, and closed surface
  late results.

## 3. Verification

| Gate | Result |
|------|--------|
| FC-01 through FC-09 | pass |
| `cargo test --manifest-path homie/Cargo.toml -p homie-app surface_shell::tests -- --nocapture` | pass, 16 tests |
| `(cd homie && cargo fmt --check)` | pass |
| `git diff --check` | pass |
| Scope guard for `homie-ui/src`, `homie-engine`, `homie-client`, `root.rs` | pass, no output |
| SVG icon count | pass, 46 files |

## 4. Not Run

- Full workspace Rust test suite was not run; this slice is limited to
  `surface_shell` lifecycle behavior plus tracked icon assets. Targeted
  `homie-app surface_shell::tests` passed.
- Real app visual launch was not run; UI layout and visible copy are unchanged.

## 5. Residual Risk

Dropping a held GPUI task does not guarantee daemon-side cancellation after a
request has been sent. The implemented generation and surface guards prevent
late results from rewriting visible UI state, which is the intended scope of
this slice.
