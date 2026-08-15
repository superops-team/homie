# Full Dev Bundle Smoke Release Readiness Report

## 1. Conclusion

`full-dev-bundle-smoke` first slice is ready to land.

## 2. Delivered

- `homie/scripts/dev.sh --full`
- `homie/scripts/dev.sh --full --no-launch`
- `homie/scripts/dev.sh --full --no-launch --smoke`
- Full dev app bundle with GUI, Engine, holder, askpass, MCP proxy, Swift CLI and Rust-owned manifests.
- Reuse of `package.sh --phase verify --app <path>`.
- Temporary Engine smoke using a temporary app support directory and socket.

## 3. Verification

| Gate | Result | Evidence |
|---|---|---|
| Spec review | pass | `spec-review-report.md` |
| OpenSpec alignment | pass | `fc-02-openspec-alignment.log` |
| Script syntax/help | pass | `fc-03-help-syntax.log` |
| Full dev bundle build and verify | pass | `fc-04-full-dev-bundle.log` |
| Temporary Engine smoke | pass | `fc-05-engine-smoke.log` |
| Static gates | pass | `fc-06-static-gates.log` |
| Code review round 1 | pass | `code-review-round-1.md` |
| Code review round 2 | pass | `code-review-round-2.md` |
| Shell syntax | pass | `bash -n scripts/*.sh homie/scripts/*.sh` |
| Diff whitespace | pass | `git diff --check` |

## 4. Not Run

- CI full-dev smoke was not added or run. This is intentionally out of scope for the first slice.
- Release package universal/notary/DMG/update zip paths were not run for this change; they are covered by `package-release-phases`.

## 5. Residual Risk

- Full dev bundle does not include remote helper catalog or sidecar in this slice.
- The smoke uses `homie status` rather than `homie doctor` because Swift `HomiePaths.stateFile` resolves the real user Application Support path independently of `HOME`. `status` is the intended socket round-trip check for the temporary Engine.
