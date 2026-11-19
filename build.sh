#!/usr/bin/env bash
# Build the Rust engine with a plain rustc (no crates or Cargo needed).
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p runtime
RUSTC_BIN="${RUSTC:-$(command -v rustc || true)}"
if [ -z "$RUSTC_BIN" ]; then
  echo "No rustc found. Install Rust or set RUSTC=/path/to/rustc" >&2
  exit 1
fi
"$RUSTC_BIN" --edition 2021 -O -A warnings src/main.rs -o runtime/game
echo "Built runtime/game"
