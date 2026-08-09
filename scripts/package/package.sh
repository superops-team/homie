#!/bin/sh
set -eu

repo_root="$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)"
version="${HOMIE_VERSION:-0.1.0}"
target_triple="$(rustc -vV | awk '/host:/ { print $2 }')"
dist_dir="${HOMIE_DIST_DIR:-dist}"
target_dir="${CARGO_TARGET_DIR:-$repo_root/target}"
case "$dist_dir" in
  /*) ;;
  *) dist_dir="$repo_root/$dist_dir" ;;
esac
case "$target_dir" in
  /*) ;;
  *) target_dir="$repo_root/$target_dir" ;;
esac
stage_dir="$dist_dir/homie-$version-$target_triple"
archive="$dist_dir/homie-$version-$target_triple.tar.gz"
app_dir="$stage_dir/Homie.app"

rm -rf "$stage_dir"
mkdir -p "$stage_dir/bin" "$app_dir/Contents/MacOS" "$app_dir/Contents/Resources/bin"

CARGO_TARGET_DIR="$target_dir" cargo build \
  --manifest-path "$repo_root/Cargo.toml" \
  --release \
  -p homie-cli --bin homie \
  -p homie-app --bin homie-app \
  -p homie-runtime --bin homie-runtime-daemon --bin homie-runtime-holder

cp "$target_dir/release/homie" "$stage_dir/bin/homie"
cp "$target_dir/release/homie-runtime-daemon" "$stage_dir/bin/homie-runtime-daemon"
cp "$target_dir/release/homie-runtime-holder" "$stage_dir/bin/homie-runtime-holder"
cp "$target_dir/release/homie" "$app_dir/Contents/Resources/bin/homie"
cp "$target_dir/release/homie-runtime-daemon" \
  "$app_dir/Contents/Resources/bin/homie-runtime-daemon"
cp "$target_dir/release/homie-runtime-holder" \
  "$app_dir/Contents/Resources/bin/homie-runtime-holder"
cp "$target_dir/release/homie-app" "$app_dir/Contents/MacOS/Homie"
chmod 0755 \
  "$stage_dir/bin/homie" \
  "$stage_dir/bin/homie-runtime-daemon" \
  "$stage_dir/bin/homie-runtime-holder" \
  "$app_dir/Contents/MacOS/Homie" \
  "$app_dir/Contents/Resources/bin/homie" \
  "$app_dir/Contents/Resources/bin/homie-runtime-daemon" \
  "$app_dir/Contents/Resources/bin/homie-runtime-holder"
cp "$repo_root/README.md" "$stage_dir/README.md"
cp "$repo_root/LICENSE" "$stage_dir/LICENSE"
cp "$repo_root/README.md" "$app_dir/Contents/Resources/README.md"
cp "$repo_root/LICENSE" "$app_dir/Contents/Resources/LICENSE"

cat > "$app_dir/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>Homie</string>
  <key>CFBundleIdentifier</key>
  <string>com.superops.homie</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>Homie</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>$version</string>
  <key>CFBundleVersion</key>
  <string>$version</string>
  <key>LSMinimumSystemVersion</key>
  <string>13.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST
if ! command -v codesign >/dev/null 2>&1; then
  echo "package: codesign is required for the macOS app closure" >&2
  exit 1
fi
if command -v xattr >/dev/null 2>&1; then
  xattr -cr "$app_dir" >/dev/null 2>&1 || true
fi

codesign --force --sign - "$app_dir/Contents/Resources/bin/homie" >/dev/null
codesign --force --sign - \
  "$app_dir/Contents/Resources/bin/homie-runtime-daemon" >/dev/null
codesign --force --sign - \
  "$app_dir/Contents/Resources/bin/homie-runtime-holder" >/dev/null
codesign --force --sign - "$app_dir/Contents/MacOS/Homie" >/dev/null
codesign --force --sign - "$app_dir" >/dev/null
codesign --verify --strict --deep "$app_dir" >/dev/null

tar -czf "$archive" -C "$dist_dir" "homie-$version-$target_triple"
echo "APP_PATH=$app_dir"
echo "DAEMON_PATH=$app_dir/Contents/Resources/bin/homie-runtime-daemon"
echo "TARBALL_PATH=$archive"
