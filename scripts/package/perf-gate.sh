#!/bin/sh
set -eu

app_path=""
scenario="all"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --app)
      shift
      app_path="${1:-}"
      ;;
    --scenario)
      shift
      scenario="${1:-}"
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
  shift
done

if [ -z "$app_path" ]; then
  echo "usage: scripts/package/perf-gate.sh --app <Homie.app> [--scenario all]" >&2
  exit 2
fi

if [ ! -d "$app_path" ]; then
  echo "perf-gate: app bundle missing: $app_path" >&2
  exit 1
fi

executable="$app_path/Contents/MacOS/Homie"
if [ ! -x "$executable" ]; then
  echo "perf-gate: executable missing or not executable: $executable" >&2
  exit 1
fi

case "$scenario" in
  all|normal|large) ;;
  *)
    echo "perf-gate: unsupported scenario: $scenario" >&2
    exit 2
    ;;
esac

echo "PERF_GATE=not_run"
echo "APP_PATH=$app_path"
echo "SCENARIO=$scenario"
echo "REASON=packaged GUI measurement requires an interactive macOS release host"
