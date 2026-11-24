---
name: sse4-detail-ladder
description: SSE4 track VP6 (detail ladder) and VP3 (texture ladder, relief) — thread "Build detail ladder and world textures", files, call shape, state as of 24 Nov 2026
metadata:
  type: project
  modified: 2026-11-24T20:49:41.851Z
---
- Thread "Build detail ladder and world textures" (split from the renderer, 24 Nov 2026). Status file handover/status/detail-ladder.md; logs handover/packages/VP6.md, VP3.md. VP6 + VP3 parts 1 and 2 all in PR 467 (one branch per thread).
- Files owned: packages/renderer/src/ladder.ts, ladder-noise.ts, ladder-node.ts, ladder-layers.json (area threads edit their own entry), texture-ladder.ts, texture-ladder-gpu.ts, packages/views/body/src/finer-tiles.ts; tests ladder.test.ts, texture-ladder.test.ts, tests/ui/finer-tiles.test.ts, tests/ui/ladder-gpu.e2e.mjs (+ tests/ui/ladder-gpu/), added to ci.yml ui job.
- Call shape relayed to giants/plumes/rings builders: resolveLayer(ladderLayer(id), facts) → rungs(layer, discScale(...), tier, t_s, ladderSeed(address), anchor) → ladderNode(kind, localMetres, {normal, along}).set(rungs, {axis, stretch}); detailLine for HUD. Facts may be per-latitude arrays (climate's `jets`); profileShare gives per-band amplitude.
- Noise domain wraps at 867 (=3×289): simplex's skewed lattice does not repeat at 289 along an axis.
- Headless Chromium here: WebGPU renders and reads back but screenshots come out blank; take look screenshots on WebGL2 (--disable-features=WebGPU). Only chromium is installed in the cloud container (no firefox) for local e2e.
- Kernel tile cost (sample Sol): ~0.2 s per tile. Finer reads agreed with the chart room: 4 tiles a request, 1.2 s settle, pause before level 2.
- Since main's TA1 (PR 444) True faces come from readTrueFace with clouds painted on; a painted face has no `faces`, so cloudy worlds show no relief yet (next: clouds as own shell, VP3 part 2b). VP3 part 3 (Body screen, card globe) waits on PR 428 (shared surface cache views/body/src/surface.ts).
- Alex 24 Nov 20:43: usage reset; test locally (full ci set for your area) and push in bigger chunks, not while CI runs.

Related: [[sse4-visual-polish]], [[sse4-renderer-track]].
