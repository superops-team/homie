#!/bin/sh
set -eu

app_path="${1:?usage: smoke-packaged-runtime.sh </absolute/path/to/Homie.app>}"
case "$app_path" in
  /*) ;;
  *)
    echo "app bundle path must be absolute: $app_path" >&2
    exit 1
    ;;
esac
if [ ! -d "$app_path" ]; then
  echo "app bundle is missing: $app_path" >&2
  exit 1
fi
app_path="$(CDPATH= cd -- "$app_path" && pwd -P)"

cli_binary="$app_path/Contents/Resources/bin/homie"
daemon_binary="$app_path/Contents/Resources/bin/homie-runtime-daemon"
holder_binary="$app_path/Contents/Resources/bin/homie-runtime-holder"
for binary in "$cli_binary" "$daemon_binary" "$holder_binary"; do
  if [ ! -f "$binary" ] || [ -L "$binary" ] || [ ! -x "$binary" ]; then
    echo "packaged runtime binary is missing or not executable: $binary" >&2
    exit 1
  fi
done

temp_root="$(mktemp -d /tmp/homie-package-smoke.XXXXXX)"
temp_root="$(CDPATH= cd -- "$temp_root" && pwd -P)"
data_dir="$temp_root/data"
daemon_log="$temp_root/daemon.log"
snapshot_response="$temp_root/state-snapshot.json"
shutdown_response="$temp_root/shutdown.json"
mkdir -m 0700 "$data_dir"
daemon_pid=""

cleanup() {
  if [ -n "$daemon_pid" ] && kill -0 "$daemon_pid" 2>/dev/null; then
    kill -TERM "$daemon_pid" 2>/dev/null || true
    wait "$daemon_pid" 2>/dev/null || true
  fi
  rm -rf "$temp_root"
}
trap cleanup EXIT HUP INT TERM

mode_of() {
  if stat -f '%Lp' "$1" >/dev/null 2>&1; then
    stat -f '%Lp' "$1"
  else
    stat -c '%a' "$1"
  fi
}

wait_for_socket() {
  attempt=0
  while [ "$attempt" -lt 200 ]; do
    if [ -S "$data_dir/runtime/daemon.sock" ]; then
      return 0
    fi
    if ! kill -0 "$daemon_pid" 2>/dev/null; then
      echo "packaged daemon exited before creating its runtime socket" >&2
      return 1
    fi
    attempt=$((attempt + 1))
    sleep 0.05
  done
  echo "packaged daemon did not create its runtime socket" >&2
  return 1
}

wait_for_daemon_exit() {
  attempt=0
  while [ "$attempt" -lt 200 ]; do
    state="$(ps -p "$daemon_pid" -o stat= 2>/dev/null || true)"
    case "$state" in
      ""|*Z*) break ;;
    esac
    attempt=$((attempt + 1))
    sleep 0.05
  done
  if [ "$attempt" -eq 200 ]; then
    echo "packaged daemon did not exit after daemon.shutdown" >&2
    return 1
  fi
  if ! wait "$daemon_pid"; then
    echo "packaged daemon exited unsuccessfully after daemon.shutdown" >&2
    return 1
  fi
  daemon_pid=""
}

require_json_value() {
  file="$1"
  key_path="$2"
  expected="$3"
  actual="$(/usr/bin/plutil -extract "$key_path" raw -o - "$file" 2>/dev/null)" || {
    echo "packaged CLI response is missing $key_path" >&2
    return 1
  }
  if [ "$actual" != "$expected" ]; then
    printf 'unexpected packaged CLI response for %s: expected %s, got %s\n' \
      "$key_path" "$expected" "$actual" >&2
    return 1
  fi
}

"$daemon_binary" --data-dir "$data_dir" >"$daemon_log" 2>&1 &
daemon_pid=$!
wait_for_socket

runtime_dir="$data_dir/runtime"
socket_path="$runtime_dir/daemon.sock"
lock_path="$runtime_dir/daemon.lock"
if [ "$(mode_of "$runtime_dir")" != "700" ]; then
  echo "packaged daemon runtime directory mode must be 0700" >&2
  exit 1
fi
if [ "$(mode_of "$socket_path")" != "600" ]; then
  echo "packaged daemon socket mode must be 0600" >&2
  exit 1
fi
if [ "$(mode_of "$lock_path")" != "600" ]; then
  echo "packaged daemon lock mode must be 0600" >&2
  exit 1
fi

printf '%s\n' \
  '{"type":"request","requestId":1,"method":"state.snapshot","params":{}}' |
  "$cli_binary" control-stdio --data-dir "$data_dir" >"$snapshot_response"
require_json_value "$snapshot_response" type response
require_json_value "$snapshot_response" requestId 1
require_json_value "$snapshot_response" ok true
/usr/bin/plutil -extract result.eventCursor raw -o - "$snapshot_response" >/dev/null
/usr/bin/plutil -extract result.sessions json -o - "$snapshot_response" >/dev/null

printf '%s\n' \
  '{"type":"request","requestId":2,"method":"daemon.shutdown","params":{}}' |
  "$cli_binary" control-stdio --data-dir "$data_dir" >"$shutdown_response"
require_json_value "$shutdown_response" type response
require_json_value "$shutdown_response" requestId 2
require_json_value "$shutdown_response" ok true
require_json_value "$shutdown_response" result.acknowledged true
wait_for_daemon_exit

echo "PACKAGED_RUNTIME_SMOKE=pass"
echo "HELLO_STATE_SNAPSHOT=pass"
echo "GUI_LAUNCH=not_run"
echo "NOTARIZATION=not_required"
