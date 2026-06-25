#!/usr/bin/env bash
set -euo pipefail

file="${1:?usage: read.sh <file> [start_line] [line_count]}"
start="${2:-1}"
count="${3:-160}"

if [[ "$file" = /* || "$file" == *".."* ]]; then
  echo "refusing unsafe path: $file" >&2
  exit 1
fi

sed -n "${start},$((start + count - 1))p" "$file"

