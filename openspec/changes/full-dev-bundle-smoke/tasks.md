# Full Dev Bundle Smoke Tasks

## T1: Spec and functional cases

- Deliverables:
  - `docs/verification/full-dev-bundle-smoke/functional-cases.md`
  - `openspec/changes/full-dev-bundle-smoke/*`
- Acceptance:
  - PRD/spec, review report and OpenSpec agree on first-slice scope;
  - every task maps to a functional case.
- Verification Cases: FC-01, FC-02

## T2: Add full/no-launch/smoke options to dev.sh

- Deliverables:
  - `homie/scripts/dev.sh`
- Acceptance:
  - help output includes `--full`, `--no-launch`, `--smoke`;
  - default quick UI dev behavior remains the launch path;
  - `--smoke` requires `--full`.
- Verification Cases: FC-03

## T3: Build local full dev runtime binaries

- Deliverables:
  - `homie/scripts/dev.sh`
- Acceptance:
  - builds GUI, Engine, holder, askpass and MCP in the selected profile;
  - builds Swift CLI in debug or release matching `--release`;
  - does not build universal binaries or remote helpers.
- Verification Cases: FC-04

## T4: Assemble and sign full dev app bundle

- Deliverables:
  - `homie/scripts/dev.sh`
- Acceptance:
  - copies all core binaries into expected bundle paths;
  - copies Rust-owned manifests into `Contents/Resources/bin/manifests`;
  - ad-hoc signs nested binaries and app;
  - calls `package.sh --phase verify --app <app>`.
- Verification Cases: FC-04

## T5: Smoke temporary bundled Engine

- Deliverables:
  - `homie/scripts/dev.sh`
- Acceptance:
  - uses temporary `HOMIE_APP_SUPPORT`;
  - uses temporary `HOMIE_SOCKET`;
  - runs bundled Swift CLI `doctor`;
  - terminates temporary Engine and cleans temp state.
- Verification Cases: FC-05

## T6: Documentation and static gates

- Deliverables:
  - `homie/README.md`
  - verification reports
- Acceptance:
  - docs explain quick dev vs full dev smoke;
  - shell syntax and diff whitespace gates pass.
- Verification Cases: FC-06
