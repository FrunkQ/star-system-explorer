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
cp packs/base-time/pack.json packages/shell/public/packs/base-time.json
cp packs/base-sol/pack.json packages/shell/public/packs/base-sol.json
cp packs/base-reference/pack.json packages/shell/public/packs/base-reference.json
# The base names pack, offered on the Names page while no pack on the stack carries names (NM6).
cp packs/base-names/pack.json packages/shell/public/packs/base-names.json
cp packs/base-ships/pack.json packages/shell/public/packs/base-ships.json
cp packages/shell/src/documents/kestrel.player-data.pack.json packages/shell/public/packs/kestrel-player-data.json
mkdir -p packages/shell/public/packs/samples
cp packages/shell/src/documents/sample.warm-worlds.pack.json packages/shell/public/packs/samples/warm-worlds.json
# The game pack the sector map and world codes screens read (X3).
cp packs/traveller/pack.json packages/shell/public/packs/samples/traveller.json
cp packs/near-future/pack.json packages/shell/public/packs/samples/near-future.json
cp packs/space-opera/pack.json packages/shell/public/packs/samples/space-opera.json
cp packs/base-hooks/pack.json packages/shell/public/packs/samples/base-hooks.json
cp packs/intrigue/pack.json packages/shell/public/packs/samples/intrigue.json
rm -rf packages/shell/public/samples && mkdir -p packages/shell/public/samples
# The sample save the Make a system dialog offers: the importer fixture with names that say "(sample)", so nobody takes it for the real Sol (Alex, 21 Nov).
cp packages/shell/src/documents/sample-sol.v3.json packages/shell/public/samples/synthetic-sol.json
rm -rf packages/shell/public/scenes
# Documents the screens composer edits and previews (U5).
rm -rf packages/shell/public/documents && mkdir -p packages/shell/public/documents
cp packages/shell/src/documents/players.screen.json packages/shell/src/documents/players.survey.screen.json packages/shell/src/documents/players.bridge-3d.screen.json packages/shell/src/documents/vocab.plain.json packages/shell/src/documents/orac.screen-request.json packages/shell/src/documents/reference-objects.json packages/shell/public/documents/
npx vite build --config packages/shell/vite.config.ts --logLevel warn
echo "shell ready in packages/shell/dist/ (serve it over http, e.g. npm run shell)"
