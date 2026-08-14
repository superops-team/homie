#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_dir="$(cd "${script_dir}/.." && pwd)"

dry_run=0
scenario="typical"
appearance="system"
reduced_motion=0
settings_tab=""

usage() {
    cat <<'USAGE'
Usage: scripts/visual-gate.sh [--dry-run] [--scenario NAME] [--appearance system|light|dark] [--reduced-motion] [--settings TAB]

Plan or run the GPUI visual/platform validation entrypoint.

Options:
  --dry-run                 Print commands without launching the app.
  --scenario NAME           Sidebar preview scenario: typical, stress, empty, artifacts.
  --appearance VALUE        system, light, or dark. Exported as HOMIE_VISUAL_APPEARANCE for evidence.
  --reduced-motion          Export HOMIE_REDUCED_MOTION=1 for evidence and future app wiring.
  --settings TAB            Forward to scripts/dev.sh --settings.
  -h, --help                Show this help.
USAGE
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)
            dry_run=1
            shift
            ;;
        --scenario)
            scenario="${2:?--scenario requires a value}"
            shift 2
            ;;
        --scenario=*)
            scenario="${1#*=}"
            shift
            ;;
        --appearance)
            appearance="${2:?--appearance requires a value}"
            shift 2
            ;;
        --appearance=*)
            appearance="${1#*=}"
            shift
            ;;
        --reduced-motion)
            reduced_motion=1
            shift
            ;;
        --settings)
            settings_tab="${2:?--settings requires a value}"
            shift 2
            ;;
        --settings=*)
            settings_tab="${1#*=}"
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

case "${scenario}" in
    typical|stress|empty|artifacts) ;;
    *)
        echo "error: unknown scenario: ${scenario}" >&2
        exit 2
        ;;
esac

case "${appearance}" in
    system|light|dark) ;;
    *)
        echo "error: unknown appearance: ${appearance}" >&2
        exit 2
        ;;
esac

cmd=(
    env
    "HOMIE_SIDEBAR_PREVIEW=1"
    "HOMIE_SIDEBAR_SCENARIO=${scenario}"
    "HOMIE_VISUAL_APPEARANCE=${appearance}"
)

if [[ "${reduced_motion}" == "1" ]]; then
    cmd+=("HOMIE_REDUCED_MOTION=1")
fi

cmd+=("${workspace_dir}/scripts/dev.sh")

if [[ -n "${settings_tab}" ]]; then
    cmd+=("--settings" "${settings_tab}")
fi

if [[ "${dry_run}" == "1" ]]; then
    printf 'Visual gate command:'
    printf ' %q' "${cmd[@]}"
    printf '\n'
    printf 'Evidence checklist: scenario=%s appearance=%s reduced_motion=%s settings=%s\n' \
        "${scenario}" "${appearance}" "${reduced_motion}" "${settings_tab:-none}"
    exit 0
fi

exec "${cmd[@]}"
