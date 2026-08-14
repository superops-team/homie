#!/usr/bin/env bash

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_dir="${root}/homie/crates/homie-engine/manifests"
mirror_dir="${root}/Sources/HomieCore/Resources/manifests"

if [[ ! -d "${source_dir}" ]]; then
    echo "error: authoritative manifest source is missing: ${source_dir}" >&2
    exit 1
fi
if [[ ! -d "${mirror_dir}" ]]; then
    echo "error: Swift manifest mirror is missing: ${mirror_dir}" >&2
    echo "hint: run scripts/sync-agent-manifests.sh" >&2
    exit 1
fi

diff_output="$(diff -qr "${source_dir}" "${mirror_dir}" || true)"
if [[ -n "${diff_output}" ]]; then
    echo "error: Swift agent manifest mirror is out of sync with the Rust Engine source" >&2
    echo >&2
    echo "${diff_output}" >&2
    echo >&2
    echo "hint: run scripts/sync-agent-manifests.sh" >&2
    exit 1
fi

echo "Agent manifest mirror is in sync."
