#!/bin/sh
set -eu

repo_root="$(CDPATH= cd -- "$(dirname "$0")/../../.." && pwd -P)"
package_script="$repo_root/scripts/package/package.sh"
verify_script="$repo_root/scripts/package/tests/verify-app-binary.sh"
fixture_root="$(mktemp -d /tmp/homie-package-closure-test.XXXXXX)"
trap 'rm -rf "$fixture_root"' EXIT

tool_dir="$fixture_root/tools"
target_dir="$fixture_root/target"
dist_dir="$fixture_root/dist"
command_log="$fixture_root/commands.log"
mkdir -p "$tool_dir" "$target_dir/release"
: > "$command_log"

cat > "$tool_dir/rustc" <<'SCRIPT'
#!/bin/sh
printf '%s\n' 'rustc 1.95.0 (fixture)'
printf '%s\n' 'host: aarch64-apple-darwin'
SCRIPT

cat > "$tool_dir/cargo" <<'SCRIPT'
#!/bin/sh
set -eu
printf 'cargo %s\n' "$*" >> "$PACKAGE_TEST_COMMAND_LOG"
mkdir -p "$CARGO_TARGET_DIR/release"
for binary in homie homie-app homie-runtime-daemon homie-runtime-holder; do
  printf '%s\n' '#!/bin/sh' 'exit 0' > "$CARGO_TARGET_DIR/release/$binary"
  chmod 0755 "$CARGO_TARGET_DIR/release/$binary"
done
SCRIPT

cat > "$tool_dir/codesign" <<'SCRIPT'
#!/bin/sh
printf 'codesign %s\n' "$*" >> "$PACKAGE_TEST_COMMAND_LOG"
SCRIPT

cat > "$tool_dir/xattr" <<'SCRIPT'
#!/bin/sh
printf 'xattr %s\n' "$*" >> "$PACKAGE_TEST_COMMAND_LOG"
SCRIPT

chmod 0755 "$tool_dir/rustc" "$tool_dir/cargo" "$tool_dir/codesign" "$tool_dir/xattr"

package_output="$(
  CARGO_TARGET_DIR="$target_dir" \
  HOMIE_DIST_DIR="$dist_dir" \
  PACKAGE_TEST_COMMAND_LOG="$command_log" \
  PATH="$tool_dir:/usr/bin:/bin" \
    sh "$package_script"
)"
app_path="$(printf '%s\n' "$package_output" | sed -n 's/^APP_PATH=//p')"

case "$app_path" in
  /*) ;;
  *)
    printf 'package APP_PATH must be absolute: %s\n' "$app_path" >&2
    exit 1
    ;;
esac

app_binary="$app_path/Contents/MacOS/Homie"
cli_binary="$app_path/Contents/Resources/bin/homie"
daemon_binary="$app_path/Contents/Resources/bin/homie-runtime-daemon"
holder_binary="$app_path/Contents/Resources/bin/homie-runtime-holder"
standalone_bin_dir="$(dirname "$app_path")/bin"
standalone_cli="$standalone_bin_dir/homie"
standalone_daemon="$standalone_bin_dir/homie-runtime-daemon"
standalone_holder="$standalone_bin_dir/homie-runtime-holder"

for binary in \
  "$app_binary" \
  "$cli_binary" \
  "$daemon_binary" \
  "$holder_binary" \
  "$standalone_cli" \
  "$standalone_daemon" \
  "$standalone_holder"
do
  if [ ! -x "$binary" ]; then
    printf 'package closure binary missing or not executable: %s\n' "$binary" >&2
    exit 1
  fi
done

if ! grep -F -- '--bin homie-runtime-daemon' "$command_log" >/dev/null; then
  echo 'release build omitted homie-runtime-daemon' >&2
  exit 1
fi
if ! grep -F -- '--bin homie-runtime-holder' "$command_log" >/dev/null; then
  echo 'release build omitted homie-runtime-holder' >&2
  exit 1
fi

actual_sign_order="$(
  awk '
    /^codesign / && / --force / {
      print $NF
    }
  ' "$command_log"
)"
expected_sign_order="$(printf '%s\n' \
  "$cli_binary" \
  "$daemon_binary" \
  "$holder_binary" \
  "$app_binary" \
  "$app_path")"
if [ "$actual_sign_order" != "$expected_sign_order" ]; then
  printf 'unexpected nested signing order\nexpected:\n%s\nactual:\n%s\n' \
    "$expected_sign_order" "$actual_sign_order" >&2
  exit 1
fi

PACKAGE_TEST_COMMAND_LOG="$command_log" PATH="$tool_dir:/usr/bin:/bin" \
  sh "$verify_script" "$app_path"

chmod 0644 "$daemon_binary"
if PACKAGE_TEST_COMMAND_LOG="$command_log" PATH="$tool_dir:/usr/bin:/bin" \
  sh "$verify_script" "$app_path" >/dev/null 2>&1
then
  echo 'verification accepted a non-executable daemon' >&2
  exit 1
fi
chmod 0755 "$daemon_binary"

mv "$daemon_binary" "$app_path/Contents/Resources/homie-runtime-daemon"
if PACKAGE_TEST_COMMAND_LOG="$command_log" PATH="$tool_dir:/usr/bin:/bin" \
  sh "$verify_script" "$app_path" >/dev/null 2>&1
then
  echo 'verification accepted a daemon outside the fixed bundle path' >&2
  exit 1
fi

echo 'package closure shell test passed'
