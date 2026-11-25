---
name: sse4-detail-ladder
description: SSE4 track VP6 (detail ladder) and VP3 (texture ladder, relief) — thread "Build detail ladder and world textures", files, call shape, state as of 24 Nov 2026
metadata:
  type: project
  modified: 2026-11-24T20:49:41.851Z
---
- Thread "Build detail ladder and world textures" (split from the renderer, 24 Nov 2026). Status file handover/status/detail-ladder.md; logs handover/packages/VP6.md, VP3.md. VP6 + VP3 parts 1 and 2 merged in PR 467 (25 Nov 00:40 UTC).
- Files owned: packages/renderer/src/ladder.ts, ladder-noise.ts, ladder-node.ts, ladder-layers.json (area threads edit their own entry), texture-ladder.ts, texture-ladder-gpu.ts, packages/views/body/src/finer-tiles.ts; tests ladder.test.ts, texture-ladder.test.ts, tests/ui/finer-tiles.test.ts, tests/ui/ladder-gpu.e2e.mjs (+ tests/ui/ladder-gpu/), added to ci.yml ui job.
- Call shape relayed to giants/plumes/rings builders: resolveLayer(ladderLayer(id), facts) → rungs(layer, discScale(...), tier, t_s, ladderSeed(address), anchor) → ladderNode(kind, localMetres, {normal, along}).set(rungs, {axis, stretch}); detailLine for HUD. Facts may be per-latitude arrays (climate's `jets`); profileShare gives per-band amplitude.
- Noise domain wraps at 867 (=3×289): simplex's skewed lattice does not repeat at 289 along an axis.
- Headless Chromium here: WebGPU renders and reads back but screenshots come out blank; take look screenshots on WebGL2 (--disable-features=WebGPU). Only chromium is installed in the cloud container (no firefox) for local e2e.
- Kernel tile cost (sample Sol): ~0.2 s per tile. Finer reads agreed with the chart room: 4 tiles a request, 1.2 s settle, pause before level 2.
- Since main's TA1 (PR 444) True faces come from readTrueFace with clouds painted on; a painted face has no `faces`, so cloudy worlds show no relief yet (next: clouds as own shell, VP3 part 2b). VP3 part 3 (Body screen, card globe) waits on PR 428 (shared surface cache views/body/src/surface.ts).
- Alex 24 Nov 20:43: usage reset; test locally (full ci set for your area) and push in bigger chunks, not while CI runs.
- Brown Moon (25 Nov): True ground = typed palette (packs/base-surface colours); sample Sol tile `albedo` is a flat 300 placeholder, so never take brightness from it. Built in PR 516 (VP3 part 5): renderer ground-optics.ts reads data sheet `optical.reflectance` rows (PR 512, merged) named in renderer ground-rows.json (base-surface edit would change the golden), log-blends fresh to mature by tile `maturity` (proven core SW1; present only on worlds with no air), sets tile `ground_rgb`, which ui-contract sampleColour uses. Dormant until SW1 merges; then check the real Moon and show Alex.
- Deep patch: mix the patch in only where weight > 0 (NaN x 0 = NaN blacked out every world once); ladder-gpu e2e checks a lit, unpatched globe.

Related: [[sse4-visual-polish]], [[sse4-renderer-track]].
- 25 Nov 12:30 UTC, MATCH1 (renderer photo matches): the tan on every airless world was ui-contract MATERIAL_RGB. PR 572 has bare ground read the sheet on every world (n+ik rows via Hapke too, ice→h2o, no maturity→mature). Still upstream: icy worlds' tiles are rock and albedo is a flat 300 (proven core package after 417), Mars and Pluto stains wait on 417, Io's sulphur has no optics (plumes thread).
- 25 Nov 14:00 UTC: PR 572 merged (sheet colour on every world). PR 589 open: natural outlines (ui-contract globe.ts sampleNatural, softenGround, edgeNoise, naturalEdge, naturalDirection; keyed on place never time; the time-lapse thread calls them) plus smooth coarse tiles in True. A local commit after 589 makes the Land map use withGround. With SW1 on main, Moon and Mercury albedo is about 165 everywhere (all mature, no fresh young craters); asked the proven core.
