# VP3: the texture ladder and relief normals (parts 1 and 2)

Design: `handover/design/SSE4-visual-polish-design.md` section 5.3 (rows "Picture size" and "Relief"), accepted by Alex 24 Nov 2026 18:57. Thread "Build detail ladder and world textures".

## What it does

System True used to wrap each world in a 128 × 64 picture made on the processor (about 32 texels a cube face, nearest-sampled, pinched at the poles, no relief). Now a world with surface tiles wears two **cube maps baked on the GPU**: its colour and its **relief normal**.

- **Sized to the screen** (`faceSize`): each face's edge is about π/4 of the disc's width in pixels, rounded up to a power of two. It is 64 to 512 on the basic tier and up to 2048 on high (the face cap goes by WebGPU or WebGL2 until the speed probe, VP4, places devices). A bake is never finer than four texels per tile sample, and is kept while it is within a factor of two of the wanted size. An LRU keeps the bakes inside a budget: 48 MB on basic, 256 MB above.
- **Any tile levels** (`composeFaces`): the tiles read so far are laid into one picture per face. The deepest read wins and coarser levels fill the gaps. So when finer tiles are read (`levelFor`, `tilesFacing` say which), the same code shows them.
- **Relief at true scale**: the normal is tilted by the slope between points one texel either side along the surface, using the tiles' heights with no exaggeration. Those points may lie across a cube edge, so there are no seams. Seas are flat. Near the terminator the sun grazes the ground, so real craters and ranges throw shade. On the Moon with level-2 tiles the craters stand out along the terminator (screens below).
- One TSL bake for WebGPU and WebGL2: a cube camera at the centre looks out at an inside-out sphere. Each texel finds its kernel face and cube coordinates by the kernel's equiangular map, then reads bilinear inside that face. The material decodes sRGB and carries the body-frame normal to view space.

Nothing is invented: both cubes come only from the kernel's tiles. Worlds without tiles are drawn as before, and so is the canvas tier (the 128 × 64 picture stays for it and for the mean colour).

## Part 2: finer tiles for the chosen world

The chart reads level-0 tiles (32 samples a face). Finer tiles cost the kernel about 0.2 s each (sample Sol in Node: Earth's 24 level-1 tiles took 5 s, its 96 level-2 tiles 20 s; the Moon's about twice that). The chart room agreed to the edit in its `SystemChart.svelte` (24 Nov, through the coordinator), with conditions that are all kept:

- `packages/views/body/src/finer-tiles.ts` `readFiner` asks for **4 tiles a request** (about 0.8 s) and checks `wanted()` before each one. Choosing another world or leaving True stops it (the chart's effect generation changes).
- It starts only after the world has stayed chosen for **1.2 s**, which is longer than the 1 s camera fly, and **pauses 1.5 s** before level 2.
- Tiles are kept in one map for the page (`finerTiles(key)`, keyed `system|body`), so the Body screen and the body card's globe can reuse them once the shared surface cache (PR 428) lands. Refused tiles are not asked for again.
- In `SystemChart.svelte`, only inside the True surface reader: the level-0 tiles are kept, finer tiles already read are laid over at once, and the chosen world's surface is recomposed after each request. Only the chosen world reads finer.

In the shell (sample Sol, WebGL2, cloud container), with the Moon chosen and zoomed in, the face 3 s after choosing already has part of level 1. At 28 s level 2 is in: finer craters and smoother edges between materials (screens below). `walk.e2e` and `perf.e2e` pass inside their budgets, and both open True.

## Code

- `packages/renderer/src/texture-ladder.ts`: `faceSize`, `levelFor`, `tilesFacing`, `faceDirection`, `directionFace`, `composeFaces` (with `radius_m` from the tiles' spacing), `sampleFaces`, `reliefNormal`, `texelAngle`, `cubeBytes`, `BAKE_BUDGET`, `faceWords`.
- `packages/renderer/src/texture-ladder-gpu.ts`: `createTextureBaker` (bake, LRU, the wearing material).
- `packages/renderer/src/true-look.ts`: `TrueTexture.faces` (additive). `trueSurface` also composes the faces.
- `packages/renderer/src/gpu.ts` (additive): a lit world whose surface has faces wears the bake sized to its disc on screen; otherwise the old material.
- `packages/views/body/src/finer-tiles.ts`: `readFiner`, `finerTiles`, `FINER_BATCH`, `FINER_SETTLE_MS`, `FINER_PAUSE_MS`, `FINER_LEVELS`.
- `packages/views/system-chart/src/SystemChart.svelte` (True surface reader only): `level0` map, finer tiles laid over, `readFiner` for the chosen world.

## Checks

- `packages/renderer/test/texture-ladder.test.ts` (7 tests): face sizes and levels; the cube map is the kernel's (round trip and agreement with ui-contract's `faceOf`); tiles facing the viewer; deeper tiles laid over coarser ones; sampling continuous across cube edges; the normal tilts by the true angle (tan 0.05 ramp), flat ground and seas point straight up; HUD words.
- `tests/ui/ladder-gpu.e2e.mjs`: a made-up world baked at 128 and read back along a 64 × 32 grid of directions, compared with the processor. WebGL2 and WebGPU: colour within 0.017 (linear), normal within 0.0025 (true tilts up to 0.63°).
- `tests/ui/finer-tiles.test.ts` (2 tests): both levels read four at a time, kept, nothing asked twice (refused tiles too); stops between requests once not wanted.
- `tests/ui/walk.e2e.mjs` and `tests/ui/perf.e2e.mjs` pass.
- `tests/ui/system3d.e2e.mjs` passes (Kestrel has no surfaces, so it checks nothing breaks). `npm run ui:check` is clean.

## Screens

Project files `sse4/handover-screens/VP-screens/`: `VP3-earth-old-new-level2.png` (the old picture, the cube from level 0, the cube from level 2: same continents in the same places), `VP3-moon-old-vs-level2-relief.png` and `VP3-moon-level2-relief-full.png` (the sample Sol's Moon, lit from the side: the old picture against level-2 tiles with relief); `VP3p2-sol-moon-3s-vs-28s.png` and `VP3p2-sol-moon-true-28s.png` (the shell's System True, Moon chosen: 3 s and 28 s after choosing). Drawn with WebGL2 in the cloud container, because headless WebGPU there renders but does not present to a screenshot.

## Fudges

Three rows in `docs/fudges.md` after the detail ladder rows: the tier by interface until VP4, the bake size rule, and coarse fill and flat seas.
