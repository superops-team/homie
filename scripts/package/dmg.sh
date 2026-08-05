#!/bin/sh
set -eu

version="${HOMIE_VERSION:-0.1.0}"
target_triple="$(rustc -vV | awk '/host:/ { print $2 }')"
dist_dir="${HOMIE_DIST_DIR:-dist}"
stage_dir="$dist_dir/homie-$version-$target_triple"
dmg_path="$dist_dir/homie-$version-$target_triple.dmg"

scripts/package/package.sh >/dev/null

rm -f "$dmg_path"
hdiutil create \
  -volname "Homie $version" \
  -srcfolder "$stage_dir" \
  -ov \
  -format UDZO \
  "$dmg_path" >/dev/null

echo "APP_PATH=$stage_dir/Homie.app"
echo "DMG_PATH=$dmg_path"
