#!/usr/bin/env bash
set -euo pipefail

root="${1:-$(pwd)}"
cd "$root"

script_dir=".octopus/harness/bash-only/scripts"
mkdir -p "$script_dir"
script="$script_dir/run-$(date +%Y%m%d%H%M%S).sh"

cat > "$script"
chmod +x "$script"
bash "$script"
echo "executed $script"

