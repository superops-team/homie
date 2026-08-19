#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_dir="$(cd "${script_dir}/.." && pwd)"
dist_dir="${HOMIE_DIST_DIR:-${workspace_dir}/dist}"
app_path="${dist_dir}/homie.app"
# The updater compares against CARGO_PKG_VERSION, so artifact names have to come
# from the same place rather than a hand-passed number that can drift from it.
cargo_version="$(sed -n 's/^version = "\(.*\)"/\1/p' "${workspace_dir}/crates/homie-app/Cargo.toml" | head -1)"
version="${HOMIE_VERSION:-${cargo_version:-0.1.0}}"
dmg_path="${dist_dir}/homie-${version}-universal.dmg"
# Update artifact: a zip of the stapled bundle, which is what homie's updater
# downloads. See crates/homie-updater/src/install.rs.
zip_path="${dist_dir}/homie-${version}-universal.zip"
entitlements="${workspace_dir}/assets/homie.entitlements"
# NEVER default to /tmp/homie-shared-target: that cache is shared with agent
# worktrees and cross-workspace fingerprint collisions produce Franken-builds
# (stale crates from other checkouts linked into the shipped app).
target_dir="${CARGO_TARGET_DIR:-${workspace_dir}/target}"
universal_dir="${target_dir}/universal-apple-darwin/release"
universal_binary="${universal_dir}/homie"
universal_engine_binary="${universal_dir}/homied-rs"
universal_holder_binary="${universal_dir}/homie-holder"
universal_askpass_binary="${universal_dir}/homie-ssh-askpass"

usage() {
    cat <<'USAGE'
Usage: scripts/package.sh [--phase PHASE] [--app APP_PATH]

Build and verify the release homie.app bundle.

Options:
  --phase preflight  Check required tools and Rust targets before long builds.
  --phase verify     Verify an existing app bundle without modifying it.
  --app APP_PATH     App bundle to verify with --phase verify.
  -h, --help         Show this help.

With no arguments, scripts/package.sh runs the complete existing package flow.
USAGE
}

phase="package"
verify_app_path=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --phase)
            if [[ $# -lt 2 ]]; then
                echo "error: --phase requires a value" >&2
                usage >&2
                exit 2
            fi
            phase="$2"
            shift 2
            ;;
        --phase=*)
            phase="${1#*=}"
            shift
            ;;
        --app)
            if [[ $# -lt 2 ]]; then
                echo "error: --app requires a path" >&2
                usage >&2
                exit 2
            fi
            verify_app_path="$2"
            shift 2
            ;;
        --app=*)
            verify_app_path="${1#*=}"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "error: unknown option: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

case "${phase}" in
    package|preflight|verify) ;;
    *)
        echo "error: unsupported phase: ${phase}" >&2
        usage >&2
        exit 2
        ;;
esac

if [[ -n "${verify_app_path}" && "${phase}" != "verify" ]]; then
    echo "error: --app is only valid with --phase verify" >&2
    exit 2
fi
if [[ "${phase}" == "verify" && -z "${verify_app_path}" ]]; then
    echo "error: --phase verify requires --app <path>" >&2
    exit 2
fi

fail_check() {
    local message="$1"
    echo "error: ${message}" >&2
    return 1
}

is_release_bundle_path() {
    local app="$1"
    [[ "$(basename "${app}")" == "homie.app" || -d "${app}/Contents/Resources/licenses" ]]
}

verify_app() {
    local app="$1"
    local failures=0
    local contents="${app}/Contents"
    local resources="${contents}/Resources"
    local bin_dir="${resources}/bin"

    add_failure() {
        echo "error: $1" >&2
        failures=$((failures + 1))
    }
    require_file() {
        [[ -f "$1" ]] || add_failure "missing file: $1"
    }
    require_exec() {
        [[ -x "$1" ]] || add_failure "missing executable: $1"
    }

    if [[ ! -d "${app}" ]]; then
        fail_check "app bundle does not exist: ${app}"
        return 1
    fi

    if command -v plutil >/dev/null 2>&1; then
        plutil -lint "${contents}/Info.plist" >/dev/null || add_failure "invalid Info.plist: ${contents}/Info.plist"
    else
        add_failure "plutil is missing"
    fi

    require_exec "${contents}/MacOS/homie"
    require_exec "${bin_dir}/homie"
    require_exec "${bin_dir}/homied-rs"
    require_exec "${bin_dir}/homie-holder"
    require_exec "${bin_dir}/homie-ssh-askpass"

    local source_manifests bundled_manifests
    source_manifests="$(find "${workspace_dir}/crates/homie-engine/manifests" -name '*.json' -type f | wc -l | tr -d ' ')"
    if [[ -d "${bin_dir}/manifests" ]]; then
        bundled_manifests="$(find "${bin_dir}/manifests" -name '*.json' -type f | wc -l | tr -d ' ')"
    else
        bundled_manifests=0
    fi
    if [[ ! -f "${bin_dir}/manifests/codex.json" || "${bundled_manifests}" != "${source_manifests}" || "${bundled_manifests}" -lt 20 ]]; then
        add_failure "bundled Agent catalog is incomplete: ${bundled_manifests} manifest(s), ${source_manifests} in source"
    fi

    if is_release_bundle_path "${app}" || [[ -d "${bin_dir}/remote-helpers" ]]; then
        require_file "${bin_dir}/remote-helpers/manifest.json"
        local helper_count=0
        if [[ -d "${bin_dir}/remote-helpers/artifacts" ]]; then
            helper_count="$(find "${bin_dir}/remote-helpers/artifacts" -name homie-remote -type f | wc -l | tr -d ' ')"
        fi
        if [[ "${helper_count}" != "3" ]]; then
            add_failure "expected 3 remote helper artifacts, found ${helper_count}"
        fi
    else
        echo "==> Skipping remote helper catalog check for non-release app bundle"
    fi

    if is_release_bundle_path "${app}"; then
        for binary in homied-rs homie-holder homie-ssh-askpass; do
            if [[ -x "${bin_dir}/${binary}" ]]; then
                lipo "${bin_dir}/${binary}" -verify_arch arm64 x86_64 >/dev/null \
                    || add_failure "${binary} is not universal arm64/x86_64"
            fi
        done
    fi

    codesign --verify --deep --strict "${app}" >/dev/null || add_failure "codesign verification failed: ${app}"

    if [[ "${failures}" -gt 0 ]]; then
        return 1
    fi
    echo "Verified ${app}"
}

run_preflight() {
    local failures=0

    require_command() {
        if ! command -v "$1" >/dev/null 2>&1; then
            echo "error: missing required tool: $1" >&2
            failures=$((failures + 1))
        fi
    }
    require_target() {
        local installed="$1"
        local target="$2"
        if ! grep -qx "${target}" <<<"${installed}"; then
            echo "error: missing Rust target: ${target}" >&2
            failures=$((failures + 1))
        fi
    }

    require_command cargo
    require_command rustup
    require_command cargo-packager
    require_command swift
    require_command lipo
    require_command codesign
    require_command plutil
    require_command python3

    local installed_targets=""
    if command -v rustup >/dev/null 2>&1; then
        installed_targets="$(rustup target list --installed 2>/dev/null || true)"
        require_target "${installed_targets}" aarch64-apple-darwin
        require_target "${installed_targets}" x86_64-apple-darwin
        require_target "${installed_targets}" x86_64-unknown-linux-musl
        require_target "${installed_targets}" aarch64-unknown-linux-musl
    fi

    if [[ "$(uname -s)" == "Darwin" ]]; then
        require_command cargo-zigbuild
    fi
    if [[ "${HOMIE_CREATE_DMG:-0}" == "1" ]]; then
        require_command hdiutil
    fi
    if [[ -n "${APPLE_NOTARIZATION_KEYCHAIN_PROFILE:-${APPLE_KEYCHAIN_PROFILE:-${NOTARY_PROFILE:-}}}" \
        || -n "${APPLE_NOTARIZATION_APPLE_ID:-${APPLE_ID:-}}" \
        || -n "${APPLE_NOTARIZATION_PASSWORD:-${APPLE_PASSWORD:-}}" \
        || -n "${APPLE_NOTARIZATION_TEAM_ID:-${APPLE_TEAM_ID:-}}" ]]; then
        require_command xcrun
    fi

    if [[ "${CARGO_TARGET_DIR:-}" == "/tmp/homie-shared-target" ]]; then
        echo "error: CARGO_TARGET_DIR must not point at /tmp/homie-shared-target" >&2
        failures=$((failures + 1))
    fi

    if [[ "${failures}" -gt 0 ]]; then
        return 1
    fi
    echo "Preflight passed"
}

if [[ "${phase}" == "verify" ]]; then
    verify_app "${verify_app_path}"
    exit $?
fi

# Toolchain location. The migration-era toolchain lived in /tmp, which macOS
# sweeps -- a reboot deleted it mid-project and releases could not be built at
# all until it was reinstalled. Prefer the persistent home install and fall back
# to /tmp only if that is where this machine still keeps it.
if [[ -x "${HOME}/.cargo/bin/cargo" ]]; then
    export CARGO_HOME="${CARGO_HOME:-${HOME}/.cargo}"
    export RUSTUP_HOME="${RUSTUP_HOME:-${HOME}/.rustup}"
else
    export CARGO_HOME="${CARGO_HOME:-/tmp/homie-cargo-home}"
    export RUSTUP_HOME="${RUSTUP_HOME:-/tmp/homie-rustup-home}"
fi
export PATH="${CARGO_HOME}/bin:${PATH}"
if ! command -v cargo >/dev/null 2>&1; then
    echo "error: no cargo on PATH (looked in ${CARGO_HOME}/bin)" >&2
    exit 1
fi
export CARGO_TARGET_DIR="${target_dir}"
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-15.0}"
export CLANG_MODULE_CACHE_PATH="${CLANG_MODULE_CACHE_PATH:-${target_dir}/clang-module-cache}"

# Enter the workspace before preflight so `rustup target list --installed`
# resolves the toolchain pinned by homie/rust-toolchain.toml, not the runner
# default. The CI "Install universal targets" step installs the slices under
# that pinned toolchain; checking the wrong toolchain makes preflight report
# every universal target as missing and the package step fails before building.
cd "${workspace_dir}"

run_preflight
if [[ "${phase}" == "preflight" ]]; then
    exit 0
fi

echo "==> Building homie for Apple silicon"
cargo build --release --package homie-app --bin homie --target aarch64-apple-darwin

echo "==> Building homie for Intel"
cargo build --release --package homie-app --bin homie --target x86_64-apple-darwin

echo "==> Creating universal executable"
mkdir -p "${universal_dir}" "${dist_dir}"
lipo -create \
    "${target_dir}/aarch64-apple-darwin/release/homie" \
    "${target_dir}/x86_64-apple-darwin/release/homie" \
    -output "${universal_binary}"
lipo "${universal_binary}" -verify_arch arm64 x86_64
echo "==> Assembling ${app_path} with cargo-packager"
cargo packager \
    --release \
    --packages homie-app \
    --formats app \
    --target universal-apple-darwin \
    --binaries-dir "${universal_dir}" \
    --out-dir "${dist_dir}"

# Ship the same reviewed dependency disclosure that CI validates. The JSON is
# also attached to GitHub Releases so users can inspect it without mounting the
# app bundle.
third_party_inventory="${dist_dir}/THIRD-PARTY-LICENSES.json"
echo "==> Generating third-party license inventory"
python3 "${workspace_dir}/../scripts/check-licenses.py" --output "${third_party_inventory}"
license_dir="${app_path}/Contents/Resources/licenses"
mkdir -p "${license_dir}"
cp "${workspace_dir}/../LICENSE" "${license_dir}/Apache-2.0.txt"
cp "${workspace_dir}/../NOTICE" "${license_dir}/NOTICE.txt"
cp "${workspace_dir}/../license-policy.json" "${license_dir}/license-policy.json"
cp "${workspace_dir}/../LICENSES/"*.txt "${license_dir}/"
cp "${third_party_inventory}" "${license_dir}/THIRD-PARTY-LICENSES.json"

# The Rust Engine is authoritative. The existing CLI remains packaged for
# local automation/hooks, but no daemon, Holder, Agent manifest, or remote
# transport artifact is sourced from a Swift product.
repo_root="$(cd "${workspace_dir}/.." && pwd)"
echo "==> Building the local automation CLI"
swift build --package-path "${repo_root}" -c release --product homie
daemon_bin="$(swift build --package-path "${repo_root}" -c release --show-bin-path)"
app_bin_dir="${app_path}/Contents/Resources/bin"
echo "==> Bundling CLI into Resources/bin"
mkdir -p "${app_bin_dir}"
cp "${daemon_bin}/homie" "${app_bin_dir}/homie"

# The Rust Engine is the authoritative daemon launched by homie. The remote
# Helper catalog below is consumed by this executable directly.
echo "==> Building the authoritative Rust Engine (universal)"
cargo build --release --package homie-engine --bin homied-rs --bin homie-holder \
    --bin homie-ssh-askpass \
    --target aarch64-apple-darwin
cargo build --release --package homie-engine --bin homied-rs --bin homie-holder \
    --bin homie-ssh-askpass \
    --target x86_64-apple-darwin
lipo -create \
    "${target_dir}/aarch64-apple-darwin/release/homied-rs" \
    "${target_dir}/x86_64-apple-darwin/release/homied-rs" \
    -output "${universal_engine_binary}"
lipo -create \
    "${target_dir}/aarch64-apple-darwin/release/homie-holder" \
    "${target_dir}/x86_64-apple-darwin/release/homie-holder" \
    -output "${universal_holder_binary}"
lipo -create \
    "${target_dir}/aarch64-apple-darwin/release/homie-ssh-askpass" \
    "${target_dir}/x86_64-apple-darwin/release/homie-ssh-askpass" \
    -output "${universal_askpass_binary}"
lipo "${universal_engine_binary}" -verify_arch arm64 x86_64
lipo "${universal_holder_binary}" -verify_arch arm64 x86_64
lipo "${universal_askpass_binary}" -verify_arch arm64 x86_64
cp "${universal_engine_binary}" "${app_bin_dir}/homied-rs"
cp "${universal_holder_binary}" "${app_bin_dir}/homie-holder"
cp "${universal_askpass_binary}" "${app_bin_dir}/homie-ssh-askpass"

# The default SSH transport bootstraps one exact Rust Helper artifact selected
# by remote OS/architecture. This build is independent of all daemon products
# above and emits a versioned manifest verified again before upload.
echo "==> Building three-platform Rust remote Helper catalog"
remote_helpers_dir="${app_bin_dir}/remote-helpers"
rm -rf "${remote_helpers_dir}"
HOMIE_SIGN_IDENTITY="${HOMIE_SIGN_IDENTITY:-}" "${script_dir}/build-remote-helpers.sh" "${remote_helpers_dir}"

# Rust-owned Agent catalog used by local and remote session orchestration.
rm -rf "${app_bin_dir}/manifests"
cp -R "${workspace_dir}/crates/homie-engine/manifests" "${app_bin_dir}/manifests"
# Count, not just presence. A catalog that is merely SMALLER never errors at
# runtime: each missing manifest silently downgrades that agent to a bare login
# shell, which looks like a working session. That shipped once. The source
# directory is the reference, so any shrink between it and the bundle is a
# packaging bug worth failing the release over.
source_manifests="$(find "${workspace_dir}/crates/homie-engine/manifests" -name '*.json' -type f | wc -l | tr -d ' ')"
bundled_manifests="$(find "${app_bin_dir}/manifests" -name '*.json' -type f | wc -l | tr -d ' ')"
if [[ ! -f "${app_bin_dir}/manifests/codex.json" || "${bundled_manifests}" != "${source_manifests}" || "${bundled_manifests}" -lt 20 ]]; then
    echo "error: bundled Agent catalog is incomplete: ${bundled_manifests} manifest(s) bundled, ${source_manifests} in source (expected at least 20)" >&2
    exit 1
fi
echo "==> Bundled ${bundled_manifests} Agent manifests"

# Inside-out signing: sign the nested daemon binaries FIRST (their own hardened
# runtime + timestamp), then the app LAST WITHOUT --deep. A --deep sign would
# re-stamp the nested executables with the app's identifier and can fail
# notarization; nested Mach-O must be signed independently.
# Auto-detect a Developer ID when none is given, exactly as release.sh does.
# TCC keys privacy grants (Documents/Desktop access prompts) to the signing
# identity: ad-hoc changes every rebuild, so each dev install used to re-ask —
# a stable Developer ID makes one "Allow" stick across every future install.
if [[ -z "${HOMIE_SIGN_IDENTITY:-}" ]]; then
    HOMIE_SIGN_IDENTITY="$(security find-identity -v -p codesigning 2>/dev/null \
        | grep "Developer ID Application" | head -1 \
        | sed -E 's/.*"(.*)".*/\1/' || true)"
fi
sign_id="${HOMIE_SIGN_IDENTITY:--}"
ts_flag=(--timestamp)
[[ "${sign_id}" == "-" ]] && ts_flag=(--timestamp=none) && echo "==> No signing identity found; ad-hoc signature"
[[ "${sign_id}" != "-" ]] && echo "==> Signing with: ${sign_id}"
echo "==> Signing nested executables"
codesign --force --options runtime "${ts_flag[@]}" --sign "${sign_id}" "${app_bin_dir}/homie"
codesign --force --options runtime "${ts_flag[@]}" --sign "${sign_id}" "${app_bin_dir}/homied-rs"
codesign --force --options runtime "${ts_flag[@]}" --sign "${sign_id}" "${app_bin_dir}/homie-holder"
codesign --force --options runtime "${ts_flag[@]}" --sign "${sign_id}" "${app_bin_dir}/homie-ssh-askpass"
# The Apple remote Helper is deliberately NOT signed here. Signing rewrites the
# Mach-O, and its length and digest are already recorded in the catalog manifest
# that the Engine verifies before upload; signing after the fact invalidates the
# manifest and the Engine rejects the entire catalog, disabling every remote
# host. build-remote-helpers.sh signs it before measuring instead.
echo "==> Signing ${app_path}"
codesign --force --options runtime "${ts_flag[@]}" \
    --entitlements "${entitlements}" \
    --identifier com.homie.homie \
    --sign "${sign_id}" \
    "${app_path}"

codesign --verify --deep --strict "${app_path}"

notary_profile="${APPLE_NOTARIZATION_KEYCHAIN_PROFILE:-${APPLE_KEYCHAIN_PROFILE:-${NOTARY_PROFILE:-}}}"
notary_apple_id="${APPLE_NOTARIZATION_APPLE_ID:-${APPLE_ID:-}}"
notary_password="${APPLE_NOTARIZATION_PASSWORD:-${APPLE_PASSWORD:-}}"
notary_team_id="${APPLE_NOTARIZATION_TEAM_ID:-${APPLE_TEAM_ID:-}}"
notary_requested=0
if [[ -n "${notary_profile}" || -n "${notary_apple_id}" || -n "${notary_password}" || -n "${notary_team_id}" ]]; then
    notary_requested=1
fi

# One place that knows how to talk to notarytool, called for the app zip and
# again for the DMG.
submit_for_notarization() {
    local artifact="$1"
    if [[ -n "${notary_profile}" ]]; then
        xcrun notarytool submit "${artifact}" --keychain-profile "${notary_profile}" --wait
    elif [[ -n "${notary_apple_id}" && -n "${notary_password}" && -n "${notary_team_id}" ]]; then
        xcrun notarytool submit "${artifact}" \
            --apple-id "${notary_apple_id}" \
            --password "${notary_password}" \
            --team-id "${notary_team_id}" \
            --wait
    else
        echo "error: set a keychain profile or all APPLE_NOTARIZATION_{APPLE_ID,PASSWORD,TEAM_ID} values" >&2
        exit 1
    fi
}

if [[ "${notary_requested}" == "1" ]]; then
    if [[ -z "${HOMIE_SIGN_IDENTITY:-}" ]]; then
        echo "error: notarization requires HOMIE_SIGN_IDENTITY" >&2
        exit 1
    fi

    # The .app is notarized and stapled BEFORE the DMG is built, so both the
    # DMG's copy and the update zip carry their own ticket. A ticket stapled
    # only to the DMG would leave the extracted bundle needing an online
    # Gatekeeper check, and the updater verifies downloads offline.
    echo "==> Notarizing ${app_path}"
    notarization_zip="$(mktemp -d "${TMPDIR:-/tmp}/homie-notarize.XXXXXX")/homie.zip"
    ditto -c -k --keepParent "${app_path}" "${notarization_zip}"
    submit_for_notarization "${notarization_zip}"
    rm -rf "$(dirname "${notarization_zip}")"
    xcrun stapler staple "${app_path}"
    xcrun stapler validate "${app_path}"
fi

if [[ "${HOMIE_CREATE_DMG:-0}" == "1" ]]; then
    echo "==> Creating ${dmg_path}"
    dmg_stage="$(mktemp -d "${TMPDIR:-/tmp}/homie-dmg.XXXXXX")"
    cleanup_dmg_stage() {
        rm -rf "${dmg_stage}"
    }
    trap cleanup_dmg_stage EXIT
    ditto "${app_path}" "${dmg_stage}/homie.app"
    ln -s /Applications "${dmg_stage}/Applications"
    rm -f "${dmg_path}"
    hdiutil create -quiet -volname "homie" -srcfolder "${dmg_stage}" -ov -format UDZO "${dmg_path}"
    if [[ -n "${HOMIE_SIGN_IDENTITY:-}" ]]; then
        codesign --force --timestamp --sign "${HOMIE_SIGN_IDENTITY}" "${dmg_path}"
    fi
fi

if [[ "${HOMIE_CREATE_DMG:-0}" == "1" && "${notary_requested}" == "1" ]]; then
    echo "==> Notarizing ${dmg_path}"
    submit_for_notarization "${dmg_path}"
    xcrun stapler staple "${dmg_path}"
    xcrun stapler validate "${dmg_path}"
fi

if [[ "${HOMIE_CREATE_ZIP:-0}" == "1" || "${notary_requested}" == "1" ]]; then
    echo "==> Creating ${zip_path}"
    rm -f "${zip_path}"
    # --keepParent puts homie.app at the archive root, which is the layout the
    # updater's unpack step requires.
    ditto -c -k --keepParent "${app_path}" "${zip_path}"
fi

bundle_size="$(du -sh "${app_path}" | awk '{print $1}')"
echo "Built ${app_path} (${bundle_size})"
if [[ "${HOMIE_CREATE_DMG:-0}" == "1" ]]; then
    echo "Built ${dmg_path}"
fi
if [[ -f "${zip_path}" ]]; then
    echo "Built ${zip_path}"
fi
