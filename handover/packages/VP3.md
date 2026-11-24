# VP3 part 1: the texture ladder and relief normals

Design: `handover/design/SSE4-visual-polish-design.md` section 5.3 (rows "Picture size" and "Relief"), accepted by Alex 24 Nov 2026 18:57. Thread "Build detail ladder and world textures".

## What it does

System True used to wrap each world in a 128 × 64 picture made on the processor (about 32 texels a cube face, nearest-sampled, pinched at the poles, no relief). Now a world with surface tiles wears two **cube maps baked on the GPU**: its colour and its **relief normal**.

- **Sized to the screen** (`faceSize`): each face's edge is about π/4 of the disc's width in pixels, rounded up to a power of two. It is 64 to 512 on the basic tier and up to 2048 on high (the face cap goes by WebGPU or WebGL2 until the speed probe, VP4, places devices). A bake is never finer than four texels per tile sample, and is kept while it is within a factor of two of the wanted size. An LRU keeps the bakes inside a budget: 48 MB on basic, 256 MB above.
- **Any tile levels** (`composeFaces`): the tiles read so far are laid into one picture per face. The deepest read wins and coarser levels fill the gaps. So when finer tiles are read (`levelFor`, `tilesFacing` say which), the same code shows them.
- **Relief at true scale**: the normal is tilted by the slope between points one texel either side along the surface, using the tiles' heights with no exaggeration. Those points may lie across a cube edge, so there are no seams. Seas are flat. Near the terminator the sun grazes the ground, so real craters and ranges throw shade. On the Moon with level-2 tiles the craters stand out along the terminator (screens below).
- One TSL bake for WebGPU and WebGL2: a cube camera at the centre looks out at an inside-out sphere. Each texel finds its kernel face and cube coordinates by the kernel's equiangular map, then reads bilinear inside that face. The material decodes sRGB and carries the body-frame normal to view space.

Nothing is invented: both cubes come only from the kernel's tiles. Worlds without tiles are drawn as before, and so is the canvas tier (the 128 × 64 picture stays for it and for the mean colour).

## Not yet (part 2)

**Reading finer tiles.** The chart reads level-0 tiles (32 a face), so on screen today the gain is the cube mapping, smoother sampling and the relief. Finer tiles cost the kernel about 0.2 s each (sample Sol in Node: Earth's 24 level-1 tiles took 5 s, its 96 level-2 tiles 20 s; the Moon's twice that). Part 2 reads them for the chosen world only, the tiles facing the viewer first, a request at a time. That touches `SystemChart.svelte`, the chart room's file, so it waits on its agreement.

## Code

- `packages/renderer/src/texture-ladder.ts`: `faceSize`, `levelFor`, `tilesFacing`, `faceDirection`, `directionFace`, `composeFaces` (with `radius_m` from the tiles' spacing), `sampleFaces`, `reliefNormal`, `texelAngle`, `cubeBytes`, `BAKE_BUDGET`, `faceWords`.
- `packages/renderer/src/texture-ladder-gpu.ts`: `createTextureBaker` (bake, LRU, the wearing material).
- `packages/renderer/src/true-look.ts`: `TrueTexture.faces` (additive). `trueSurface` also composes the faces.
- `packages/renderer/src/gpu.ts` (additive): a lit world whose surface has faces wears the bake sized to its disc on screen; otherwise the old material.

## Checks

- `packages/renderer/test/texture-ladder.test.ts` (7 tests): face sizes and levels; the cube map is the kernel's (round trip and agreement with ui-contract's `faceOf`); tiles facing the viewer; deeper tiles laid over coarser ones; sampling continuous across cube edges; the normal tilts by the true angle (tan 0.05 ramp), flat ground and seas point straight up; HUD words.
- `tests/ui/ladder-gpu.e2e.mjs`: a made-up world baked at 128 and read back along a 64 × 32 grid of directions, compared with the processor. WebGL2 and WebGPU: colour within 0.017 (linear), normal within 0.0025 (true tilts up to 0.63°).
- `tests/ui/system3d.e2e.mjs` passes (Kestrel has no surfaces, so it checks nothing breaks). `npm run ui:check` is clean.

## Screens

Project files `sse4/handover-screens/VP-screens/`: `VP3-earth-old-new-level2.png` (the old picture, the cube from level 0, the cube from level 2: same continents in the same places), `VP3-moon-old-vs-level2-relief.png` and `VP3-moon-level2-relief-full.png` (the sample Sol's Moon, lit from the side: the old picture against level-2 tiles with relief). Drawn with WebGL2 in the cloud container, because headless WebGPU there renders but does not present to a screenshot.

## Fudges

Three rows in `docs/fudges.md` after the detail ladder rows: the tier by interface until VP4, the bake size rule, and coarse fill and flat seas.
