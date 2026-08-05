#!/bin/sh
set -eu

version="${HOMIE_VERSION:-0.1.0}"
target_triple="$(rustc -vV | awk '/host:/ { print $2 }')"
dist_dir="${HOMIE_DIST_DIR:-dist}"
stage_dir="$dist_dir/homie-$version-$target_triple"
archive="$dist_dir/homie-$version-$target_triple.tar.gz"
app_dir="$stage_dir/Homie.app"

rm -rf "$stage_dir"
mkdir -p "$stage_dir/bin" "$app_dir/Contents/MacOS" "$app_dir/Contents/Resources/bin"

cargo build --release -p homie-cli
cp target/release/homie "$stage_dir/bin/homie"
chmod +x "$stage_dir/bin/homie"
cp target/release/homie "$app_dir/Contents/Resources/bin/homie"
chmod +x "$app_dir/Contents/Resources/bin/homie"
cp README.md "$stage_dir/README.md"
cp LICENSE "$stage_dir/LICENSE"
cp README.md "$app_dir/Contents/Resources/README.md"
cp LICENSE "$app_dir/Contents/Resources/LICENSE"

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

cat > "$app_dir/Contents/MacOS/Homie" <<'APP'
#!/bin/sh
set -eu

resources_dir="$(cd "$(dirname "$0")/../Resources" && pwd)"
homie_bin="$resources_dir/bin/homie"
data_dir="${HOME}/Library/Application Support/Homie"

output="$("$homie_bin" doctor --data-dir "$data_dir" --json 2>&1)" || {
  /usr/bin/osascript -e 'display dialog "Homie failed to start. Run bin/homie doctor in Terminal for details." buttons {"OK"} default button "OK" with icon stop' >/dev/null 2>&1 || true
  exit 1
}

/usr/bin/osascript <<OSA >/dev/null 2>&1 || true
display dialog "Homie local V1 is ready.\n\nStorage initialized at:\n$data_dir\n\n$output" buttons {"OK"} default button "OK" with title "Homie" with icon note
OSA
APP
chmod +x "$app_dir/Contents/MacOS/Homie"

tar -czf "$archive" -C "$dist_dir" "homie-$version-$target_triple"
echo "$archive"
