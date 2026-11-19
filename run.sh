#!/usr/bin/env bash
# Start the Rust engine and the browser viewer together.
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p runtime
if [ ! -x runtime/game ] || [ src/main.rs -nt runtime/game ]; then
  ./build.sh
fi
./runtime/game >/tmp/rustcraft-engine.log 2>&1 &
ENGINE=$!
trap 'kill $ENGINE 2>/dev/null || true' EXIT
echo "RustCraft running at http://localhost:3000"
node web/server.js
