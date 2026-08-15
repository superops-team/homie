# Package/Release Phases Plan

## 1. Scope

First slice for `package-release-phases`.

## 2. In Scope

- Keep default `homie/scripts/package.sh` behavior as the full release package path.
- Add command-line usage and phase parsing.
- Add `--phase preflight`.
- Add read-only `--phase verify --app <path>`.
- Move the existing CI bundle structure checks into the package verify phase.
- Update package documentation for the new phases.
- Record evidence under `docs/verification/package-release-phases/`.

## 3. Out Of Scope

- `--local-arm64` implementation.
- `--skip-build` implementation.
- Notarization/signing policy changes.
- DMG/update zip artifact format changes.
- New package runtime such as Bun, TypeScript, or Python orchestration.
- Full dev bundle implementation. That remains `full-dev-bundle-smoke` and should reuse this verify phase later.

## 4. Dependency Analysis

See `docs/verification/package-release-phases/dependency-priority-analysis.md`.

Execution order across the six open PRDs is dependency-driven, not priority-number-driven:

1. `package-release-phases`
2. `full-dev-bundle-smoke`
3. `protocol-contract-golden-fixtures`
4. `typed-agent-driver-capabilities`
5. `persistence-incremental-state`
6. `gpui-large-module-test-boundaries`

## 5. Design

`package.sh` will expose a small phase dispatcher:

```text
package.sh
package.sh --help
package.sh --phase preflight
package.sh --phase verify --app <path>
```

Default no-argument execution remains the full package flow.

`preflight` validates required tools and Rust targets before any long build. It may fail on the current machine if targets are missing, but the failure must happen before the first `cargo build`.

`verify` validates an existing app bundle and must be read-only. It checks:

- `Contents/Info.plist` parses;
- core bundled binaries exist and are executable;
- bundled Rust Engine binaries have expected universal arches when they are Mach-O universal release artifacts;
- Rust-owned agent manifest catalog exists and has the same JSON count as source;
- remote helper manifest and three helper artifacts exist when the app is a full release bundle;
- `codesign --verify --deep --strict` passes when codesign can verify the app.

`verify` must not sign, copy, delete, notarize, rebuild, or mutate the bundle.

## 6. Evidence

- Spec review: `docs/verification/package-release-phases/spec-review-report.md`
- Functional cases: `docs/verification/package-release-phases/functional-cases.md`
- Dependency analysis: `docs/verification/package-release-phases/dependency-priority-analysis.md`
- Functional verification report: `docs/verification/package-release-phases/functional-verification-report.md`
- Code review: `docs/verification/package-release-phases/code-review-report.md`
- Release readiness: `docs/verification/package-release-phases/release-readiness-report.md`

## 7. Risks

| Risk | Control |
|---|---|
| Default package behavior changes | Keep full flow as default and verify script syntax/static gates |
| Verify mutates bundle | FC-05 compares file mtimes before and after verify |
| Preflight accidentally starts build | FC-04 asserts no build markers appear |
| CI maintains second verification logic | CI calls package verify phase |
| Full dev bundle duplicates package checks | This change lands verify first so full dev smoke can reuse it |
