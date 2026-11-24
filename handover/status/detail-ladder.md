# Status: Detail ladder and world textures

Owner thread: "Build detail ladder and world textures" (split from "Build photoreal renderer", 24 Nov 2026). Covers: VP6 (the detail ladder) and VP3 (texture ladder and relief normals) of `handover/design/SSE4-visual-polish-design.md`, accepted by Alex 24 Nov 2026 18:57 UTC.

Only the owner thread edits this file. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

Files: `packages/renderer/src/ladder.ts`, `ladder-noise.ts`, `ladder-node.ts`, `ladder-layers.json` (each area thread edits its own layer's entry), `texture-ladder.ts` (VP3), their tests `packages/renderer/test/ladder.test.ts`, `tests/ui/ladder-gpu.e2e.mjs` and `tests/ui/ladder-gpu/`. `gpu.ts` is shared and additive-only. `probe.ts`, `recipes.ts` and `governor.ts` belong to "Build quality tiers and speed probe".

| Package | State | Log |
|---|---|---|
| VP6 the detail ladder: rung table by screen size and tier, bindings to kernel values (missing value, nothing drawn), seed and clock, anchors folded on the processor, noises written once for processor and GPU (simplex with turning gradients, curl, cells, ridged, flow), TSL node, HUD detail line, six starter declarations | done | packages/VP6.md |
| VP3 texture ladder and relief normals: cube faces 64 to 2048 px by screen size, baked from the kernel's tiles, normals from heights, cached | next | |

No kernel change and no golden change.
