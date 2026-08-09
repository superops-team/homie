#!/bin/sh
set -eu

repo_root="$(CDPATH= cd -- "$(dirname "$0")/../../.." && pwd -P)"
smoke_script="$repo_root/scripts/package/tests/smoke-packaged-runtime.sh"
fixture_root="$(mktemp -d /tmp/homie-packaged-runtime-smoke-test.XXXXXX)"
trap 'rm -rf "$fixture_root"' EXIT

app_path="$fixture_root/Homie.app"
bin_dir="$app_path/Contents/Resources/bin"
app_bin_dir="$app_path/Contents/MacOS"
command_log="$fixture_root/commands.log"
mkdir -p "$bin_dir" "$app_bin_dir"
: > "$command_log"

cat > "$bin_dir/homie-runtime-daemon" <<SCRIPT
#!/bin/sh
set -eu
[ "\$#" -eq 2 ]
[ "\$1" = "--data-dir" ]
case "\$2" in
  /*) ;;
  *) exit 91 ;;
esac
data_dir="\$2"
printf 'daemon_data=%s\n' "\$data_dir" >> "$command_log"
umask 077
mkdir -p "\$data_dir/runtime"
chmod 0700 "\$data_dir/runtime"
: > "\$data_dir/runtime/daemon.lock"
chmod 0600 "\$data_dir/runtime/daemon.lock"
printf '%s\n' "\$\$" > "\$data_dir/runtime/daemon.pid"
exec python3 - "\$data_dir/runtime/daemon.sock" <<'PY'
import signal
import socket
import sys
import time
import os

def stop(_signum, _frame):
    sys.exit(0)

signal.signal(signal.SIGTERM, stop)
listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
listener.bind(sys.argv[1])
os.chmod(sys.argv[1], 0o600)
while True:
    time.sleep(1)
PY
SCRIPT

cat > "$bin_dir/homie" <<SCRIPT
#!/bin/sh
set -eu
[ "\$#" -eq 3 ]
[ "\$1" = "control-stdio" ]
[ "\$2" = "--data-dir" ]
case "\$3" in
  /*) ;;
  *) exit 92 ;;
esac
data_dir="\$3"
request="\$(cat)"
printf 'cli_data=%s\nrequest=%s\n' "\$data_dir" "\$request" >> "$command_log"
case "\$request" in
  *'"method":"state.snapshot"'*)
    printf '%s\n' '{"type":"response","requestId":1,"ok":true,"result":{"eventCursor":0,"sessions":[]}}'
    ;;
  *'"method":"daemon.shutdown"'*)
    printf '%s\n' '{"type":"response","requestId":2,"ok":true,"result":{"acknowledged":true}}'
    kill -TERM "\$(cat "\$data_dir/runtime/daemon.pid")"
    ;;
  *)
    exit 93
    ;;
esac
SCRIPT

printf '%s\n' '#!/bin/sh' 'exit 0' > "$bin_dir/homie-runtime-holder"
printf '%s\n' '#!/bin/sh' 'exit 0' > "$app_bin_dir/Homie"
chmod 0755 \
  "$app_bin_dir/Homie" \
  "$bin_dir/homie" \
  "$bin_dir/homie-runtime-daemon" \
  "$bin_dir/homie-runtime-holder"

HOME="$fixture_root/no-home" PATH="/usr/bin:/bin:/usr/sbin:/sbin" \
  sh "$smoke_script" "$app_path"

daemon_data="$(sed -n 's/^daemon_data=//p' "$command_log")"
cli_data="$(sed -n 's/^cli_data=//p' "$command_log")"
case "$daemon_data" in
  /*) ;;
  *)
    printf 'daemon did not receive an absolute data directory: %s\n' "$daemon_data" >&2
    exit 1
    ;;
esac
if [ "$cli_data" != "$(printf '%s\n%s' "$daemon_data" "$daemon_data")" ]; then
  printf 'daemon and CLI did not share one data directory\ndaemon: %s\nCLI:\n%s\n' \
    "$daemon_data" "$cli_data" >&2
  exit 1
fi
if ! grep -F '"method":"state.snapshot"' "$command_log" >/dev/null; then
  echo 'packaged smoke omitted state.snapshot' >&2
  exit 1
fi
if ! grep -F '"method":"daemon.shutdown"' "$command_log" >/dev/null; then
  echo 'packaged smoke did not prove control of the launched daemon' >&2
  exit 1
fi

echo 'packaged runtime smoke shell test passed'
