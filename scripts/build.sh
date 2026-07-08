#!/usr/bin/env bash
# Builds all three contracts to release Wasm.
#
# Prefers `stellar contract build` (from stellar-cli), which applies the
# exact build settings the Soroban runtime requires and embeds contract
# metadata. Falls back to a plain `cargo build --release --target
# wasm32v1-none` per contract if stellar-cli isn't installed (see
# scripts/setup.sh) -- fine for local iteration, but the stellar-cli path is
# recommended before deploying anywhere.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

contracts=(governance registry reputation)
out_dir="target/contracts"
mkdir -p "$out_dir"

if command -v stellar >/dev/null 2>&1; then
  for name in "${contracts[@]}"; do
    echo "==> stellar contract build: $name"
    stellar contract build --package "scamwatchxlm-$name"
    cp "target/wasm32v1-none/release/$name.wasm" "$out_dir/$name.wasm"
  done
else
  echo "stellar-cli not found; falling back to cargo build (run scripts/setup.sh to install it)." >&2
  cargo build --release --target wasm32v1-none \
    -p scamwatchxlm-governance -p scamwatchxlm-registry -p scamwatchxlm-reputation
  for name in "${contracts[@]}"; do
    cp "target/wasm32v1-none/release/$name.wasm" "$out_dir/$name.wasm"
  done
fi

echo "==> Built:"
ls -lh "$out_dir"/*.wasm
