#!/usr/bin/env bash

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
run_browser=0

case "${1:-}" in
    "") ;;
    --browser) run_browser=1 ;;
    -h|--help)
        echo "usage: ./scripts/check.sh [--browser]"
        exit 0
        ;;
    *)
        echo "error: unknown option: $1" >&2
        echo "usage: ./scripts/check.sh [--browser]" >&2
        exit 2
        ;;
esac

for tool in bash cargo python3 swift; do
    if ! command -v "${tool}" >/dev/null 2>&1; then
        echo "error: ${tool} is required; see CONTRIBUTING.md" >&2
        exit 1
    fi
done

# Keep compiler caches inside the checkout. This works in sandboxed development
# environments and avoids depending on writable global Swift/Clang cache paths.
export CLANG_MODULE_CACHE_PATH="${CLANG_MODULE_CACHE_PATH:-${root}/.build/clang-module-cache}"
export SWIFTPM_MODULECACHE_OVERRIDE="${SWIFTPM_MODULECACHE_OVERRIDE:-${CLANG_MODULE_CACHE_PATH}}"
mkdir -p "${CLANG_MODULE_CACHE_PATH}"

swift_test_flags=()
developer_dir="$(xcode-select -p 2>/dev/null || true)"
testing_framework_dirs=()
testing_interop_dirs=()
if [[ -n "${developer_dir}" ]]; then
    testing_framework_dirs+=(
        "${developer_dir}/Library/Developer/Frameworks"
        "${developer_dir}/Platforms/MacOSX.platform/Developer/Library/Frameworks"
    )
    testing_interop_dirs+=(
        "${developer_dir}/Library/Developer/usr/lib"
        "${developer_dir}/Platforms/MacOSX.platform/Developer/usr/lib"
    )
fi
for framework_dir in "${testing_framework_dirs[@]}"; do
    [[ -d "${framework_dir}/Testing.framework" ]] || continue
    swift_test_flags+=(
        -Xswiftc -F
        -Xswiftc "${framework_dir}"
        -Xlinker -rpath
        -Xlinker "${framework_dir}"
    )
    break
done
for interop_dir in "${testing_interop_dirs[@]}"; do
    [[ -f "${interop_dir}/lib_TestingInterop.dylib" ]] || continue
    swift_test_flags+=(-Xlinker -rpath -Xlinker "${interop_dir}")
    break
done

echo "==> Shell and release publishing guards"
bash -n "${root}"/scripts/*.sh "${root}"/homie/scripts/*.sh
bash "${root}/scripts/check-agent-manifest-drift.sh"
bash "${root}/homie/scripts/test-publish-github-release.sh"
bash "${root}/homie/scripts/test-publish-homebrew-cask.sh"

echo "==> Swift CLI/protocol support"
swift test --package-path "${root}" --no-parallel "${swift_test_flags[@]}"

echo "==> Rust app"
(
    cd "${root}/homie"
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
)

echo "==> Dependency license policy"
python3 "${root}/scripts/check-licenses.py"

if [[ "${run_browser}" == "1" ]]; then
    if ! command -v npm >/dev/null 2>&1; then
        echo "error: npm is required for --browser" >&2
        exit 1
    fi
    echo "==> Browser sidecar"
    (
        cd "${root}/sidecar"
        npm ci
        npm audit --omit=dev
        npx playwright install chromium webkit firefox
    )
    HOMIE_RUN_BROWSER_TESTS=1 swift test --package-path "${root}" --filter BrowserPoolTests "${swift_test_flags[@]}"
fi

echo "All contributor checks passed."
