#!/usr/bin/env bash
# Build the cross-device check site into tests/harness/ (pkg/ is generated):
# hash harness (direct wasm) plus the kernel contract run through a Web Worker.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build -p sse-wasm --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir tests/harness/pkg target/wasm32-unknown-unknown/release/sse_wasm.wasm
mkdir -p packages/kernel-client/wasm
cp tests/harness/pkg/sse_wasm* packages/kernel-client/wasm/
[ -d node_modules/typescript ] || npm install --no-audit --no-fund
npx tsc -p packages/kernel-client
rm -rf tests/harness/pkg/client tests/harness/pkg/fixtures
mkdir -p tests/harness/pkg/client tests/harness/pkg/fixtures/packs
cp packages/kernel-client/dist/kernel-client/src/*.js tests/harness/pkg/client/
cp tests/fixtures/packs/*.json tests/harness/pkg/fixtures/packs/
cp tests/contract/requests.json tests/harness/pkg/fixtures/requests.json
cp tests/golden/harness.json tests/harness/pkg/golden.json
cp tests/golden/contract.jsonl tests/harness/pkg/contract.jsonl
python3 -c 'import json; print(json.dumps({k: json.load(open(f"tests/golden/{k}.json")) for k in ["resolve", "sift", "observe", "suite", "cosmos", "formation", "formation-early", "formation-accrete", "formation-accrete-dole", "formation-growth", "formation-upheaval", "formation-leftovers", "formation-pipeline", "classify", "life", "place", "time", "time-generated", "surface", "surface-atlas", "surface-plates", "surface-wear", "cosmology", "haloes", "galaxy", "envelope", "life-budget", "life-history", "life-biosphere", "stars", "life-signatures", "life-feedback"]}))' > tests/harness/pkg/engines.json
ls tests/fixtures/packs/*.json | xargs -n1 basename | sed 's/\.json$//' | python3 -c 'import sys,json; print(json.dumps([l.strip() for l in sys.stdin]))' > tests/harness/pkg/fixtures/packs.json
echo "site ready in tests/harness/ (serve that folder over http, e.g. npm run harness)"
