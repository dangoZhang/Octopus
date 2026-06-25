#!/usr/bin/env bash
set -euo pipefail

root="${1:-$(pwd)}"
cd "$root"

script_dir=".octopus/harness/computer-use-agent/scripts"
mkdir -p "$script_dir"
script="$script_dir/bash-$(date +%Y%m%d%H%M%S).sh"
cat > "$script"
chmod +x "$script"
bash "$script"
echo "executed $script"

