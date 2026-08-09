#!/bin/sh
set -eu

lock_file="${1:-docs/research/diri-parity-lock.md}"

if [ ! -f "$lock_file" ]; then
  echo "missing parity lock: $lock_file" >&2
  exit 1
fi

invalid_statuses=$(awk -F'|' '
  /^\| [A-Z]+-[0-9]+ / {
    status=$6
    gsub(/^[ \t]+|[ \t]+$/, "", status)
    if (status != "implemented" && status != "partial" && status != "missing" && status != "blocked") {
      print $2 " has invalid status: " status
    }
  }
' "$lock_file")

if [ -n "$invalid_statuses" ]; then
  echo "$invalid_statuses" >&2
  exit 1
fi

remaining=$(awk -F'|' '
  /^\| [A-Z]+-[0-9]+ / {
    id=$2
    status=$6
    gsub(/^[ \t]+|[ \t]+$/, "", id)
    gsub(/^[ \t]+|[ \t]+$/, "", status)
    if (status != "implemented") {
      print id ": " status
    }
  }
' "$lock_file")

if grep -Eq '^diri_parity:[[:space:]]*complete[[:space:]]*$' "$lock_file" && [ -n "$remaining" ]; then
  echo "lock file claims Diri parity complete while rows remain incomplete:" >&2
  echo "$remaining" >&2
  exit 1
fi

echo "Diri parity lock valid."
if [ -n "$remaining" ]; then
  echo "Incomplete rows remain:"
  echo "$remaining"
fi
