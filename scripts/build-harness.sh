#!/usr/bin/env bash
# Build the cross-device hash harness site into tests/harness/ (pkg/ is generated).
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build -p sse-wasm --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir tests/harness/pkg target/wasm32-unknown-unknown/release/sse_wasm.wasm
cp tests/golden/harness.json tests/harness/pkg/golden.json
echo "harness site ready in tests/harness/ (serve that folder over http)"
