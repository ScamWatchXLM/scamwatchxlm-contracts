#!/usr/bin/env bash
# Installs the toolchain needed to build and deploy the ScamWatchXLM
# contracts: the wasm32v1-none Rust target and the Stellar CLI.
set -euo pipefail

command -v rustup >/dev/null 2>&1 || {
  echo "rustup not found. Install it first: https://rustup.rs" >&2
  exit 1
}

echo "==> Installing wasm32v1-none target"
rustup target add wasm32v1-none

echo "==> Installing rustfmt + clippy"
rustup component add rustfmt clippy

if command -v stellar >/dev/null 2>&1; then
  echo "==> stellar-cli already installed: $(stellar --version)"
else
  echo "==> Installing stellar-cli (this can take a few minutes)"
  cargo install --locked stellar-cli --features opt
fi

echo "==> Done. Try: scripts/build.sh"
