#!/usr/bin/env bash
# Build the deployable static site into dist/ (Cloudflare Workers Builds runs
# this as `npm run build`; `npx wrangler deploy` then serves dist/ per
# wrangler.jsonc). Works on a bare Linux build image: installs Rust (the
# pinned toolchain from rust-toolchain.toml) and the prebuilt wasm-bindgen CLI
# when they are missing. Today the site is the cross-device check page; the
# GM shell joins it when the UI track merges.
set -euo pipefail
cd "$(dirname "$0")/.."
WB=0.2.130
if ! command -v cargo >/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none
  export PATH="$HOME/.cargo/bin:$PATH"
fi
if ! command -v wasm-bindgen >/dev/null || [ "$(wasm-bindgen --version | cut -d' ' -f2)" != "$WB" ]; then
  mkdir -p .tools
  curl -sSfL "https://github.com/rustwasm/wasm-bindgen/releases/download/$WB/wasm-bindgen-$WB-x86_64-unknown-linux-musl.tar.gz" \
    | tar -xz -C .tools
  export PATH="$PWD/.tools/wasm-bindgen-$WB-x86_64-unknown-linux-musl:$PATH"
fi
scripts/build-harness.sh
rm -rf dist
mkdir -p dist
cp -r tests/harness/. dist/
rm -f dist/run-node.mjs dist/run-browsers.mjs
echo "site ready in dist/"
