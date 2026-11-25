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

The chart reads level-0 tiles (256 samples a face: `tile_samples` on the sample Sol; an earlier note here said 32, which was wrong). Finer tiles cost the kernel about 0.2 s each (sample Sol in Node: Earth's 24 level-1 tiles took 5 s, its 96 level-2 tiles 20 s; the Moon's about twice that). The chart room agreed to the edit in its `SystemChart.svelte` (24 Nov, through the coordinator), with conditions that are all kept:

- `packages/views/body/src/finer-tiles.ts` `readFiner` asks for **4 tiles a request** (about 0.8 s) and checks `wanted()` before each one. Choosing another world or leaving True stops it (the chart's effect generation changes).
- It starts only after the world has stayed chosen for **1.2 s**, which is longer than the 1 s camera fly, and **pauses 1.5 s** before level 2.
- Tiles are kept in one map for the page (`finerTiles(key)`, keyed `system|body`), so the Body screen and the body card's globe can reuse them once the shared surface cache (PR 428) lands. Refused tiles are not asked for again.
- In `SystemChart.svelte`, only inside the True surface reader: finer tiles already read are laid over the kept face at once (`withFiner`), and the chosen world's face is recomposed after each request. Only the chosen world reads finer.
- After the merge with main (TA1, PR 444), System True reads faces through `readTrueFace` (true-air.ts), which paints clouds and haze over the ground. A face with clouds painted over it no longer carries `faces`, so it keeps the flat picture and shows no relief; airless and clear worlds (the Moon, Mercury) wear the bake. **Next:** draw the clouds as their own shell over the baked ground, so cloudy worlds get relief too (with the air thread).

In the shell (sample Sol, WebGL2, cloud container), with the Moon chosen and zoomed in, the face 3 s after choosing already has part of level 1. At 28 s level 2 is in: finer craters and smoother edges between materials (screens below). `walk.e2e` and `perf.e2e` pass inside their budgets, and both open True.

## Part 3, first piece: giants on the Body globe and the card wear their decks

Alex saw Saturn black and white on the Body screen. The Body globe drew the tiles of a giant's cloud-deck relief as grey ground.

- **Body side.** `views/body/src/surface.ts` keeps the world's outermost layer `kind` from `surfaceTiles` (it arrives with the giant bands thread's TA3a). For a `cloud-deck` world, `loadDecks` reads the same True face as the system chart (`readTrueFace`) once and keeps it, and uses it only when that face says `decks: true`.
- **Globe and card.** The Body screen globe, the card's globe and the card's map wear it.
  - The renderer's `globe-face.ts` holds the one lookup for both tiers: `faceSample` (the equirect read) and `withFace` (recolours the globe's samples, with flat normals and albedo from the colour). `runtime.ts` gains `setFace`, and gpu-globe.ts and the canvas tier take the recoloured samples unchanged. This was agreed with the renderer thread.
  - The face's `flow` is passed through. The Body screen gives no time since the read yet, so its bands stand still.
- **Before TA3a lands** no world carries `kind`, so nothing changes. When it lands, giants switch over by themselves.

## Part 4: the deep patch under a close camera (building blocks)

For the renderer's close-up package (True down to a grazing orbit). The renderer hands `GroundView { body, toward, m_per_px }` (renderer `close-up.ts`, System3D's `onground`); this part turns it into ground at the screen's own scale.

- **Level.** `patchLevel(m_per_px, radius, samples, floor)` is the coarsest level with a sample every 2 screen pixels (`PATCH_PX_PER_SAMPLE`), never past the kernel's fact floor. The sample Sol's tiles have 256 samples (level 0 is 39 km a sample on Earth, 10.6 km on the Moon). The fact floor is level 9 on Earth (76 m) and level 7 on the Moon (166 m).
- **Tiles.** `tilesNear(toward, level)` finds the 9 tiles (`PATCH_TILES`) nearest the point by looking round it on the sphere, so a patch that runs over a cube edge takes the neighbouring face's tiles. It never walks a whole level. `patchFor` combines the two, and below level 3 nothing is read, because the globe's faces already do.
- **Reading.** `readDeep(session, system, body, ask, radius, samples, wanted)` (in `views/body/src/finer-tiles.ts`) reads 4 tiles per request and keeps them for the page. A refusal that names the fact floor is kept for the world, and the read drops to that floor. It returns the patch picture.
  - Kernel cost is about 0.3 s a tile at any level (Earth and the Moon, Node), so a patch takes about 3 s.
- **Picture.** `composePatch(tiles, centre, radius)` lays the tiles into a 512 × 512 gnomonic square about the centre, a tile and a half each way.
  - Colour and heights are bilinear across tile edges.
  - Normals come from the true slope between neighbouring texels, one-sided where a neighbour was not read, and are kept in floats (a byte a component bands gentle slopes).
  - Gaps keep alpha 0. It takes about 0.26 s for 9 tiles of 256.
- **Drawing.** The bake's material (`texture-ladder-gpu.ts` `patchNodes`, `Baked.setPatch`) lays the patch over the cube where it was read, eased in over its outer seventh. `TrueTexture.patch` carries it, and `gpu.ts` hands it to the bake each frame (additive, in `bakedMat`).
  - The material mixes in the patch only where its weight is above zero. Off the patch's side of the globe its read is NaN, and NaN times zero is still NaN: the first build blacked out every True world. `ladder-gpu.e2e.mjs` now draws the lit baked globe with no patch and fails on any black or NaN pixel.
- **Wired (25 Nov, after the renderer's PR 488):** SystemChart watches `groundView`. Once the camera has stayed low over the chosen world for 600 ms (`DEEP_SETTLE_MS`), it calls `readDeep` and lays the patch on the world's True face, and on the ground under its clouds when it has a cloud layer. A newer view stops the read between requests. The chart says the level laid in `data-deep-level`. Checked by hand on the sample Sol's Earth: at 3 km over the sea the patch is read at level 9 (76 m, the fact floor). Kestrel's worlds have no surface tiles, so the system3d e2e cannot check it there.
- **Next:** the detail ladder's ground roughness below the fact floor.

## Part 5: bare ground in its measured colour (Alex 25 Nov, "the moon is very brown")

True's ground was a typed palette (packs/base-surface `colours`, ui-contract `sampleColour`): the Moon's "sediment" and "regolith" were beige, and nothing weathered airless soil. Now:
- **Data.** The data sheet holds laboratory spectra of Apollo soils (MP35, PR 512: mature highland 62231, mature, submature and immature mare 10084, 71501, 12030). `renderer/src/ground-rows.json` names which rows each kernel material reads (a fudge row until base-surface names them in the golden lane).
- **Weathering.** The kernel's space weathering (proven core SW1) gives each bare-ground sample a `maturity`, 0 fresh to 255 mature, only on worlds where space reaches the ground (no air). Mature soil is darker and redder (nanophase iron, Hapke 2001).
- **Drawing.** `ground-optics.ts` samples each row at 650, 550 and 450 nm, blends fresh to mature in log space by the sample's maturity, and lays the result on the tiles as `ground_rgb` (sRGB). ui-contract `sampleColour` uses it for bare ground, so True, the Body globe, the card and finer and deep tiles all agree. Applied in `readTrueFace`, `views/body/src/surface.ts` and `finer-tiles.ts`.
- **On for every world (25 Nov, MATCH1):** the renderer's photo matches found Mercury, Mars, Io, Europa and Pluto all wearing the same typed tan palette. Bare ground now reads the sheet on every world. A tile with no `maturity` is drawn fully mature (fudge row). A row may also be a refractive index n + ik (water ice `h2o` for the kernel's "ice" material), drawn as a thick layer of 50 µm grains by Hapke's slab (`rowBands`, through sheet-optics.ts). Granite has no row yet and keeps the palette. Stains (TRUE4) still move each texel's hue over this base.
- **What the base alone can't fix (25 Nov dump of the real Sol, base-sol through solBatches):**
  - Every icy world's tiles are rock: Europa, Ganymede, Callisto, Pluto, Triton and Titan carry no "ice" material and no ice cover. That's the proven core's (kernel, golden lane).
  - Tile albedo is a flat 300.
  - Mars and Pluto get no stains until proven core PR 417 hands the catalogued gases to the Orbit atlas (plumes thread).
  - Io's sulphur has no optics on the sheet.
  So those worlds come out an honest neutral grey until then, not tan.
- **Look.** With every sample mature, the sample Sol's Moon goes from beige to a dark neutral grey (mature mare soil about 6 % reflective, highland about 13 %). Screens: `VP-screens/VP3-part5-moon-palette-vs-measured.png`.

## Smooth coarse tiles (25 Nov, MATCH1)

Basin discs had hard pixel edges in True. A coarse tile was laid into a finer face picture in nearest-neighbour blocks (`composeFaces`), and a coarser face was repeated up in blocks in the GPU atlas. Both now spread bilinearly between sample centres. A shore stays the nearest sample's, so there are no half-wet texels. Test: `texture-ladder.test.ts`, "spread smoothly", fails on the old code.

Craters at globe scale are kernel facts down to the fact floor: they come with deeper tiles and are never invented. The sub-floor crater layer (`ground.craters`, the renderer's VP10) and ground roughness (this thread's) both wait on kernel facts (`crater_cover`, `roughness_m`, `grain_m`), so neither is drawn yet.

## Natural outlines (Alex 25 Nov: "they look a bit bitmap"; "reuse on the timelapse")

One shared, deterministic step in `ui-contract/src/globe.ts`. It never changes a fact: at every sample's centre the kernel's own cover shows.
- `sampleNatural(globe, d)`: the four samples round a point are blended. Each kind (bare ground, ice, liquid) gets its share plus the edge noise, and the largest wins, so coasts and ice edges are smooth contours instead of steps.
- `softenGround(tile)`: bare ground's colour is averaged over bare neighbours two cells each way (two box passes), laid on as `ground_rgb`. `withGround` runs it for every True and Body read.
- `edgeNoise(d, cell_rad, seed)`, `naturalEdge(...)` and `naturalDirection(...)`: the pieces underneath, keyed only on direction and seed, never time. The surface time-lapse thread calls them in `renderer/src/timelapse.ts recolour` on its own cells, so frames don't shimmer.
- Used by the Land map (`views/surface/Surface.svelte`) and True's small equirect pictures (`true-look.ts`).
- Not yet: the canvas-tier Body globe, whose per-pixel loop is too hot for the noise per frame; it gets softened colour only.
- Still blocky because the kernel's facts are blocky: the polar ice ends on the climate's latitude band, and some lakes are multi-cell rectangles.
- Pictures: `VP-screens/VP3-natural-outlines-earth-close.png` and `-map.png`.

## Code

- `packages/renderer/src/texture-ladder.ts`: `faceSize`, `levelFor`, `tilesFacing`, `faceDirection`, `directionFace`, `composeFaces` (with `radius_m` from the tiles' spacing, and the tiles it was made from), `withFiner`, `sampleFaces`, `reliefNormal`, `texelAngle`, `cubeBytes`, `BAKE_BUDGET`, `faceWords`.
- `packages/renderer/src/texture-ladder-gpu.ts`: `createTextureBaker` (bake, LRU, the wearing material).
- `packages/renderer/src/true-look.ts`: `TrueTexture.faces` (additive). `trueSurface` also composes the faces.
- `packages/renderer/src/gpu.ts` (additive): a lit world whose surface has faces wears the bake sized to its disc on screen; otherwise the old material.
- `packages/views/body/src/finer-tiles.ts`: `readFiner`, `finerTiles`, `FINER_BATCH`, `FINER_SETTLE_MS`, `FINER_PAUSE_MS`, `FINER_LEVELS`.
- `packages/views/system-chart/src/SystemChart.svelte` (True surface reader only): finer tiles laid over with `withFiner`, `readFiner` for the chosen world.

## Checks

- Part 3: `packages/renderer/test/globe-face.test.ts` (a face read along a pixel's own direction gives that pixel, longitude wraps; the samples wear the face with flat normals). `globe`, `body-pictures`, `render`, `compare` and `walk` e2e pass.

- Part 4: `texture-ladder.test.ts` (level from metres a pixel at the fact floor, tiles round a point and over a cube edge, the patch picture's colour, gaps and true tilt); `tests/ui/finer-tiles.test.ts` (readDeep drops to the floor a refusal names, keeps tiles and floor, nothing read from far off); `tests/ui/ladder-gpu.e2e.mjs` reads the patch on WebGL2 and WebGPU at texel centres against `samplePatch` (colour within 0.0002, weight within 0.0005, normal within 0.0004).

- `packages/renderer/test/texture-ladder.test.ts` (8 tests): finer tiles laid over a kept face once, never doubled, a painted face left alone; face sizes and levels; the cube map is the kernel's (round trip and agreement with ui-contract's `faceOf`); tiles facing the viewer; deeper tiles laid over coarser ones; sampling continuous across cube edges; the normal tilts by the true angle (tan 0.05 ramp), flat ground and seas point straight up; HUD words.
- `tests/ui/ladder-gpu.e2e.mjs`: a made-up world baked at 128 and read back along a 64 × 32 grid of directions, compared with the processor. WebGL2 and WebGPU: colour within 0.017 (linear), normal within 0.0025 (true tilts up to 0.63°).
- `tests/ui/finer-tiles.test.ts` (2 tests): both levels read four at a time, kept, nothing asked twice (refused tiles too); stops between requests once not wanted.
- `tests/ui/walk.e2e.mjs` and `tests/ui/perf.e2e.mjs` pass.
- `tests/ui/system3d.e2e.mjs` passes (Kestrel has no surfaces, so it checks nothing breaks). `npm run ui:check` is clean.

## Screens

Project files `sse4/handover-screens/VP-screens/`: `VP3-earth-old-new-level2.png` (the old picture, the cube from level 0, the cube from level 2: same continents in the same places), `VP3-moon-old-vs-level2-relief.png` and `VP3-moon-level2-relief-full.png` (the sample Sol's Moon, lit from the side: the old picture against level-2 tiles with relief); `VP3p2-sol-moon-3s-vs-28s.png` and `VP3p2-sol-moon-true-28s.png` (the shell's System True, Moon chosen: 3 s and 28 s after choosing). Drawn with WebGL2 in the cloud container, because headless WebGPU there renders but does not present to a screenshot.

## Fudges

Three rows in `docs/fudges.md` after the detail ladder rows: the tier by interface until VP4, the bake size rule, and coarse fill and flat seas.
