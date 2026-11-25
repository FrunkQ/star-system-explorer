---
name: sse4-light-track-local-ci
description: How the light thread runs the full CI set locally before one push (Alex 24 Nov: test locally, push in bigger chunks), and the disk trap
metadata:
  type: project
  modified: 2026-11-25T01:06:50.897Z
---

Alex, 24 Nov 2026 20:43 UTC (relayed): usage reset; get round the GitHub runner queue by testing locally and pushing bigger chunks. Run the same steps ci.yml runs for your area before every push, push once, don't push again while CI runs unless a check fails.

How it worked for PR 406 (merged 25 Nov 01:06 UTC, all green first time):
- `cargo install wasm-bindgen-cli --version 0.2.130 --locked` and `npm ci` once.
- Sequence: scripts/check.sh; git diff --exit-code packages/contracts/src/generated; build-base-reference.py --check; `cargo run -q -p sse-cli --release -- harness`; scripts/build-harness.sh; npm run typecheck; node tests/harness/run-node.mjs; npm test; scripts/build-wasm.sh; npm run ui:check; npm run ui:test; scripts/build-shell.sh; the e2e files for the area (light, render) in chromium. About 45 minutes on 4 cores.
- Set `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`: with debug info the target dir reached 22 GB and filled the disk (check.sh then hangs silently). Remove old worktrees' target dirs.
- Coordinator ruling (24 Nov 23:11): never merge with a red check, even when the failure is main's own; wait for main's fix, merge main, push again. Cancel a run that cannot lead to a merge, to free runners.

Related: [[sse4-rulings-24nov-evening]], [[ui-checks-before-commit]]
