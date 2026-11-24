# Status: Detail ladder and world textures

Owner thread: "Build detail ladder and world textures" (split from "Build photoreal renderer", 24 Nov 2026). Covers: VP6 (the detail ladder) and VP3 (texture ladder and relief normals) of `handover/design/SSE4-visual-polish-design.md`, accepted by Alex 24 Nov 2026 18:57 UTC.

Only the owner thread edits this file. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

Files: `packages/renderer/src/ladder.ts`, `ladder-noise.ts`, `ladder-node.ts`, `ladder-layers.json` (each area thread edits its own layer's entry), `texture-ladder.ts`, `texture-ladder-gpu.ts` (VP3), their tests `packages/renderer/test/ladder.test.ts`, `tests/ui/ladder-gpu.e2e.mjs` and `tests/ui/ladder-gpu/`. `gpu.ts` is shared and additive-only. `probe.ts`, `recipes.ts` and `governor.ts` belong to "Build quality tiers and speed probe".

| Package | State | Log |
|---|---|---|
| VP6 the detail ladder: rung table by screen size and tier, bindings to kernel values (missing value, nothing drawn), seed and clock, anchors folded on the processor, noises written once for processor and GPU (simplex with turning gradients, curl, cells, ridged, flow), TSL node, HUD detail line, six starter declarations | done | packages/VP6.md |
| VP3 part 1: texture ladder and relief normals: cube faces 64 to 2048 px by screen size, baked on the GPU from the kernel's tiles at any levels, relief normals at true scale, LRU by budget; System True wears it | done | packages/VP3.md |
| VP3 part 2: read finer tiles (levels 1 and 2, facing first) for the chosen world, a request at a time (touches the chart room's `SystemChart.svelte`) | next (asked the chart room) | |

No kernel change and no golden change.
