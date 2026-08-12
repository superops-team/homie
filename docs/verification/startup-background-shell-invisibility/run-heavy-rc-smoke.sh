#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
engine="${repo_root}/homie/target/debug/homied-rs"
log="${repo_root}/docs/verification/startup-background-shell-invisibility/fc-03-heavy-rc-smoke.log"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/homie-heavy-rc.XXXXXX")"
cleanup() {
    if [[ -n "${pid:-}" ]]; then
        kill "${pid}" >/dev/null 2>&1 || true
        wait "${pid}" >/dev/null 2>&1 || true
    fi
    rm -rf "${tmp}"
}
trap cleanup EXIT

cat > "${tmp}/fake-shell" <<'SH'
#!/usr/bin/env bash
if [[ "$*" == *"printenv PATH"* ]]; then
    echo "HEAVY_RC_WAS_RUN"
    sleep 3
    touch "${HOME}/rc-was-run"
    echo "/heavy/bin:/usr/bin:/bin"
    exit 0
fi
exec /bin/sh "$@"
SH
chmod +x "${tmp}/fake-shell"

home="${tmp}/home"
support="${tmp}/support"
mkdir -p "${home}" "${support}"

{
    echo "engine=${engine}"
    echo "home=${home}"
    echo "support=${support}"
    echo "fake_shell=${tmp}/fake-shell"
} > "${log}"

if [[ ! -x "${engine}" ]]; then
    echo "missing engine binary: ${engine}" | tee -a "${log}"
    exit 2
fi

HOME="${home}" \
SHELL="${tmp}/fake-shell" \
HOMIE_APP_SUPPORT="${support}" \
"${engine}" >> "${log}" 2>&1 &
pid=$!

deadline=$((SECONDS + 5))
while [[ ${SECONDS} -lt ${deadline} ]]; do
    if [[ -S "${support}/daemon.sock" ]]; then
        break
    fi
    sleep 0.05
done

if [[ ! -S "${support}/daemon.sock" ]]; then
    echo "daemon socket was not created" | tee -a "${log}"
    exit 1
fi

if [[ -e "${home}/rc-was-run" ]]; then
    echo "interactive shell rc was executed during startup" | tee -a "${log}"
    exit 1
fi

if grep -q "HEAVY_RC_WAS_RUN" "${log}"; then
    echo "shell rc output leaked into startup log" | tee -a "${log}"
    exit 1
fi

echo "PASS: daemon startup did not execute heavy interactive shell capture" | tee -a "${log}"
