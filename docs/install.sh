#!/usr/bin/env sh
set -eu

repo="https://github.com/dangoZhang/Octopus"
package="octopus-core"
binary="octopus"

if ! command -v cargo >/dev/null 2>&1; then
  printf '%s\n' "cargo is required to install Octopus."
  printf '%s\n' "Install Rust from https://rustup.rs, then run this script again."
  exit 1
fi

cargo install --git "$repo" "$package" --locked --bin "$binary" --force

printf '\n%s\n' "Octopus installed."
printf '%s\n' "Run: octopus download"
printf '%s\n' "Run: octopus start --open"
