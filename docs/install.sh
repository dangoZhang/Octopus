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

bin_path=""
if command -v "$binary" >/dev/null 2>&1; then
  bin_path="$(command -v "$binary")"
elif [ -n "${CARGO_HOME:-}" ] && [ -x "$CARGO_HOME/bin/$binary" ]; then
  bin_path="$CARGO_HOME/bin/$binary"
elif [ -n "${HOME:-}" ] && [ -x "$HOME/.cargo/bin/$binary" ]; then
  bin_path="$HOME/.cargo/bin/$binary"
fi

printf '\n%s\n' "Octopus installed."
if [ -n "$bin_path" ]; then
  "$bin_path" --version
else
  printf '%s\n' "Octopus was installed, but '$binary' is not on PATH yet."
  printf '%s\n' "Add Cargo's bin directory to PATH, then run: octopus --version"
fi
printf '%s\n' "Run: octopus download"
printf '%s\n' "Run: octopus start --open"
