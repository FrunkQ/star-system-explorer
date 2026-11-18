#!/usr/bin/env bash
# Build the GM shell into packages/shell/dist (static files; any host).
# The kernel runs in a Web Worker from dist/kernel/: the same worker and wasm
# the cross-device harness uses.
set -euo pipefail
cd "$(dirname "$0")/.."
[ -f packages/kernel-client/wasm/sse_wasm_bg.wasm ] || scripts/build-wasm.sh
[ -d node_modules/svelte ] || npm install --no-audit --no-fund
npx tsc -p packages/kernel-client
rm -rf packages/shell/public/kernel
mkdir -p packages/shell/public/kernel
cp packages/kernel-client/dist/kernel-client/src/*.js packages/shell/public/kernel/
cp packages/kernel-client/wasm/sse_wasm.js packages/kernel-client/wasm/sse_wasm_bg.wasm packages/shell/public/kernel/
rm -rf packages/shell/public/packs && mkdir -p packages/shell/public/packs
cp packs/base-physics/pack.json packages/shell/public/packs/base-physics.json
npx vite build --config packages/shell/vite.config.ts --logLevel warn
echo "shell ready in packages/shell/dist/ (serve it over http, e.g. npm run shell)"
