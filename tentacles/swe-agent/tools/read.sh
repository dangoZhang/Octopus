#!/usr/bin/env bash
set -euo pipefail

file="${1:?usage: read.sh <file> [start_line] [line_count]}"
start="${2:-1}"
count="${3:-160}"

if [[ "$file" = /* || "$file" == *".."* ]]; then
  echo "refusing unsafe path: $file" >&2
  exit 1
fi

if [[ ! "$start" =~ ^[0-9]+$ ]]; then
  echo "invalid start line: $start" >&2
  exit 1
fi

if ((start < 1)); then
  echo "invalid start line: $start" >&2
  exit 1
fi

if [[ ! "$count" =~ ^[0-9]+$ ]]; then
  echo "invalid line count: $count" >&2
  exit 1
fi

if ((count < 1 || count > 400)); then
  echo "invalid line count: $count" >&2
  exit 1
fi

if [[ ! -f "$file" ]]; then
  echo "file not found: $file" >&2
  exit 1
fi

end=$((start + count - 1))
echo "== read ${file}:${start}-${end} =="
awk -v start="$start" -v end="$end" 'NR >= start && NR <= end { printf "%6d  %s\n", NR, $0 }' "$file"
