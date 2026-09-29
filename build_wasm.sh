#!/usr/bin/env bash
# Builds both wasm variants of the visualizer:
#   1. raw (no wasm-bindgen) -> www/wasm_visualizer.wasm   (index.html, tree.html)
#   2. bindgen               -> www/pkg/                   (bindgen.html)
# Then runs the Node smoke test.
set -euo pipefail
cd "$(dirname "$0")"

TOOLS=".tools/wasm-bindgen-0.2.100-x86_64-pc-windows-msvc/wasm-bindgen.exe"
if [[ -x "$TOOLS" ]]; then
  WASM_BINDGEN="$TOOLS"
else
  WASM_BINDGEN="$(command -v wasm-bindgen || true)"
  if [[ -z "$WASM_BINDGEN" ]]; then
    echo "Install wasm-bindgen-cli 0.2.100 or place it at $TOOLS" >&2
    exit 1
  fi
fi

echo "==> raw build (no wasm-bindgen)"
cargo build --release --target wasm32-unknown-unknown -p wasm-visualizer --no-default-features
cp target/wasm32-unknown-unknown/release/wasm_visualizer.wasm wasm-visualizer/www/wasm_visualizer.wasm

echo "==> bindgen build + post-process"
cargo build --release --target wasm32-unknown-unknown -p wasm-visualizer
"$WASM_BINDGEN" --target web --out-dir wasm-visualizer/www/pkg \
  target/wasm32-unknown-unknown/release/wasm_visualizer.wasm

echo "==> smoke test"
(cd wasm-visualizer/www && node verify.mjs)
echo "==> done. Serve with: node wasm-visualizer/www/server.js"
