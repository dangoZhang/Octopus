#!/usr/bin/env bash
set -euo pipefail

file="${1:?usage: edit.sh <file> <old> <new>}"
old="${2:?missing old text}"
new="${3:?missing new text}"

if [[ "$file" = /* || "$file" == *".."* ]]; then
  echo "refusing unsafe path: $file" >&2
  exit 1
fi

python3 - "$file" "$old" "$new" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
old = sys.argv[2]
new = sys.argv[3]
text = path.read_text()
if old not in text:
    raise SystemExit(f"old text not found in {path}")
path.write_text(text.replace(old, new, 1))
PY

echo "edited $file"

