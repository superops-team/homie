# Full Dev Bundle Smoke Code Review Round 1

## 1. Scope

Reviewed files:

- `homie/scripts/dev.sh`
- `docs/verification/full-dev-bundle-smoke/*`
- `openspec/changes/full-dev-bundle-smoke/*`

## 2. Findings

| Severity | Finding | Result |
|---|---|---|
| P0 | `--full --smoke` may read or write real user Application Support | fixed: smoke uses temporary `HOMIE_APP_SUPPORT`/`HOMIE_SOCKET`; after review it uses CLI `status` instead of `doctor` to avoid Swift `HomiePaths.stateFile` checking real state |
| P0 | Bash `set -u` may fail when no cargo args are passed | fixed: `cargo_build` helper handles empty `cargo_args` |
| P1 | Full dev mode may replace quick UI dev behavior | pass: default path still reaches `==> Launching` and `exec env ... Contents/MacOS/homie`; full mode is opt-in |
| P1 | Full dev may duplicate release package checks | pass: full mode calls `package.sh --phase verify --app <app>` |
| P1 | Full dev may become release package clone | pass: no universal build, remote helper catalog, notary, DMG or update zip is added |
| P2 | Nested binary signatures may be missing before app signing | pass: copied runtime binaries are ad-hoc signed before app signing |

## 3. Verification Reviewed

| Evidence | Result |
|---|---|
| `fc-03-help-syntax.log` | parser/help/default launch marker present |
| `fc-04-full-dev-bundle.log` | build, bundle, package verify and smoke passed |
| `fc-05-engine-smoke.log` | socket round-trip passed and real app support path absent |
| `fc-06-static-gates.log` | shell syntax and diff checks passed |

## 4. Conclusion

Round 1 issues were fixed during implementation. No remaining P0/P1 issue found.
