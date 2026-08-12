#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
engine="${repo_root}/homie/target/debug/homied-rs"
log="${repo_root}/docs/verification/startup-background-shell-invisibility/fc-04-startup-exec-probe.log"
calls="${repo_root}/docs/verification/startup-background-shell-invisibility/fc-04-exec-calls.jsonl"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/homie-exec-probe.XXXXXX")"
cleanup() {
    if [[ -n "${pid:-}" ]]; then
        kill "${pid}" >/dev/null 2>&1 || true
        wait "${pid}" >/dev/null 2>&1 || true
    fi
    rm -rf "${tmp}"
}
trap cleanup EXIT

mkdir -p "${tmp}/bin" "${tmp}/home" "${tmp}/support"
: > "${calls}"

for name in ssh rsync node gh lsof open osascript; do
    cat > "${tmp}/bin/${name}" <<'SH'
#!/usr/bin/env bash
printf '{"tool":"%s","argv":%q}\n' "$(basename "$0")" "$*" >> "${HOMIE_EXEC_PROBE_CALLS}"
exit 127
SH
    chmod +x "${tmp}/bin/${name}"
done

{
    echo "engine=${engine}"
    echo "support=${tmp}/support"
    echo "wrapper_bin=${tmp}/bin"
} > "${log}"

if [[ ! -x "${engine}" ]]; then
    echo "missing engine binary: ${engine}" | tee -a "${log}"
    exit 2
fi

PATH="${tmp}/bin:/usr/bin:/bin:/usr/sbin:/sbin" \
HOMIE_EXEC_PROBE_CALLS="${calls}" \
HOME="${tmp}/home" \
HOMIE_APP_SUPPORT="${tmp}/support" \
"${engine}" >> "${log}" 2>&1 &
pid=$!

deadline=$((SECONDS + 5))
while [[ ${SECONDS} -lt ${deadline} ]]; do
    if [[ -S "${tmp}/support/daemon.sock" ]]; then
        break
    fi
    sleep 0.05
done

if [[ ! -S "${tmp}/support/daemon.sock" ]]; then
    echo "daemon socket was not created" | tee -a "${log}"
    exit 1
fi

if [[ -s "${calls}" ]]; then
    echo "unexpected startup exec calls:" | tee -a "${log}"
    cat "${calls}" | tee -a "${log}"
    exit 1
fi

echo "PASS: startup did not call remote/browser/system wrappers" | tee -a "${log}"
