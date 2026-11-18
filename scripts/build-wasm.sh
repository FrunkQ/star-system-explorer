#!/usr/bin/env bash
# Build the kernel to WebAssembly and generate its JS glue for the client.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build -p sse-wasm --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir packages/kernel-client/wasm target/wasm32-unknown-unknown/release/sse_wasm.wasm
