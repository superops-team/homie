# Full Dev Bundle Smoke Code Review Round 2

## 1. Scope

Second-pass review focused on hidden lifecycle, environment and packaging risks.

## 2. Hidden Risk Review

| Risk | Result | Evidence / Handling |
|---|---|---|
| Temporary Engine might remain alive after smoke | pass: `run_full_smoke` installs a `RETURN` trap that kills and waits for the child process, then removes temp app support |
| Smoke might inherit agent/session env | pass: Engine and CLI smoke unset `HOMIE_SOCKET`, `HOMIE_SESSION_ID`, `HOMIE_CLI`, and `NO_COLOR` where appropriate; CLI gets only the temporary socket |
| `--smoke` might run without full bundle resources | pass: parser sets `full_bundle=1` and `no_launch=1` when `--smoke` is passed |
| Swift CLI configuration might mismatch Cargo profile | pass: debug uses Swift debug, `--release` maps to Swift release |
| Full dev bundle might include stale manifests | pass: full mode removes and recopies `Contents/Resources/bin/manifests`; package verify checks source count |
| Smoke might touch real state through `doctor` | fixed before final report: smoke now uses `status`, which only requires daemon socket round-trip |
| Default quick dev path might be blocked by full-mode validation | pass: package verify and smoke are guarded by `full_bundle` / `smoke`; default path unchanged |

## 3. Not Changed

- No CI full-dev smoke job was added.
- No sidecar, remote helper, DMG, zip, notary or universal dev bundle was added.
- No Rust app or Engine source code changed.

## 4. Conclusion

No P0/P1 hidden risks remain for this first slice.
