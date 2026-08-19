#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_dir="$(cd "${script_dir}/.." && pwd)"
target_dir="${CARGO_TARGET_DIR:-${workspace_dir}/target}"
profile="debug"
settings_preview=""
full_bundle=0
smoke=0
no_launch=0
cargo_args=()

usage() {
    cat <<'USAGE'
Usage: scripts/dev.sh [--release] [--settings TAB] [--full] [--no-launch] [--smoke] [-- CARGO_BUILD_ARGS...]

Build and launch an unmistakable development copy of homie.

Options:
  --release       Build with Cargo's release profile.
  --settings TAB  Open Settings on general, terminal, resources, or remote.
  --full          Bundle the local Engine, holder, askpass, MCP, CLI, and manifests.
  --no-launch     Build the app bundle and print its path without launching it.
  --smoke         With --full, verify the bundle and smoke a temporary Engine.
  -h, --help      Show this help.

Arguments after -- are passed to cargo build. Options that change Cargo's
target directory, target triple, or profile are not supported; set
CARGO_TARGET_DIR or use --release instead.
USAGE
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --release)
            profile="release"
            cargo_args+=("$1")
            shift
            ;;
        --settings)
            if [[ $# -lt 2 ]]; then
                echo "error: --settings requires a tab" >&2
                usage >&2
                exit 2
            fi
            settings_preview="$2"
            shift 2
            ;;
        --settings=*)
            settings_preview="${1#*=}"
            shift
            ;;
        --full)
            full_bundle=1
            shift
            ;;
        --no-launch)
            no_launch=1
            shift
            ;;
        --smoke)
            smoke=1
            full_bundle=1
            no_launch=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        --)
            shift
            cargo_args+=("$@")
            break
            ;;
        *)
            echo "error: unknown option: $1 (put cargo build arguments after --)" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [[ "${smoke}" == "1" && "${full_bundle}" != "1" ]]; then
    echo "error: --smoke requires --full" >&2
    exit 2
fi

case "${settings_preview}" in
    ""|general|terminal|resources|remote) ;;
    *)
        echo "error: unknown Settings tab: ${settings_preview}" >&2
        exit 2
        ;;
esac

if (( ${#cargo_args[@]} > 0 )); then
    for argument in "${cargo_args[@]}"; do
        case "${argument}" in
            --target|--target=*|--target-dir|--target-dir=*|--profile|--profile=*)
                echo "error: ${argument} changes where the app binary is written" >&2
                exit 2
                ;;
        esac
    done
fi

branch="$(git -C "${workspace_dir}" symbolic-ref --quiet --short HEAD || true)"
branch="${branch:-detached}"
short_sha="$(git -C "${workspace_dir}" rev-parse --short=8 HEAD)"
dirty=""
if ! git -C "${workspace_dir}" diff --quiet --ignore-submodules -- \
    || ! git -C "${workspace_dir}" diff --cached --quiet --ignore-submodules --; then
    dirty="+dirty"
fi
build_label="${branch}@${short_sha}${dirty}"
bundle_id="com.homie.homie.dev.${short_sha}"
display_name="homie dev ${short_sha}"
swift_configuration="${profile}"
[[ "${swift_configuration}" == "debug" ]] && swift_configuration="debug"

sign_binary() {
    local binary="$1"
    codesign --force --timestamp=none --sign - "${binary}" >/dev/null
}

copy_executable() {
    local source="$1"
    local target="$2"
    if [[ ! -x "${source}" ]]; then
        echo "error: missing executable ${source}" >&2
        exit 1
    fi
    cp "${source}" "${target}"
    chmod 755 "${target}"
    sign_binary "${target}"
}

cargo_build() {
    if (( ${#cargo_args[@]} > 0 )); then
        cargo build "$@" "${cargo_args[@]}"
    else
        cargo build "$@"
    fi
}

wait_for_socket() {
    local socket="$1"
    local attempts="${2:-80}"
    for _ in $(seq 1 "${attempts}"); do
        [[ -S "${socket}" ]] && return 0
        sleep 0.1
    done
    return 1
}

run_full_smoke() {
    local app="$1"
    local app_support
    app_support="$(mktemp -d "${TMPDIR:-/tmp}/homie-dev-smoke-support.XXXXXX")"
    local socket="${app_support}/daemon.sock"
    local engine="${app}/Contents/Resources/bin/homied-rs"
    local cli="${app}/Contents/Resources/bin/homie"
    local log="${app_support}/homied-rs.log"
    local smoke_home="${app_support}/home"
    local pid=""
    mkdir -p "${smoke_home}"
    cleanup_smoke() {
        if [[ -n "${pid}" ]] && kill -0 "${pid}" >/dev/null 2>&1; then
            kill "${pid}" >/dev/null 2>&1 || true
            wait "${pid}" >/dev/null 2>&1 || true
        fi
        rm -rf "${app_support}"
    }
    trap cleanup_smoke RETURN

    echo "==> Smoking bundled Engine with temporary app support"
    env \
        -u HOMIE_SOCKET \
        -u HOMIE_SESSION_ID \
        -u HOMIE_CLI \
        -u NO_COLOR \
        HOMIE_APP_SUPPORT="${app_support}" \
        "${engine}" >"${log}" 2>&1 &
    pid="$!"
    if ! wait_for_socket "${socket}"; then
        cat "${log}" >&2 || true
        echo "error: temporary Engine did not create ${socket}" >&2
        return 1
    fi
    env \
        -u HOMIE_SESSION_ID \
        -u HOMIE_CLI \
        -u NO_COLOR \
        HOMIE_SOCKET="${socket}" \
        HOME="${smoke_home}" \
        "${cli}" status
    if grep -F "${HOME}/Library/Application Support/Homie" "${log}" >/dev/null 2>&1; then
        cat "${log}" >&2 || true
        echo "error: smoke log referenced the real Homie app support directory" >&2
        return 1
    fi
}

mkdir -p "${target_dir}"

cd "${workspace_dir}"
echo "==> Building ${display_name} (${profile})"
cargo_build --package homie-app --bin homie

binary="${target_dir}/${profile}/homie"
if [[ ! -x "${binary}" ]]; then
    echo "error: cargo did not produce ${binary}" >&2
    exit 1
fi

# Every invocation gets a fresh bundle. Replacing a bundle beneath a still-
# running process invalidates its code signature, which is especially easy to
# do when judging two builds side by side.
bundle_root="$(mktemp -d "${target_dir}/homie-dev-${short_sha}.XXXXXX")"
app_path="${bundle_root}/${display_name}.app"
contents="${app_path}/Contents"
mkdir -p "${contents}/MacOS" "${contents}/Resources"
cp "${binary}" "${contents}/MacOS/homie"
cp "${workspace_dir}/assets/dev-icon.icns" "${contents}/Resources/dev-icon.icns"

app_bin_dir="${contents}/Resources/bin"
if [[ "${full_bundle}" == "1" ]]; then
    echo "==> Building full development runtime (${profile})"
    cargo_build --package homie-engine --bin homied-rs --bin homie-holder --bin homie-ssh-askpass
    swift build --package-path "${workspace_dir}/.." -c "${swift_configuration}" --product homie
    swift_bin_dir="$(swift build --package-path "${workspace_dir}/.." -c "${swift_configuration}" --show-bin-path)"

    mkdir -p "${app_bin_dir}"
    copy_executable "${swift_bin_dir}/homie" "${app_bin_dir}/homie"
    copy_executable "${target_dir}/${profile}/homied-rs" "${app_bin_dir}/homied-rs"
    copy_executable "${target_dir}/${profile}/homie-holder" "${app_bin_dir}/homie-holder"
    copy_executable "${target_dir}/${profile}/homie-ssh-askpass" "${app_bin_dir}/homie-ssh-askpass"
    rm -rf "${app_bin_dir}/manifests"
    cp -R "${workspace_dir}/crates/homie-engine/manifests" "${app_bin_dir}/manifests"
fi

version="$(sed -n 's/^version = "\(.*\)"/\1/p' "${workspace_dir}/crates/homie-app/Cargo.toml" | head -1)"
cat > "${contents}/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key><string>en</string>
    <key>CFBundleDisplayName</key><string>${display_name}</string>
    <key>CFBundleExecutable</key><string>homie</string>
    <key>CFBundleIconFile</key><string>dev-icon.icns</string>
    <key>CFBundleIdentifier</key><string>${bundle_id}</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>CFBundleName</key><string>${display_name}</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>${version}</string>
    <key>CFBundleVersion</key><string>1</string>
    <key>LSMinimumSystemVersion</key><string>15.0</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>NSPrincipalClass</key><string>NSApplication</string>
</dict>
</plist>
PLIST

# A bundle assembled after Cargo's link step must be sealed as a unit. Ad-hoc
# signing gives it coherent bundle metadata without ever resembling a release.
codesign --force --sign - \
    --entitlements "${workspace_dir}/assets/homie.entitlements" \
    --identifier "${bundle_id}" \
    "${app_path}"
codesign --verify --deep --strict "${app_path}"

if [[ "${full_bundle}" == "1" ]]; then
    "${workspace_dir}/scripts/package.sh" --phase verify --app "${app_path}"
fi
if [[ "${smoke}" == "1" ]]; then
    run_full_smoke "${app_path}"
fi
if [[ "${no_launch}" == "1" ]]; then
    echo "==> Built ${display_name} (${build_label})"
    echo "    ${app_path}"
    exit 0
fi

launch_environment=("HOMIE_DEV=1" "HOMIE_DEV_BUILD=${build_label}")
if [[ -n "${settings_preview}" ]]; then
    launch_environment+=("HOMIE_SETTINGS_PREVIEW=${settings_preview}")
fi

echo "==> Launching ${display_name} (${build_label})"
echo "    ${app_path}"
exec env \
    -u HOMIE_SOCKET \
    -u HOMIE_SESSION_ID \
    -u HOMIE_CLI \
    -u NO_COLOR \
    "${launch_environment[@]}" \
    "${contents}/MacOS/homie"
