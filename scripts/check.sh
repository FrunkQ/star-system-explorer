#!/usr/bin/env bash
# Everything CI runs for Rust, in one go. Run before every commit.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo fmt --all
cargo clippy --workspace --all-targets -q -- -D warnings
cargo test --workspace -q
# The importer must still build without formation (the converter module).
cargo build -q -p sse-convert-wasm
echo "all checks passed"
