#!/usr/bin/env bash
# Build the V3 importer as the converter's browser module (sse-convert-wasm):
# the importer alone, without formation. Output in target/convert-wasm/ with
# a PIN file (SSE4 commit and sha256 of each file) for the converter to vendor.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build -p sse-convert-wasm --release --target wasm32-unknown-unknown
out=target/convert-wasm
rm -rf "$out" && mkdir -p "$out"
wasm-bindgen --target web --out-dir "$out" target/wasm32-unknown-unknown/release/sse_convert_wasm.wasm
{
  echo "sse4_commit $(git rev-parse HEAD)$(git diff --quiet HEAD -- crates || echo '-dirty')"
  (cd "$out" && sha256sum sse_convert_wasm_bg.wasm sse_convert_wasm.js)
} > "$out/PIN"
cat "$out/PIN"
ls -l "$out"/sse_convert_wasm_bg.wasm
