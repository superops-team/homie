#!/bin/sh
set -eu

prefix=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --prefix)
      shift
      prefix="${1:-}"
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
  shift
done

if [ -z "$prefix" ]; then
  echo "usage: scripts/dev/install-local.sh --prefix <path>" >&2
  exit 2
fi

cargo build --release -p homie-cli
mkdir -p "$prefix/bin"
cp target/release/homie "$prefix/bin/homie"
chmod +x "$prefix/bin/homie"

echo "$prefix/bin/homie"
