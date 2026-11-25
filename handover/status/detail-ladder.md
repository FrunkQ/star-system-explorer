# Status: Detail ladder and world textures

Owner thread: "Build detail ladder and world textures" (split from "Build photoreal renderer", 24 Nov 2026). Covers: VP6 (the detail ladder) and VP3 (texture ladder and relief normals) of `handover/design/SSE4-visual-polish-design.md`, accepted by Alex 24 Nov 2026 18:57 UTC.

Only the owner thread edits this file. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

Files: `packages/views/body/src/finer-tiles.ts` (VP3 part 2), `packages/renderer/src/ladder.ts`, `ladder-noise.ts`, `ladder-node.ts`, `ladder-layers.json` (each area thread edits its own layer's entry), `texture-ladder.ts`, `texture-ladder-gpu.ts` (VP3), their tests `packages/renderer/test/ladder.test.ts`, `tests/ui/ladder-gpu.e2e.mjs` and `tests/ui/ladder-gpu/`. `gpu.ts` is shared and additive-only. `probe.ts`, `recipes.ts` and `governor.ts` belong to "Build quality tiers and speed probe".

| Package | State | Log |
|---|---|---|
| VP6 the detail ladder: rung table by screen size and tier, bindings to kernel values (missing value, nothing drawn), seed and clock, anchors folded on the processor, noises written once for processor and GPU (simplex with turning gradients, curl, cells, ridged, flow), TSL node, HUD detail line, six starter declarations | done | packages/VP6.md |
| VP3 part 1: texture ladder and relief normals: cube faces 64 to 2048 px by screen size, baked on the GPU from the kernel's tiles at any levels, relief normals at true scale, LRU by budget; System True wears it | done | packages/VP3.md |
| VP3 part 2: finer tiles (levels 1 and 2) for the chosen world, 4 tiles a request after a 1.2 s settle, stopping when not wanted, kept for the page in `views/body/src/finer-tiles.ts`; the chart room agreed the edit in `SystemChart.svelte`'s True surface reader | done | packages/VP3.md |
| VP3 part 2b: clouds as their own shell over the baked ground, so worlds with air keep their relief in True (with the air thread, true-air.ts) | next | |
| VP3 part 4: the deep patch under a close camera (renderer's close-up package hands `GroundView {body, toward, m_per_px}` from System3D's `onground`): level from metres a pixel (`patchLevel`), the tiles round the point over cube edges (`tilesNear`, `patchFor`), drawn over the bake, ground roughness from the ladder below the fact floor. Building blocks done: planner, reader (`readDeep`), patch picture, GPU drawing over the bake, checks on both back ends. Next: SystemChart calls `readDeep` from the renderer's `groundView` once close-up.ts lands | in progress | packages/VP3.md |
| VP3 part 3a: giants on the Body globe, the card's globe and map wear their decks (readTrueFace, `decks: true`), one lookup in renderer `globe-face.ts` for both tiers (agreed with the renderer); switches on when TA3a sends `kind` | done | packages/VP3.md |
| VP3 part 5: bare ground in its measured colour: data sheet reflectance rows (MP35) blended fresh to mature by the kernel's space weathering (SW1 `maturity`), laid on as `ground_rgb` for every view (renderer `ground-optics.ts`, `ground-rows.json`) | done, dormant until SW1 sends `maturity` | packages/VP3.md |
| VP3 part 3: the Body screen and the body card's globe read `finerTiles` through the shared surface cache (after PR 428); the detail ladder's ground roughness below the tiles on the globe | next | |

No kernel change and no golden change.
