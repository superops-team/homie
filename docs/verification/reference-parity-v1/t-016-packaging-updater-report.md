# T-016 Packaging Updater Performance Report

```yaml
change_id: reference-parity-v1
openspec_task: T-016
beads: homie-wty
status: partial
functional_cases:
  - FC-017
  - FC-018
```

## 1. Summary

T-016 implemented the first packaging/updater trust contract slice.

Implemented:

- `homie-updater` crate.
- update candidate trust decision for host, bundle id, Team ID, promised version, codesign, system policy and newer-version checks.
- `scripts/package/perf-gate.sh` structural packaged-app gate.
- local package build verified with ad-hoc signing.

This is a partial T-016 pass because real Developer ID signing, notarization, update feed publication, old-to-new self-update and GUI performance measurement require release infrastructure or an interactive release host.

## 2. RED

Added updater trust tests:

- `crates/homie-updater/tests/trust.rs`

The tests require:

- matching identity and newer version succeeds.
- wrong Team ID fails.
- wrong bundle id fails.
- failed codesign fails.
- failed system policy assessment fails.
- non-newer candidate fails.

## 3. GREEN

Implemented:

- `crates/homie-updater/Cargo.toml`
- `crates/homie-updater/src/lib.rs`
- `scripts/package/perf-gate.sh`
- workspace registration in `Cargo.toml`

## 4. Verification

Focused command:

```bash
cargo test -p homie-updater
```

Result:

- Exit code: 0
- Updater trust tests: 3 passed
- Doc tests: 0 tests

Packaging command:

```bash
HOMIE_DIST_DIR=/private/tmp/homie-reference-parity-dist scripts/package/package.sh
```

Result:

- Exit code: 0
- App bundle: `/private/tmp/homie-reference-parity-dist/homie-0.1.0-aarch64-apple-darwin/Homie.app`
- Tarball: `/private/tmp/homie-reference-parity-dist/homie-0.1.0-aarch64-apple-darwin.tar.gz`
- Signature: ad-hoc local signing

Packaged perf-gate structural command:

```bash
scripts/package/perf-gate.sh --app /private/tmp/homie-reference-parity-dist/homie-0.1.0-aarch64-apple-darwin/Homie.app --scenario all
```

Result:

- Exit code: 0
- `PERF_GATE=not_run`
- Reason: packaged GUI measurement requires an interactive macOS release host.

Workspace regression command:

```bash
cargo test --workspace
```

Result:

- Exit code: 0
- Homie agents/app/CLI/context/LLM/memory/orchestrator/proto/remote/runtime/storage/task/term/UI/updater tests passed.

Safety checks:

```bash
rg -n -i "<old-reference-name-pattern>" .
git diff --check
```

Result:

- Old reference name scan: no matches.
- Markdown/patch whitespace check: pass.

## 5. Not Run

| Gate | Status | Reason |
|------|--------|--------|
| Developer ID signing | not_run | No Developer ID signing configuration was provided in this run |
| Notarization/stapling | not_run | Requires Developer ID and notarytool credentials |
| DMG notarization | not_run | Requires release signing environment |
| Old-to-new self-update | not_run | Requires published update feed and prior signed build |
| Physical memory/CPU GUI perf | not_run | Requires interactive packaged app measurement host |

## 6. Remaining Scope

T-016 remains partially complete until real release infrastructure is available. The code-level updater trust and local package structure gates are complete.

