#!/usr/bin/env bash
# Everything CI runs for Rust, in one go. Run before every commit.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo fmt --all
cargo clippy --workspace --all-targets -q -- -D warnings
cargo test --workspace -q
echo "all checks passed"
