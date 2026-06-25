#!/usr/bin/env bash
set -euo pipefail

root="${1:-$(pwd)}"
cd "$root"

patch_dir=".octopus/harness/swe-agent/patches"
mkdir -p "$patch_dir"
patch_file="$patch_dir/patch-$(date +%Y%m%d%H%M%S).diff"
cat > "$patch_file"

git apply --check "$patch_file"

if [[ "${OCTOPUS_APPLY:-0}" == "1" ]]; then
  git apply "$patch_file"
  echo "applied $patch_file"
else
  echo "validated $patch_file"
  echo "set OCTOPUS_APPLY=1 to apply"
fi

