#!/bin/sh
set -eu

exec "$(dirname "$0")/check-diri-parity-lock.sh" "$@"
