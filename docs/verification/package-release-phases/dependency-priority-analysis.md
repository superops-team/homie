# Package/Release Phases Dependency And Priority Analysis

## 1. Scope

This analysis covers the six open PRD/spec changes from the Waku comparison review queue:

| Bead | change_id | Nominal priority | Area |
|---|---|---:|---|
| `homie-54o` | `protocol-contract-golden-fixtures` | P0 | Swift/Rust protocol drift |
| `homie-ceg` | `full-dev-bundle-smoke` | P0 | Dev bundle runtime verification |
| `homie-d5w` | `package-release-phases` | P1 | Package/release verification phases |
| `homie-kcq` | `typed-agent-driver-capabilities` | P1 | Agent runtime typed capabilities |
| `homie-or4` | `persistence-incremental-state` | P1 | Registry persistence evolution |
| `homie-wgv` | `gpui-large-module-test-boundaries` | P2 | GPUI testability boundaries |

The user requested treating P0/P1/P2 equally. The execution order therefore follows dependency and risk reduction, not priority number alone.

## 2. Dependency Graph

```text
package-release-phases
  -> full-dev-bundle-smoke

protocol-contract-golden-fixtures
  -> typed-agent-driver-capabilities (only if typed control expands wire methods)

persistence-incremental-state
  -> independent; high data-migration risk

gpui-large-module-test-boundaries
  -> independent; must avoid overlap with completed GPUI child slices
```

## 3. Ordering Rationale

| Order | change_id | Rationale |
|---:|---|---|
| 1 | `package-release-phases` | Provides read-only bundle `verify` and `preflight`, which should become the shared verification base for `full-dev-bundle-smoke`; it also gives immediate value by failing missing targets/tools before long builds. |
| 2 | `full-dev-bundle-smoke` | Depends on package verification concepts. After `verify_app` exists, full dev smoke can reuse the same bundle structure checks instead of creating a second source of truth. |
| 3 | `protocol-contract-golden-fixtures` | Establishes Swift/Rust fixture drift gate before future protocol-affecting changes. It is independent from packaging and can run next. |
| 4 | `typed-agent-driver-capabilities` | Should not expand wire methods before protocol fixture discipline exists. First slice may remain internal/fake-driver only, but the fixture gate lowers future protocol risk. |
| 5 | `persistence-incremental-state` | Independent but risky because it touches user data migration. It should start after the lower-risk verification foundations are complete so the dev-loop has stable evidence patterns. |
| 6 | `gpui-large-module-test-boundaries` | Independent and valuable, but mostly maintainability-oriented. It should still complete under equal treatment, but after runtime/package/data risks are under control. |

## 4. Current Dev-Loop Entry

Start with `package-release-phases`.

Entry criteria:

- PRD review completed: `docs/verification/package-release-phases/spec-review-report.md`.
- Bead claimed: `homie-d5w`.
- Scope limited to first slice:
  - keep default `homie/scripts/package.sh` behavior;
  - add `--phase preflight`;
  - add read-only `--phase verify --app <path>`;
  - allow CI bundle job to call verify;
  - do not implement `--local-arm64` in this slice unless OpenSpec tasks explicitly include it after preflight/verify are green.

## 5. Risk Notes

- Current local machine is missing at least `x86_64-apple-darwin` and `aarch64-unknown-linux-musl`; `preflight` should report these before package starts any long build.
- `verify` must be read-only. It may run `plutil`, `test`, `find`, `lipo`, and `codesign --verify`, but must not sign, copy, delete, notarize, or rebuild the app.
- Existing package behavior signs nested binaries and app, builds remote helper catalog, copies Rust-owned manifests, and optionally notarizes/creates DMG/ZIP. Default execution must preserve that flow.
