# Full Dev Bundle Smoke Plan

## 1. Scope

First slice for `full-dev-bundle-smoke`.

## 2. In Scope

- Extend `homie/scripts/dev.sh` with:
  - `--full`;
  - `--no-launch`;
  - `--smoke`.
- Build a local-architecture full dev app bundle.
- Bundle:
  - Rust GUI `homie`;
  - Rust Engine `homied-rs`;
  - Rust holder `homie-holder`;
  - Rust askpass `homie-ssh-askpass`;
  - Rust MCP proxy `homie-mcp`;
  - Swift CLI `homie`;
  - Rust-owned agent manifests.
- Reuse `homie/scripts/package.sh --phase verify --app <app>` for bundle checks.
- Smoke a temporary Engine using `HOMIE_APP_SUPPORT` and `HOMIE_SOCKET`.

## 3. Out Of Scope

- Universal dev bundle.
- Notarization.
- DMG/update zip.
- Three-platform remote helper catalog.
- Sidecar/browser packaging.
- Hot reload/watch.
- CI full-dev smoke. This can be added after local smoke stabilizes.

## 4. Dependency

Depends on `package-release-phases` first slice:

- `package.sh --phase verify --app <path>` is the shared bundle verification surface.
- Full dev smoke must not duplicate release bundle structure checks.

## 5. Design

`dev.sh --full --no-launch --smoke` will:

1. Build Rust binaries for the current host/profile.
2. Build Swift CLI in matching debug/release configuration.
3. Assemble a commit-specific `.app` under `homie/target`.
4. Copy core binaries into `Contents/Resources/bin`.
5. Copy Rust-owned manifests into `Contents/Resources/bin/manifests`.
6. Sign nested binaries and app ad-hoc.
7. Run `package.sh --phase verify --app <app>`.
8. Start bundled `homied-rs` with temporary `HOMIE_APP_SUPPORT`.
9. Use bundled Swift CLI with temporary `HOMIE_SOCKET` to run `doctor`.
10. Terminate the temporary Engine and clean temporary app support.

Default `dev.sh` behavior remains build-and-launch quick UI dev.

## 6. Evidence

- Spec review: `docs/verification/full-dev-bundle-smoke/spec-review-report.md`
- Functional cases: `docs/verification/full-dev-bundle-smoke/functional-cases.md`
- Functional verification: `docs/verification/full-dev-bundle-smoke/functional-verification-report.md`
- Code review: `docs/verification/full-dev-bundle-smoke/code-review-round-1.md`, `code-review-round-2.md`
- Release readiness: `docs/verification/full-dev-bundle-smoke/release-readiness-report.md`

## 7. Risks

| Risk | Control |
|---|---|
| Full dev path replaces fast UI dev | Default path remains unchanged; `--full` opt-in only |
| Smoke touches real user state | Use temp `HOMIE_APP_SUPPORT` and temp `HOMIE_SOCKET`; unset inherited agent/session env |
| Bundle checks drift from package | Call `package.sh --phase verify --app` |
| Full dev becomes release package clone | Do not build remote helpers, universal binary, DMG, zip or notary |
