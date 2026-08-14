#!/usr/bin/env bash

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_dir="${root}/homie/crates/homie-engine/manifests"
mirror_dir="${root}/Sources/HomieCore/Resources/manifests"

if [[ ! -d "${source_dir}" ]]; then
    echo "error: authoritative manifest source is missing: ${source_dir}" >&2
    exit 1
fi

mkdir -p "${mirror_dir}"
staging="$(mktemp -d "${mirror_dir}.staging.XXXXXX")"
cleanup() {
    rm -rf "${staging}"
}
trap cleanup EXIT

find "${source_dir}" -maxdepth 1 -type f -name '*.json' -print0 \
    | while IFS= read -r -d '' manifest; do
        cp "${manifest}" "${staging}/$(basename "${manifest}")"
    done

source_count="$(find "${source_dir}" -maxdepth 1 -type f -name '*.json' | wc -l | tr -d ' ')"
mirror_count="$(find "${staging}" -maxdepth 1 -type f -name '*.json' | wc -l | tr -d ' ')"
if [[ "${source_count}" != "${mirror_count}" ]]; then
    echo "error: synced ${mirror_count} manifest(s), expected ${source_count}" >&2
    exit 1
fi

find "${mirror_dir}" -maxdepth 1 -type f -name '*.json' -delete
find "${staging}" -maxdepth 1 -type f -name '*.json' -print0 \
    | while IFS= read -r -d '' manifest; do
        mv "${manifest}" "${mirror_dir}/$(basename "${manifest}")"
    done

echo "Synced ${mirror_count} agent manifest(s) from Rust Engine source to Swift mirror."
