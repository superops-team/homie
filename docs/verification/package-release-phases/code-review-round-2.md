# Package/Release Phases Code Review Round 2

## 1. Scope

Second-pass review focused on hidden shell, release, CI and verification risks after round 1.

## 2. Hidden Risk Review

| Risk | Result | Evidence / Handling |
|---|---|---|
| `Contents/MacOS/homie` check may be stricter than previous CI | pass | `homie/crates/homie-app/Cargo.toml` declares `binaries = [{ path = "homie", main = true }]`; local dev app also contains `Contents/MacOS/homie` |
| `verify` might require release-only remote helper catalog for dev bundles | pass | `is_release_bundle_path` only requires remote helpers for `homie.app`, bundles with `Resources/licenses`, or bundles already containing `remote-helpers`; dev bundle path skips that check |
| `preflight` may report one missing target and stop before finding others | pass | current evidence reports both missing targets: `x86_64-apple-darwin` and `aarch64-unknown-linux-musl` |
| `--help` or `--phase verify` may require Cargo/Rust setup | pass | both return before toolchain setup and preflight |
| CI could drift from local verify behavior | pass | CI now delegates to the same `package.sh --phase verify --app` path |
| Full package default path may skip preflight | pass | default `phase="package"` calls `run_preflight` before entering the existing build sequence |

## 3. Not Changed

- Signing and notarization env names are unchanged.
- DMG and updater zip artifact names/formats are unchanged.
- Remote helper catalog build remains in the default full package path.
- `--local-arm64` and `--skip-build` are intentionally not implemented in this slice.

## 4. Conclusion

No P0/P1 hidden risks remain for this slice. Remaining limitation is environmental: the local machine cannot run a full package until the missing Rust targets are installed.
