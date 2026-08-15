# Package/Release Phases Tasks

## T1: Record dependency analysis and reviewed scope

- Deliverables:
  - `docs/verification/package-release-phases/dependency-priority-analysis.md`
  - `docs/verification/package-release-phases/functional-cases.md`
- Acceptance:
  - six open PRDs are ordered by dependency, not nominal priority;
  - first slice is limited to preflight and read-only verify.
- Verification Cases: FC-01

## T2: Add phase parser and help

- Deliverables:
  - `homie/scripts/package.sh`
- Acceptance:
  - `package.sh --help` documents default behavior, `--phase preflight`, `--phase verify`, and `--app <path>`;
  - unknown phases and missing `--app` produce non-zero errors before build;
  - `bash -n homie/scripts/package.sh` passes.
- Verification Cases: FC-03

## T3: Implement preflight phase

- Deliverables:
  - `homie/scripts/package.sh`
- Acceptance:
  - checks required tools before long builds: `cargo`, `rustup`, `cargo-packager`, `swift`, `lipo`, `codesign`, `plutil`;
  - checks release Rust targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`;
  - reports all missing checks found in one run where practical;
  - does not run `cargo build`, `cargo packager`, signing, notary, copying, or deletion.
- Verification Cases: FC-04

## T4: Implement read-only verify phase

- Deliverables:
  - `homie/scripts/package.sh`
- Acceptance:
  - validates an existing app passed through `--app`;
  - checks plist, core binaries, manifests, optional/full remote helper catalog, and codesign;
  - returns non-zero for missing resources;
  - does not mutate bundle files.
- Verification Cases: FC-05, FC-06

## T5: Preserve default full package path

- Deliverables:
  - `homie/scripts/package.sh`
- Acceptance:
  - no-argument invocation still runs the full existing package flow;
  - full package flow invokes preflight once before building;
  - signing, notary, DMG, zip, remote helper catalog, and manifest copy semantics remain unchanged.
- Verification Cases: FC-07

## T6: Reuse verify phase in CI and docs

- Deliverables:
  - `.github/workflows/ci.yml`
  - `homie/PACKAGING.md`
- Acceptance:
  - CI bundle job calls `homie/scripts/package.sh --phase verify --app <path>`;
  - docs explain phase usage and read-only verify semantics.
- Verification Cases: FC-07

## T7: Final static and alignment gates

- Deliverables:
  - `openspec/changes/package-release-phases/alignment-report.md`
  - verification reports under `docs/verification/package-release-phases/`
- Acceptance:
  - OpenSpec tasks map to FC-01 through FC-07;
  - `git diff --check` passes;
  - any skipped full package run records exact preflight blockers.
- Verification Cases: FC-02, FC-07
