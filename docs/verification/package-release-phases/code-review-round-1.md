# Package/Release Phases Code Review Round 1

## 1. Scope

Reviewed files:

- `homie/scripts/package.sh`
- `.github/workflows/ci.yml`
- `homie/PACKAGING.md`
- `docs/verification/package-release-phases/*`
- `openspec/changes/package-release-phases/*`

## 2. Findings

| Severity | Finding | Result |
|---|---|---|
| P0 | `--phase verify` might trigger toolchain setup or build side effects | pass: verify returns before toolchain initialization and only runs read-only checks |
| P0 | `--phase preflight` might start long builds before reporting missing targets | pass: FC-04 confirms no build markers or `cargo packager` appear before failure |
| P1 | CI could keep a second copy of bundle verification logic | pass: CI bundle job now calls `homie/scripts/package.sh --phase verify --app "${app}"` |
| P1 | Default package behavior could be bypassed by phase parser | pass: default `phase="package"` falls through to the original full package sequence after preflight |
| P1 | Verify could incorrectly require release-only remote helpers for dev bundles | pass: release bundles still require remote helpers; non-release/dev bundles skip that check intentionally |
| P2 | Help output might be unavailable without toolchain setup | pass: `--help` exits before preflight/build and documents phase usage |

## 3. Verification Reviewed

| Evidence | Result |
|---|---|
| `fc-03-help-syntax.log` | `bash -n` and help output passed |
| `fc-04-preflight.log` | missing targets reported before build |
| `fc-05-verify-existing-app.log` | verify passed and mtime hash did not change |
| `fc-06-verify-failure-readonly.log` | broken app failed verify and remained on disk |
| `fc-07-default-ci-gates.log` | CI reuse and static gates passed |

## 4. Conclusion

No P0/P1 code issues found in round 1. Continue to hidden-risk review.
