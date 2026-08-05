#!/bin/sh
set -eu

version="${HOMIE_VERSION:-0.1.0}"
target_triple="$(rustc -vV | awk '/host:/ { print $2 }')"
dist_dir="${HOMIE_DIST_DIR:-dist}"
stage_dir="$dist_dir/homie-$version-$target_triple"
archive="$dist_dir/homie-$version-$target_triple.tar.gz"

rm -rf "$stage_dir"
mkdir -p "$stage_dir/bin"

cargo build --release -p homie-cli
cp target/release/homie "$stage_dir/bin/homie"
chmod +x "$stage_dir/bin/homie"
cp README.md "$stage_dir/README.md"
cp LICENSE "$stage_dir/LICENSE"

tar -czf "$archive" -C "$dist_dir" "homie-$version-$target_triple"
echo "$archive"
