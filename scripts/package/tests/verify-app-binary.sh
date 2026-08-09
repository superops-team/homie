#!/bin/sh
set -eu

app_path="${1:?usage: verify-app-binary.sh </absolute/path/to/Homie.app>}"
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

app_binary="$app_path/Contents/MacOS/Homie"
cli_binary="$app_path/Contents/Resources/bin/homie"
daemon_binary="$app_path/Contents/Resources/bin/homie-runtime-daemon"
holder_binary="$app_path/Contents/Resources/bin/homie-runtime-holder"

mode_of() {
  if stat -f '%Lp' "$1" >/dev/null 2>&1; then
    stat -f '%Lp' "$1"
  else
    stat -c '%a' "$1"
  fi
}

for binary in "$app_binary" "$cli_binary" "$daemon_binary" "$holder_binary"; do
  if [ ! -f "$binary" ] || [ -L "$binary" ]; then
    echo "required bundle binary is missing or is a symlink: $binary" >&2
    exit 1
  fi
  if [ ! -x "$binary" ]; then
    echo "required bundle binary is not executable: $binary" >&2
    exit 1
  fi
  mode="$(mode_of "$binary")"
  if [ "$mode" != "755" ]; then
    echo "required bundle binary mode must be 0755, got $mode: $binary" >&2
    exit 1
  fi
done

if ! command -v codesign >/dev/null 2>&1; then
  echo "codesign is required to verify the macOS app closure" >&2
  exit 1
fi
codesign --verify --strict "$cli_binary"
codesign --verify --strict "$daemon_binary"
codesign --verify --strict "$holder_binary"
codesign --verify --strict "$app_binary"
codesign --verify --strict --deep "$app_path"

echo "verified packaged app closure: $app_path"
