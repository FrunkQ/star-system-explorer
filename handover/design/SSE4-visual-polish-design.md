# SSE4 visual polish: closing the gap to SpaceEngine, pass by pass

Design note, 24 Nov 2026. Track code **VP** (never shown in the product). Status: **accepted by Alex, 24 Nov 2026 18:57 UTC ("yes to all" ten questions in section 14)**. Builders may take the packages in section 12. Design only: no code was written for this note. **Addendum of 24 Nov 19:06 in section 16** (systems, not Solar System fixes; aurorae): its questions 11 to 15 were accepted by Alex at 19:12 UTC ("yes to all"), with one change: **"Long exposure" is the default and the eye-true view is the choice** (ruling in 16.7, which overrides the default written in 5.2 and questions 9 and 13). Still open: the citations marked "from memory" (Alex is bringing a research pass back) and every cost marked as an estimate.

Asked for by Alex, 24 Nov, in four lines:

- "Surely Rust and our two GL layers can expose most power to be used. We should shoot for the stars, use the best ideas and code out there. The NASA/ESA tools have great ideas. We need to fine tune and hone. Add visual flourishes where they don't cost a fortune, eg more swirls when close up."
- "Let it have a DEEP dive into render passes and options for us to do this WHILE staying performant. There are millions of shader tricks and amazing stuff on r/procgen. Some of our longer physics calcs maybe could be shoved into the GPU for a big update. Not sure, but bear in mind."
- "Let's use everyone else's experience to add the gloss they have spent years doing. Getting close would be a major achievement because we can always beat them on breadth."
- "Any new render techniques that hammer the GPU we can have gated on performance checks to stay compatible with GPU-lite machines."

**Read with:** `SSE4-surfaces-and-rendering.md` (renderer base, tiers, recipes, governor, licences: this note extends its sections 5 to 7), `SSE4-surface-design.md` (the fact floor, and 4A.3 "flourishes"), `SSE4-weather-design.md`, `SSE4-nebulae-and-light-design.md`, `SSE4-exotic-objects-design.md` (light bending is designed there, not here), and the third audit in `SSE4-v3-gaps.md` (33 SSE3 render features against True, R1 to R33). In the repo: `handover/rendering-rules.md`, `handover/status/renderer.md`, `star-look.md`, and the status files for the two "True" threads (on open PRs 444 and 449, copied in `handover/status-on-open-prs.md`).

**Citation check, 24 Nov 19:27.** Alex brought back a research pass on eighteen of the "from memory" items. Its results are folded in and summarised in section 17; the full check with every DOI is `SSE4-visual-polish-citation-check.md` beside this note.

**How claims are marked.** "Checked" means read on the source's own page or in SSE4's code on 24 Nov. **[from memory]** means not checked for this note: verify the title, the formula and the licence before building on it, and record the check in `docs/citations.md`. **[estimate]** is a cost guess to be replaced by a measurement from the performance readout.

---

## 1. Short answers

**Is the language the limit?** No. Alex is right. WebGPU hands a page the same graphics card that SpaceEngine's OpenGL uses, and TSL shaders compile to the same driver. Rust (the kernel) is not on the drawing path at all. What a browser really lacks is short and specific (section 2.3), and none of it blocks the look.

**So why does SpaceEngine look better today?** Three reasons, in order of size:

1. **Finish.** SpaceEngine draws everything into a high-range picture, sets an exposure, lets bright things bloom, and tone-maps. SSE4's system True draws lit spheres straight to the screen on black, with no stars behind them. This one difference is most of what the eye reads as "gloss", and it is a few milliseconds of standard work.
2. **Detail that arrives as you approach.** Their giants have curls inside the bands; ours have clean stripes. Their ground has stones; ours is a smooth brown plain. Ours is correct to the kernel's facts and then stops. Theirs keeps going with invented detail.
3. **Years of tuning.** Hundreds of small things (ring shadows, eclipse shadows, city lights, the terminator's colours). Each is cheap. The list is long, and it is written down in sections 5 and 9.

**Can SSE4 get close?** Yes on a desktop with a real graphics card, in about the order of section 9. On a phone it gets close in a picture that sharpens when the view stops moving (section 8.4), not while flying. The earlier note said the same; this note says how.

**Where SSE4 then wins.** Every one of their flourishes is invented from a noise preset. Ours can be driven by the world's own numbers: the swirl size on a giant from its wind speed and spin, the width of the sun's glint on a sea from the wind over it, the place of a gap in a ring from the moon that clears it. Section 10 lists twenty-four of these "an expert can test it" touches. That is the part nobody else can copy quickly.

**GPU physics.** Weighed in section 11. Short form: use the GPU now for everything that is drawing or preview; get the first big speed-ups for the kernel on the processor (several kernel workers, then SIMD), which keep results bit-identical for free; try one measured GPU experiment later before deciding anything.

---

## 2. Where True stands today (24 Nov, main at f95a334)

### 2.1 What was looked at

The newest screenshots in project files (`sse4/handover-screens/TRUE-atmos-screens`, `R2-screens`, `SL1-screens`, `SG-screens`) and the renderer's source (`packages/renderer/src`, about 10,400 lines).

### 2.2 What they show

| View | Today | Reads as |
|---|---|---|
| System True (`gpu.ts`, `true-look.ts`) | Each world a lit sphere wearing a **128 x 64** picture made on the processor from level-0 tiles. Black background, no stars. No tone mapping, no bloom. Giants are grey-white stripes (TA1 has decks, colours wait on the data sheet). Rings and belts drawn. The star has its glow (SG1, a 160 px sprite) | A correct diagram in the dark |
| Body globe (`gpu-globe.ts`, recipes) | Relief globe, single-scattering air from the kernel's optics, ACES tone mapping in physical lighting, today's clouds laid on the ground. Runs on recipes with the governor and a quality badge | Good bones. Smooth ball, soft blue rim |
| Land (`gpu-surface.ts`) | Ground at the fact floor under a Rayleigh sky, waves and glint on seas (V10k) | Honest and empty: a smooth plain to a clean horizon, no stones, no sun glare |
| Star (SG1 to SG3) | Glare from the eye's own scatter (CIE), churning cells, lumpy giants, flares, ejections | Already the best-looking thing in SSE4 |
| Galaxy True, web (V9t, V9w) | Layers of light per band | Good from afar |

### 2.3 Four findings in the code that shape this note

1. **There is no shared finishing chain.** Five separate renderers (`gpu.ts`, `gpu-globe.ts`, `gpu-surface.ts`, `gpu-ship.ts`, `gpu-station.ts`) each open their own `WebGPURenderer`. Two of them tone-map. None blooms. System True does neither.
2. **Only the Body globe is governed.** Recipes, the degrade order, the frame-time governor and the quality badge (R1) run the globe. System True, Land, ships and stations draw outside them, so nothing there can be gated on performance yet.
3. **The device probe was designed but not built.** `passes.ts` refers to a `probe.ts` that does not exist. Today the tier is the API: WebGPU means `gpu-high`, WebGL2 means `gpu-basic`. Alex's Pixel 11 Pro runs WebGL2 and is fast; an old laptop may run WebGPU and be slow. The tier must come from a measurement (section 8).
4. **The kernel runs in one worker, without SIMD.** That matters for section 11.

### 2.4 What a browser really lacks against OpenGL 4.6 on a desktop

- No tessellation or geometry shaders in WebGPU. Terrain detail must come from mesh level of detail and from the pixel shader (normals, parallax), not from the card splitting triangles.
- No 64-bit floats on the GPU, in WebGPU or WebGL2. Already handled by rendering rule 3 (camera-relative, small numbers).
- No hardware ray tracing in browsers yet.
- Tighter memory, and no sparse textures. Assets must be generated, not shipped (already the plan).
- Compute shaders only on WebGPU. Anything using them needs a second form or a tier gate.
- Coarse GPU timers (timestamp queries are optional and rounded for privacy). The governor's frame time stays the main signal.
- Phones throttle with heat after a minute or two.

None of these stops any item in section 9 except the ones marked Ultra.

---

## 3. The rule every flourish obeys

Alex's project rule is "lean into the physics, never fake it". The surface design already has the tool for this: **the fact floor** (facts above it, decoration below it, and nothing in play depends on the decoration). This note widens that one idea to every drawn thing, so that polish never becomes faked physics.

Every drawn detail belongs to exactly one of three grades, and each layer declares which:

| Grade | What it is | Example | How the user can tell |
|---|---|---|---|
| **Fact** | Drawn from a kernel value at that place and time | A named storm; a crater; the ring's edge; the glint's width from the wind | It is on the card and Explain can answer "why" |
| **Drawn from the numbers** | Detail finer than the kernel resolves, made in the shader, but whose size, direction, speed, colour and amount are set by kernel values | Curls inside a giant's band, sized by the band's shear and the planet's spin; stones on the ground sized by the surface's age and gravity | The HUD's detail line: "Finer than 400 km is drawn from the wind's figures, not computed." One line per view, as the fact floor already has |
| **Camera effect** | Belongs to the instrument or the eye, not the world | Glare, bloom, lens spikes, grain, vignette, motion blur | Named as such in the recipe. Eye effects (glare) are on. Lens effects are off in True unless a recipe asks |

Rules that follow:

1. **Nothing in the second grade may be mistaken for the first.** A drawn curl is never clickable, never named, never in a report, and never read by a rule or a hazard. If the GM wants it to be real, they pin a storm (the weather track already has pins).
2. **The second grade is seeded and clocked by the kernel.** Seed from the body's address, time from the universe clock. Every seat then sees the same curl in the same place, and it is there again tomorrow. Tiny differences between graphics cards are allowed, exactly as for ground below the fact floor.
3. **Its statistics are real numbers or it is not drawn.** If the kernel has no wind for a world, there are no curls, and the HUD says why. A flourish with no number behind it is a camera effect or it is cut.
4. **Each one is in the fudge register** (`docs/fudges.md`) with what real physics would replace it. Most entries will read "replace when the weather block resolves eddies".

This is "one general mechanism over specials": one grade system, one HUD line, one seed rule, for giants, clouds, ground, rings, seas and stars alike. The mechanism that produces second-grade detail is the **detail ladder** in section 6.

---

## 4. Who does what well, and what may be reused

### 4.1 Licence rule (SSE4 is Apache-2.0)

- **MIT, BSD, Apache-2.0, public domain:** code may be ported (to TSL or Rust) with the notice kept, a row in `docs/citations.md`, and a line in a `NOTICE` file.
- **MPL-2.0:** usable only as unmodified separate files. In practice, read and reimplement.
- **GPL, AGPL, Creative Commons NonCommercial or ShareAlike, and Shadertoy's default licence:** ideas only. Read the paper or the write-up, close it, and write fresh. The same rule Alex set for n-body (never from REBOUND).
- **A paper's formula is free to implement.** A paper's listing is under whatever licence its code carries; when none is stated, write it from the formula.

Shadertoy matters because much of the best shader work lives there. Its default licence is CC BY-NC-SA 3.0 unless the author says otherwise **[from memory, well known; check each shader's header]**, which SSE4 cannot take. Many authors mark theirs MIT; those are fine.

### 4.2 The table

"Have" is SSE4 True today: **yes**, **partly** or **no**, from section 2 and the third audit (R numbers).

| Source | Licence | Best-looking things it does | SSE4 has | Reuse |
|---|---|---|---|---|
| **SpaceEngine** | Proprietary | High-range picture with real exposure modes and planet shine (checked, their blog 13 Nov 2020). Terrain: splat map of 4 materials per pixel from a 64-material detail set in one texture array, small and large textures mixed by their displacement, tiling hidden by Inigo Quilez's two-sample method (checked, blog 2 Nov 2017). Ray-marched nebulae and 3D rings with rocks (planned in those posts). Also **[from memory]**: giants with cyclones and animated flow, volumetric clouds, eclipse and ring shadows, aurorae, comet tails, lens flares and spikes, light bending | finish no; terrain detail no; the rest per rows below | Ideas only. It is the benchmark |
| **Universe Sandbox** | Proprietary | Surface grids: water, ice and temperature simulated cell by cell across the surface (checked, DevLog 8, 2019). "Localized glow": bloom only where something is truly hot; hot things emit by temperature; stars physically bright; star glow drawn behind atmospheres (checked, graphics preview Nov 2024). Its physics moved to many processor cores in Dec 2025, not to the GPU (checked) | hot glow partly (TRUE3); the rest no | Ideas only. Note the last point for section 11 |
| **NASA Eyes** | Not open (earlier note) | Calm camera flights between bodies, trails that fade, labels that never collide, time scrubbing, real spacecraft models | camera eases partly | Look and feel only |
| **OpenSpace** (NASA, AMNH) | MIT | Atmosphere after Bruneton and Elek, ring shadows, eclipse shadows with penumbra, globe browsing by chunked level of detail, star rendering from catalogues with a point-spread shape **[from memory]** | air partly; shadows no | **Shader ideas and GLSL can be ported with notice.** Desktop C++, so pieces, not the engine |
| **Celestia** | GPL-2.0-or-later **[from memory]** | Eclipse shadows, ring shadows, cloud shadows, night-side lights, sea glint, comet tails, and a "lunar-Lambert" surface law for airless bodies | no, no, no, no, partly, no, no | Ideas only |
| **Gaia Sky** (ESA, ARI) | MPL-2.0 (earlier note) | Procedural worlds: height and humidity noise sampled on a sphere, a two-axis look-up picture for biome colour, clouds by the same method stretched along latitude, a sample count for scattering (checked, its manual). Stars from Gaia with real colours and motions | ours come from history, better | Ideas. Its star handling is the reference for the sky |
| **Cosmographia** (NAIF; built on the open-source Cosmographia and the VESTA library, checked on NAIF's credits page; no licence stated there) | Unclear: treat as ideas | Mission geometry: trajectories, sensor view cones, instrument footprints on a surface **[from memory]** | no | Ideas. Sensor cones and footprints suit the Run desk and the sensing design |
| **Stellarium** | GPL-2.0 **[from memory]** | The sky from the ground: twinkling, dimming and reddening toward the horizon, refraction, the Milky Way, light pollution | partly (sky drawn; none of the effects) | Ideas only |
| **Cosmos Journeyer** | AGPL-3.0 (earlier note) | A whole procedural universe in a browser | n/a | Do not read the code |
| **three.js examples and TSL display nodes** | MIT | Bloom, ambient occlusion, temporal anti-aliasing, FXAA, depth of field, lens flare, film grain, tone mappers (ACES, AgX, Neutral), noise nodes, compute particles, sky and water meshes. **Check the list in the installed version (0.187) under `examples/jsm/tsl/display`; `node_modules` was not installed where this note was written** | tone mapping in two views | **Use directly.** The cheapest gloss there is |
| **Hillaire 2020**, "A scalable and production ready sky and atmosphere rendering technique"; `webgpu-sky-atmosphere` | MIT (earlier note) | Any planet's air from small look-up pictures, with multiple scattering and aerial perspective | partly: ours is single scattering, ray-marched every frame | **Port** (already recommended; still to do) |
| **Bruneton and Neyret 2008**; `precomputed_atmospheric_scattering` | BSD-3-Clause **[from memory]** | Precomputed scattering, the classic | no | Reference; Hillaire suits many planets better |
| **Schneider, "Nubis"** (Horizon Zero Dawn, SIGGRAPH courses 2015, 2017, 2022) | Talks | Volumetric clouds: weather map, shape and detail noise, powder term, quarter-resolution march rebuilt over frames | no | Implement from the talks |
| **`@takram/three-clouds`** | MIT (checked) | A working browser Nubis: Beer shadow maps, temporal upscaling to 1/16 of the texels, dual-lobe phase, light shafts. Measured by its authors: 92 to 95 fps on an M3 Max at High, 36 to 53 fps on an iPhone 13 at Low. **GLSL on WebGL post-processing only; no TSL, no WebGPU yet (checked)** | no | **Port the method and constants to TSL with notice.** Cannot be dropped in |
| **`@takram/three-atmosphere`** | MIT (checked) | Bruneton in the browser; effectively Earth only (checked) | n/a | No |
| **Jendersie and d'Eon 2023**, "An approximate Mie scattering function for fog and cloud rendering" (SIGGRAPH Talks) | Paper; its HLSL has no stated licence (checked) | One cheap formula for how droplets scatter light, with the droplet's diameter as its input; fitted for water in air | no | **Write from the paper's formula.** For other liquids it is a stand-in: fudge register |
| **Bridson, Hourihan and Nordenstam 2007**, "Curl-noise for procedural fluid flow" | Paper | Swirling flow that never piles up or thins out, from the curl of a noise field | no | Implement. The base of the giants' curls |
| **Gaseous Giganticus** (S. Cameron, 2014) | Code licence not stated on its page; its parent project is GPL **[from memory]** | Gas giant pictures by moving particles through curl noise on a cube map; offline, on the processor (checked) | no | Idea only |
| **J. R., "Gas giant particle sim on a sphere"** (2022) | None stated | The same in real time: 4 million particles by compute shader at 80 fps on a desktop (checked) | no | Idea. The Ultra form of the giants |
| **Perlin and Neyret 2001**, "Flow noise"; **Vlachos 2010**, "Water flow in Portal 2" (flow maps) | Papers and talk | Texture that flows along a wind field for two extra texture reads | no | Implement. The Basic form of the giants and clouds |
| **`psrdnoise`, `webgl-noise`** (Gustavson, McEwan) | MIT (checked for the Ashima code) | Simplex noise with exact slopes and rotating gradients (flow noise built in), no textures needed | no | **Port.** The simplex patent has expired **[from memory]** |
| **Inigo Quilez's articles** | Articles; his listings are often MIT **[check each]** | Domain warping, noise with derivatives, hiding texture repeats, smooth cell noise | no | Implement from the articles |
| **Heitz and Neyret 2018** (histogram-preserving blending); **Mikkelsen 2022**, "Practical real-time hex-tiling" | Papers; hex-tiling sample is MIT **[from memory]** | A small texture covering a planet with no visible repeat | no | Implement. Needed for ground detail |
| **Cox and Munk 1954** | Paper | Width of the sun's glint from wind speed | glint yes on the ground; from orbit no | Implement as a block |
| **Tessendorf 2001**, "Simulating ocean water" | Notes | Wave fields from a spectrum | partly (wave trains, V10k) | Later, Ultra |
| **Hapke; Lommel-Seeliger** | Textbook | How dusty airless ground reflects (no dark rim at full phase; a bright surge at opposition) | no (Lambert spheres) | Implement as a block |
| **Chandrasekhar 1960** (thin layer, single scattering) | Textbook | Rings lit from the front and seen from the dark side | lit side yes (`ringLight`); dark side to check | Extend |
| **AMD FidelityFX Super Resolution 1** | MIT **[from memory]** | Draw at half size, enlarge sharply for about half a millisecond | no | **Port.** The biggest single lever for phones |
| **NVIDIA spatiotemporal blue noise** | Permissive **[check]** | Removes banding from ray marches at few steps | no | Use the textures, or make blue noise ourselves |
| **Sebastian Lague's "Coding Adventures"** (clouds, atmosphere, planets, erosion) | MIT repositories **[from memory]** | Clear, popular builds of exactly these effects; the procedural generation community's common reference | n/a | Read for approach |
| **NASA SVS** (Deep Star Maps, CGI Moon Kit, Blue Marble), **Photojournal** | US government work, generally public domain | Reference pictures for the per-body check (audit R3); real maps for real bodies as pins | n/a | Reference in project files; ship only as optional packs |
| **ESA images and Gaia sky maps** | CC BY-SA 3.0 IGO | Reference pictures | n/a | Reference only (ShareAlike); Gaia data stays in its optional pack (standing ruling) |

### 4.3 The procedural generation community's bag of tricks

Alex named r/procgen. The community's recurring tricks, and where each lands in this note:

| Trick | What it buys | Used in |
|---|---|---|
| Domain warping (noise fed into noise) | Marbled, flowing shapes for almost nothing | Giants (5.4), nebulae (5.14) |
| Curl noise | Flow that looks like fluid | Giants, clouds, nebula wisps, plumes |
| Flow maps with two-phase blending | Textures that move along a wind field | Giants, clouds, dust storms, lava |
| Octaves added by distance (band-limited noise) | Detail that arrives as you approach, at constant cost | The detail ladder (6) |
| Noise with analytic derivatives, erosion-like damping | Ridges and gullies without simulating erosion | Ground below the fact floor (5.13) |
| Ridged and billowed noise | Mountains, dunes, cloud tops | Ground, cloud shell |
| Cell (Worley) noise | Cracked ice, lava crust, convection cells, cloud puffs, craters | Ice moons, lava, cloud shapes |
| Triplanar mapping and hex-tiling | Textures on a sphere with no seams or repeats | Ground detail |
| Poisson-disc scatter, instanced impostors | Stones, boulders, ring particles by the hundred thousand | Ground (5.13), rings (5.9) |
| Ray-marched distance shapes | Lumpy asteroids with no mesh | Small bodies (5.10) |
| Blue noise and interleaved gradient noise | Few-step ray marches that look smooth | Air, clouds, nebulae |
| Temporal accumulation | Quality that builds while the view is still | Section 8.4 |
| Half-resolution effects with depth-aware enlargement | Volumetrics at a quarter of the cost | Clouds, nebulae, plumes |
| Point sprites sized by brightness | A real star field for one draw call | Sky (5.2) |
| Reaction-diffusion patterns | Alien surface markings | Camera-effect grade only, unless a life pack gives it numbers |
| Hydraulic erosion on the GPU | Terrain in milliseconds | Preview only (section 11) |

---

## 5. The render passes, one by one

Each pass: what the best do, the options, the recommendation, and the tier marks from section 8 (**B** Basic, **H** High, **U** Ultra, **S** still or at rest). Phone figures are for the reference phone (Alex's Pixel 11 Pro on WebGL2, render scale 0.75) and are all **[estimate]** until measured.

### 5.1 The frame finish (the largest single gap)

**The best:** draw the scene in linear light with real brightness ratios into a half-float picture; choose an exposure; let anything far brighter than white spread as glare; tone-map; dither.

| Step | Options | Recommended | Tier | Phone |
|---|---|---|---|---|
| High-range target | 8-bit (today); half-float | Half-float where the device has it, else 8-bit with exposure applied in the material | B | free |
| Exposure | Fixed; from mean scene brightness with slow adaptation; spot-metered on the selected body | Metered on what the view is centred on, adapting over about a second, with limits. The HUD says the exposure in stops | B | 0.2 ms |
| Glare and bloom | None (today); thresholded blur chain; physical glare (the eye's scatter, as SG1 does for stars) | One glare for everything, using SG1's CIE curve as its shape, at quarter size. Then a red-hot lava lake, a city, a sun glint and a star all glow by their real brightness with no special cases. SG1's sprite becomes this pass's first user | B (quarter size), H (half size) | 1 ms |
| Tone map | ACES (today, two views); AgX; Khronos Neutral | AgX: keeps bright colours from going garish. One choice for every True view | B | 0.1 ms |
| Dither | none; blue noise | Blue noise, one texture read. Ends banding in dark skies and limbs | B | 0.05 ms |
| Edges | MSAA (today); FXAA; temporal | Keep MSAA on H; FXAA on B; temporal at rest (8.4) | B | 0.5 ms |
| Enlarge | none; FSR1-style | Draw True at 0.5 to 0.77 scale on B and enlarge sharply | B | 0.5 ms, and it **saves** far more |

Grade: camera effect (eye). Lens effects (spikes, flare ghosts, grain, vignette, chromatic fringe) are separate passes, off in True, for recipes, trailers and Orac.

**Structure:** one shared "finish" module that every GPU view calls, and every GPU view brought under recipes and the governor (finding 2.3.2). This is the precondition for gating anything else.

### 5.2 The sky behind everything

System True is black. SpaceEngine never shows black: there is always a star field and the galaxy's band.

- **Have:** the kernel's `sky_from_here` and `sky_glow` (Y2, Y3), and the Window's "stars behind" (Y5).
- **Do:** bake a sky cube map per vantage (once per system, cheap) and draw it behind every True view, dimmed by the exposure. Stars as point sprites sized by brightness, coloured by temperature, capped at 20,000 on B and 200,000 on H (already agreed for Y5).
- **Exposure (changed by Alex's ruling, 16.7):** at the exposure that shows a sunlit planet, most stars vanish to a real eye. The default is "Long exposure", which shows both and says it is a composite; "Eye" is the choice that shows what an eye would see.
- Grade: fact (it is the kernel's sky). Tier B. Phone: one cube read, 0.3 ms.

### 5.3 A world's face from space

| Item | Today | Options | Recommended | Tier |
|---|---|---|---|---|
| Picture size | 128 x 64, made on the processor | Bigger fixed; a cube map sized to the world's pixels on screen, re-baked as it grows | **A texture ladder:** cube faces from 64 to 2048 pixels chosen so one texel is about one screen pixel, baked on the GPU from the kernel's tiles, cached. This is the fact rung of the detail ladder | B to 512, H to 2048 |
| Relief | Hill shading on the globe; none in system True | Normal map from the height tiles | Normals from heights, so craters and ranges throw shade at the terminator. The cheapest drama in the list | B |
| How ground reflects | Lambert | Lommel-Seeliger with an opposition surge for airless dusty ground; Lambert under thick air; mix by the kernel's surface kind | As described. The full Moon then looks flat and bright to its edge, as the real one does | B, free |
| Long shadows | none | Horizon angles baked per tile; shadow maps | Horizon angles baked once per tile on the GPU | H |
| Oblateness | spheres (TA6 queued) | Scale along the spin axis from the kernel's flattening | TA6 as planned | B |
| Detail below the tiles | none | The detail ladder (6) by material | Second grade, from material class, surface age and gravity | B one rung, H three |

### 5.4 Giants: bands, storms and the swirls Alex asked for

**Have:** the kernel's bands, storm families, polar regimes and polygon jets (W3), drawn as decks (TA1). Colours wait on the data sheet (TA3, TA4).

**What makes Jupiter look like Jupiter**, in order: the colours (TA3); the bands drifting at different speeds; curls and festoons along the edges between bands; storms that turn; the dimmer, yellower edge of the disc.

| Layer | Driven by (kernel) | Options | Tier | Phone |
|---|---|---|---|---|
| **Differential rotation**: each band slides at its own speed | The jet speed by latitude (to add to the `weather` op if not there: kernel ask K-a) | Longitude offset per latitude in the shader. The equator then laps the temperate bands as Jupiter's does | B | free |
| **Flow-carried detail**: the band picture is carried along a wind field | Jet profile plus each storm's place, size and spin sense | Flow map with two-phase blending: two extra reads | B | 0.4 ms |
| **Curls at band edges**: detail octaves added as the giant grows on screen | Shear between neighbouring jets sets strength; the Rhines scale (already computed in `sse-climate`) sets the largest curl, worked from the local eddy speed and not the jet speed (section 17); the deformation radius sets storm size; spin direction sets which way storms turn in each hemisphere | Curl noise with rotating gradients, one to four octaves by screen size (the detail ladder) | B 1 octave, H 4 | 0.3 to 1.2 ms |
| **Storms that turn** | Storm list (W3): centre, radii, spin sense, drift | A rotating warp inside each storm's ellipse | B | 0.2 ms |
| **Advected particles**: the Gaseous Giganticus look, live | Same wind field | Compute shader, 1 to 4 million particles into a cube map, updated a slice per frame | **U** | not on phones |
| **Edge of the disc**: dimmer and yellower; brighter at the edge in methane's absorption bands | Gas column and haze from the weather op, through `light_through_gas` | A look-up by the cosine of the view angle | B | free |
| **Lightning on the night side** | `lightning_km2_yr` per cell (in the weather contract) | Brief points of light where the rate is high, sharing the "three flashes a second" cap | B | free |

Grades: bands, storms, drift and edge colour are facts. Curls are drawn from the numbers. HUD line: "Curls smaller than the bands are drawn from the wind's figures."

The same three layers (drift, flow-carried detail, curls by shear) serve **brown dwarfs** (R28), **Venus's cloud-top streaks** and **hot Jupiters' day-to-night flow**, with different numbers. One mechanism.

### 5.5 Air: the rim, the terminator and the view from inside

**Have:** single scattering from `read_air_optics`, ray-marched on the globe and the ground; nothing in system True (TA5 queued).

| Item | Options | Recommended | Tier |
|---|---|---|---|
| Model | Today's per-pixel march; Hillaire's look-up pictures (transmittance, multiple scattering, sky view, aerial perspective) | **Port Hillaire to TSL**, fed by `read_air_optics`. Cheaper per frame than today's march and adds multiple scattering, which is what gives thick air its soft glow. Look-ups are rebuilt only when the air or the sun changes | B (small look-ups), H (full) |
| Rim in system True | none | A thin shell drawn from the same look-ups; at a few pixels it collapses to a coloured rim term | B |
| Terminator | grey fade | Comes free from transmittance: the red band at sunset, twilight reaching past the edge by an amount set by the scale height | B |
| Hazes | white | Colour from the sheet's absorption rows (TA3); a second thin layer at height for a detached haze when the weather op gives one (Titan, Pluto) | B |
| Particle phase | one Henyey-Greenstein lobe | **A phase table per kind of particle, computed by the kernel from the particle's size and the sheet's refractive index** (Mie theory, once per kind, a small look-up). Jendersie and d'Eon's fit only for water droplets of 5 to 50 µm, which is all it was fitted for. One Henyey-Greenstein lobe is not good enough for dust (section 17) | B |
| Light shafts | none | Shadowed march at half size | U |

### 5.6 Clouds

| Form | What it is | Cost | Tier |
|---|---|---|---|
| Painted layer (today) | Today's cloud field laid on the ground | free | B |
| **Layer at height** | The same field on a shell at the cloud-top height: it shifts against the ground as you turn the globe, and **its shadow falls on the ground offset by the sun's angle and the real height** (one extra read along the sun's direction) | 0.3 ms | B |
| Flow and curls | The field carried by the kernel's wind, with detail octaves from the ladder; storms spiral the way the world's spin dictates | 0.5 ms | B |
| **Short march through a shell** | 8 to 16 steps through a thin slab shaped by the field plus cell noise, self-shadowed with 2 light steps, at half size (R2b part 2, TA5) | 2 to 4 ms desktop | H |
| **Full volumetric** | Nubis-style: quarter size, rebuilt over frames, shadow map, powder term, light shafts; as you descend through them on a landing | 4 to 8 ms desktop | U |

Grades: where clouds are, how thick and how high are facts. Puff shapes are drawn from the numbers (cloud kind from the weather op picks the noise: sheets, puffs, streets, anvils).

### 5.7 Seas, lakes and ice

- **Glint from orbit:** a bright patch at the mirror point whose width follows Cox and Munk from the kernel's wind over that sea. Calm seas give a tight spark; rough seas a broad glow. Titan's seas, nearly waveless, give a pinpoint. Tier B, free.
- **Colour:** from the liquid's absorption and depth (climate thread, already ruled). Shallow shelves lighter. Tier B.
- **Whitecaps and foam** from wind above a threshold (already ruled). Tier B.
- **Ice:** a mirror-like sheen on smooth young ice, none on old cratered ice (surface age from the tiles). Cracks from cell noise below the fact floor, aligned with the kernel's stress direction. Tier B.
- **Wave field up close:** today's wave trains (V10k) are right for B and H. A spectrum-driven field is U.

### 5.8 Shadows and borrowed light between bodies

The gloss item the eye misses most after the finish, and nearly free because spheres and rings have exact formulas.

| Item | How | Tier | Phone |
|---|---|---|---|
| A moon's shadow on its planet, with a soft edge | The share of the star's disc hidden by the moon, seen from that point: a closed formula for two overlapping discs | B | 0.1 ms per shadowing body, capped at 4 |
| The planet's shadow across its rings, and across a moon | The same formula | B | same |
| The rings' shadow on the planet | Trace to the ring plane, read the ring's opacity row | B | 0.1 ms |
| **The colour of light inside a shadow** | A world with air bends red light into its own shadow. One colour per world from the kernel's air (`light_through_gas` along a grazing path): an eclipsed moon turns copper round an Earth, and stays black round an airless world | B | free |
| Planet-shine and ring-shine | The night side of a moon lit by its planet: planet's brightness x its phase x how big it looks from there. One extra dim light per moon | B | free |
| Self-shadowing terrain | 5.3 | H | |

All facts.

### 5.9 Rings and belts

**Have:** edges, opacity, albedo, tilt, lit as a layer of particles; belts as members on real orbits (V10f, V10l).

| Item | Driven by | Tier |
|---|---|---|
| Shadows both ways (5.8) | geometry | B |
| **The dark side**: seen from below, dense rings are dark and thin ones glow | Opacity; the transmitted-light twin of `ringLight` | B |
| **Opposition surge**: brighter when the sun is straight behind you | Phase angle | B |
| **Forward scatter**: dusty rings flare when backlit | Particle size (kernel ask K-c) through the phase fit | B |
| **Structure at resonances**: a gap or a wave train where a moon's period ratio says it must be | Moons' orbits: radius = moon's orbit x ((m−1)/m)^(2/3). Computed, then drawn; fine ringlets below that are drawn from the numbers | B |
| Wavy gap edges beside a moon inside the rings | Wavelength = 3π x the gap's half-width | H |
| **Particles up close** | Size distribution; thousands of lit, tumbling lumps near the camera fading into the sheet | H (instanced), U (shadowed, with depth of field) |
| Belts | Members with size-sorted brightness; a faint dust band | B |

### 5.10 Small bodies and comets

- **Shape from the body's own numbers:** small ones lumpy (a ray-marched or displaced shape seeded per body), larger ones rounded by their gravity at the size the kernel's strength figures give; **fast spinners take a top shape with a ridge at the equator**; some are two lobes in contact. Tier B (displaced sphere), H (true shape with self-shadow).
- **Boulders and dust ponds** up close from the detail ladder, by gravity and age. Tier H.
- **Comets:** a coma that switches on inside the distance where its ice sublimes, which depends on the ice (about 3 au for water, 12 for carbon dioxide, 50 for carbon monoxide, checked), so a comet rich in the colder ices is active far out; a dust tail that curves back along the orbit; a straight, bluer gas tail pointing exactly away from the star; **green in the coma only**. Soft particles at half size. Tier B (billboards), H (soft particles).
- **Plumes** (TRUE2): brighten strongly when backlit, by particle size. Tall umbrella shape on airless worlds from the kernel's ballistic height. Tier B.

### 5.11 The night side

All tier B and nearly free, because each is an emissive term feeding the shared glare:

- **Hot ground and lava** by temperature (TRUE3): Planck against the exposure, so a lava lake is dull red at the edges and yellow-white in fresh cracks. Crust pattern from cell noise (drawn from the numbers).
- **City lights** where the kernel's holdings put people (R22), in a colour the pack sets.
- **Aurora ovals** at the kernel's latitudes (MF9), in colours from the air's gases (MF9 part 2: oxygen green and red, nitrogen blue-violet, hydrogen pink).
- **Lightning** flickers (5.4).
- **Night glow of thin air at the rim** only if the light thread supplies a figure; otherwise not drawn.

### 5.12 Stars, beyond SG1 to SG3

- **Fast spinners:** flattened, with bright hot poles and a dim cooler equator (gravity darkening). Free. Tier B.
- **Close pairs:** a star filling its lobe drawn as the teardrop it is. Tier B.
- **Corona:** only visible when the disc is hidden (by a world, or a GM's "coronagraph" switch); streamers along the equator at a quiet point in the cycle, all round at an active one. Tier B.
- **Spots that drift** at different rates by latitude. Free.
- **Prominences** (SG6) when the kernel lists them.
- **Route SG1's glow through the shared glare** (5.1) so stars, lava and glints agree and nothing is counted twice.

### 5.13 Standing on the ground

The Land view is the furthest from SpaceEngine today: a smooth plain. In order of gain:

| Item | How | Grade | Tier | Phone |
|---|---|---|---|---|
| **Ground detail** | Detail normal and colour from a small set of generated material textures (rock, regolith, sand, ice, salt, lava, sediment), chosen by the tile's material class, mixed by slope and height, laid on with triplanar mapping and hex-tiling so nothing repeats. Detail octaves by distance | drawn from the numbers | B (2 materials), H (4, with parallax) | 1 ms |
| **Stones and boulders** | Instanced, scattered by Poisson disc; sizes from surface age and gravity (old airless ground is cratered and dusty; young lava is blocky) | drawn from the numbers | H | |
| **Small craters below the floor** | The kernel's size-frequency slope continued downward | drawn from the numbers | B | 0.3 ms |
| **Air between you and the hills** | Hillaire's aerial perspective look-up (5.5). On an airless world the far hills stay razor sharp, which is its own effect | fact | B | |
| **The sun as a disc with glare**, flattened near the horizon by the air's bending | Angular size from the orbit; bending from air density | fact | B | free |
| **Shadows** | Cascaded shadow map near the camera; baked horizon angles far away | fact | H | |
| **Stars by day on thin-air worlds; twinkling by air density; planets steady** | Sky brightness against star brightness at the current exposure | fact | B | free |
| **Rainbows and halos** | Bow radius from the refractive index of what is raining (data sheet); only when the weather op has rain and a low sun | fact | H | |
| **Blowing dust, snow, rain streaks** | Particles by the weather op's rates | drawn from the numbers | H | |
| **Living cover** (parked with the life track) | | | later | |
| True displacement of the near ground | Parallax occlusion in the pixel shader (no tessellation in WebGPU) | drawn | U | |

### 5.14 Nebulae, galaxies and the web

- **Nebulae:** a ray-march at half size through a density made from the kernel's shape (shell, lobes, remnant) plus curl-noise wisps, lit in the real line colours from the light thread's line table (hydrogen red, oxygen teal, sulphur deep red), with dust in front absorbing. Tier H; B draws layered billboards from the same density; U adds scattering of the central star's light.
- **Galaxy True:** dust lanes as absorbing layers, pink star-forming knots along arms, a yellow bulge, blue young arms. Mostly there (V9t); the gain now is the finish (5.1) and a particle layer of bright stars. Tier B.
- **Web:** additive filaments with depth fade so it reads as a 3D spiderweb (Alex's ruling). The finish helps most. Tier B.

### 5.15 Light bending

Designed in `SSE4-exotic-objects-design.md` (a bend table from the kernel). Nothing to add except that it should read the sky cube of 5.2, and that the disc's bright approaching side comes free from the light thread's Doppler transform (O7).

### 5.16 Ships and stations

- **Lit by their surroundings:** reflections and fill light from the same sky cube plus the nearby world's lit face. Removes the "model in a void" look. Tier B.
- **Drive plumes that know the pressure around them:** a narrow pencil with shock diamonds in thick air, a wide fan in vacuum. Driven by chamber and ambient pressure. Tier B.
- **Radiators glowing by temperature** (done) now bloom through the shared glare.
- **Ambient occlusion** from the three.js node. Tier H.

### 5.17 Motion and the camera

Part of what reads as polish in NASA Eyes is how the camera moves.

- Eased flights along a curve between bodies, never a cut (the interface style's motion rules apply; none under reduced motion).
- Worlds turning on the universe clock in True (the renderer's status notes worlds still stand still in True).
- A light motion blur on fast flights: camera effect, tier H, off under reduced motion.
- Depth of field in "Take photograph" only.

---

## 6. The detail ladder: one mechanism for "more as you close in"

Alex's example ("more swirls when close up") is one case of a general need. The mechanism:

1. **Rungs.** A layer's picture is built from rungs, each covering a range of sizes on the body. The top rungs are **facts** (kernel tiles, the weather field, the ring profile). Below the kernel's resolution, rungs are **drawn from the numbers**.
2. **Which rungs are drawn** depends only on how many screen pixels the body covers: a rung is added when its features would be two pixels or more, and faded in over a factor of two so nothing pops. Far away only the facts are drawn. Cost per pixel stays roughly constant, because coarse rungs drop out as fine ones arrive.
3. **Each drawn rung is a noise with a declared recipe:** kind (curl, cell, ridged, flow), slope of its spectrum, stretch and direction, amplitude, and speed. **Every one of those five is bound to a kernel value** by the layer's declaration, for example:

| Layer | Largest drawn size | Direction | Amplitude | Speed |
|---|---|---|---|---|
| Giant's curls | Rhines scale | along the jets | shear between neighbouring jets | jet speed |
| Cloud puffs | cloud kind's cell size from the weather op | wind | cover and convective strength | wind |
| Ground roughness | the fact floor | downslope | material class and surface age | none |
| Small craters | the floor | none | age (the kernel's size-frequency slope) | none |
| Ice cracks | shell thickness | the stress direction | tidal strain | none |
| Ring fine structure | the profile's sample spacing | round the ring | opacity | Kepler rate |
| Star cells | already real (SG2) | | | |

4. **Seed and clock** from the kernel (section 3, rule 2).
5. **The tier sets how many drawn rungs** are allowed: B one, H three or four, U as many as fit, S all.
6. **The HUD's detail line** names the size where facts end, in the body's own units.

The ladder is a small shared module (a rung table plus the noise nodes) in the renderer. Each area thread supplies its layer's declaration and never writes its own noise. Declarations are data, so Orac can tune them and a pack can add one.

---

## 7. What stays out, on purpose

- No picture textures of real planets pasted on procedural ones. Real bodies get real data as pins (existing ruling).
- No invented feature that looks like a fact (a second Great Red Spot from noise). Curls yes, named storms no.
- No lens flare, spikes or grain in True by default.
- No effect that hides a kernel caveat (Pluto's too-thick air must look too thick until it is fixed).
- No copied code from GPL, AGPL or NonCommercial sources.

---

## 8. Quality tiers and performance gates

Alex: heavy techniques are welcome if each is gated so weak machines stay supported.

### 8.1 The tiers

The code has five (`text`, `canvas`, `gpu-basic`, `gpu-high`, `still`). This note adds one, **`gpu-ultra`**, and changes how a device is placed.

| Mark | Tier | Who gets it | Frame budget | Picture size |
|---|---|---|---|---|
| | `text`, `canvas` | no usable GPU | as today | |
| **B** | `gpu-basic` | every GPU device, WebGL2 included, phones | 33 ms | 0.5 to 0.77, enlarged |
| **H** | `gpu-high` | devices whose **measured** speed fits; WebGL2 allowed | 16.7 ms | 1.0 |
| **U** | `gpu-ultra` | WebGPU with compute, and measured headroom | 16.7 ms | 1.0 |
| **S** | `still` / at rest | any GPU device while the view is not moving | builds over 0.5 to 3 s | 1.0 or more |

### 8.2 What is measured, and when

| Check | When | What | Used for |
|---|---|---|---|
| **Capabilities** | every page load, instant | WebGPU or WebGL2; compute; half-float colour targets; 3D textures; largest texture; timestamp queries; memory hint; screen pixels; reduced motion; data saver | Which passes **can** run. A pass lists its `needs`; a pass whose needs are unmet does not exist on that device |
| **Probe** | first visit, and again when the graphics adapter or browser version changes; about half a second, off screen; "Measure again" in More › Performance | Three tiny scenes at one megapixel: **fill** (many texture reads), **march** (a 24-step ray march), **geometry** (300,000 triangles). Timed by timestamp query where present, otherwise by timing frames closed with a one-pixel read-back | Three device factors that scale every pass's declared cost (the recipe's `PassCost` already has the slots). Cached on the device |
| **Placement** | after the probe | Predicted cost of each tier's standard True recipe at this view's pixel count | Start at the richest tier whose prediction is within 60% of its budget (the governor's own "well under" margin) |
| **Governor** | every frame (built, R1) | Smoothed frame time | Walks the recipe's degrade order down when over budget, back up slowly |
| **Heat** | running | Three steps down inside a minute | Lowers the ceiling for the session, so a hot phone stops trying |
| **Bench** | on demand (`/harness/`) | The three probe scenes plus each standard recipe | A "Drawing" row per tier in `docs/minimum-spec.md` |

The user can always override (More: Quality Auto, Light, Full, Ultra); the badge then says the choice is theirs. Nothing is sent anywhere (as the performance readout already promises).

### 8.3 Rules for a pass that hammers the GPU

1. It declares its needs and its cost per megapixel on the reference device for each tier it runs on.
2. It is **never the only way to see a fact.** Every U pass has a B form that shows the same facts more plainly (volumetric clouds over a cloud layer; particle giants over flow-mapped giants).
3. It sits **first** in the degrade order, with a plain-words line for the badge ("Cloud depth simplified: this device's graphics are at their limit").
4. It is off until the probe has placed the device at U.
5. Its cost shows as its own line in the performance readout.

### 8.4 Sharpen at rest (the leveller)

People look far more than they fly. When the view has not moved for half a second, every True view should keep refining: jittered samples accumulated for smooth edges, more march steps, full-size clouds, more ladder rungs, softer shadows. On input it drops back at once. The `still` tier and "Take photograph" (R3) already do this on a button. Making it automatic means **a phone shows close to the Ultra picture two seconds after you stop**, which closes most of the phone gap without a frame of extra cost while moving. Under "reduce motion" the refinement fades in rather than shimmering.

### 8.5 What each tier switches on

| Pass | B | H | U | S |
|---|---|---|---|---|
| Finish (5.1) | quarter-size glare, FXAA, enlarged | half-size glare, MSAA | same | accumulated |
| Sky behind (5.2) | 20,000 stars | 200,000 | same | same |
| Texture ladder (5.3) | to 512 | to 2048, horizon shadows | same | to 4096 |
| Drawn rungs (6) | 1 | 3 to 4 | 6 | all |
| Giants (5.4) | drift, flow map, 1 curl octave | 4 octaves | advected particles | + |
| Air (5.5) | small look-ups, rim | full Hillaire | light shafts | more samples |
| Clouds (5.6) | layer at height with shadow | short march | full volumetric | full, full size |
| Shadows between bodies (5.8) | all | all | all | soft terrain shadows |
| Rings (5.9) | sheet, both sides, resonances | particles near camera | shadowed particles | + |
| Ground (5.13) | 2 materials, small craters | 4 materials, stones, shadows, parallax | displacement | + |
| Nebulae (5.14) | billboards | half-size march | scattering | full size |

**Phone budget:** all B flourishes together must fit in **8 ms of the 33 ms frame** on the reference phone at 0.75 scale **[estimate]**. Section 9 gives each item's share; they sum to about 6.5 ms, and no single view shows them all at once.

---

## 9. The ranked list: how much of the gap each item closes, per cost

"Gap closed" is how much nearer a side-by-side with SpaceEngine becomes (5 is most). "Build" is the size of the work (S a day or two, M several, L a week or more, for a builder thread). Owner names are the live builder threads (section 12).

| # | Item | Gap closed | Build | Tier | Phone ms | Grade | Owner |
|---|---|---|---|---|---|---|---|
| 1 | **Frame finish** in every True view: high range, exposure, shared glare, AgX, dither (5.1) | 5 | M | B | 1.9 | camera (eye) | Photoreal |
| 2 | **Stars and the galaxy's band behind every True view** (5.2) | 5 | S | B | 0.3 | fact | Photoreal |
| 3 | **Texture ladder and relief normals** on worlds (5.3) | 5 | M | B | 0.3 | fact | Photoreal |
| 4 | **Giants: drift, flow and curls** (5.4), Alex's example. Needs TA3 colours to shine | 5 | M | B, H | 0.9 | fact + drawn | Giant bands |
| 5 | **Air's rim and terminator in system True; Hillaire look-ups** (5.5, TA5) | 4 | M | B | 0.6 | fact | Giant bands |
| 6 | **Shadows and borrowed light between bodies** (5.8) | 4 | S | B | 0.4 | fact | Photoreal; rings part by Asteroids and rings |
| 7 | **Ground detail on Land** (5.13 rows 1 to 4) | 4 | L | B, H | 1.3 | drawn | Photoreal |
| 8 | **Clouds at height with shadows and flow** (5.6 B rows) | 4 | S | B | 0.8 | fact + drawn | Giant bands |
| 9 | **Everything under the governor, the probe, sharpen at rest, enlarging** (8) | 4 (on phones 5) | M | all | saves | | Photoreal |
| 10 | **Airless ground reflects like dust** (5.3) | 3 | S | B | 0 | fact | Photoreal |
| 11 | **Night side: lava, cities, aurora, lightning** (5.11) | 3 | M | B | 0.2 | fact | Plumes and hot glow (lava); Photoreal (the rest); Light (aurora colours) |
| 12 | **Rings: dark side, surge, forward scatter, resonances** (5.9) | 3 | M | B | 0.2 | fact | Asteroids and rings |
| 13 | **Sea glint from orbit by the wind; ice sheen** (5.7) | 3 | S | B | 0 | fact | Photoreal, numbers from Climate |
| 14 | **Plumes and comets with real scattering and colours** (5.10) | 3 | M | B, H | 0.4 | fact | Plumes and hot glow |
| 15 | **Short-march cloud shell** (5.6 H) | 3 | M | H | | fact + drawn | Giant bands |
| 16 | **Stars: spin shape and bright poles, corona when hidden, shared glare** (5.12) | 2 | S | B | 0 | fact | Star glow |
| 17 | **Ships lit by their surroundings; pressure-aware plumes** (5.16) | 2 | S | B | 0.2 | fact | Photoreal |
| 18 | **Ring particles and asteroid shapes up close** (5.9, 5.10) | 3 close up | L | H | | fact + drawn | Asteroids and rings |
| 19 | **Nebulae as lit volumes in line colours** (5.14) | 3 where seen | M | H, B plain | | fact + drawn | Light and nebulae (facts), Photoreal (drawing) |
| 20 | **Camera: eased flights, worlds turning in True** (5.17) | 2 | S | B | 0 | | Photoreal |
| 21 | **Sky from the ground: sun disc, twinkle, stars by day, rainbows** (5.13) | 2 | M | B, H | 0 | fact | Photoreal; Light (angles) |
| 22 | **Full volumetric clouds, light shafts** | 3 | L | U | | | Giant bands |
| 23 | **Advected-particle giants** | 2 over item 4 | M | U | | drawn | Giant bands |
| 24 | **Lens effects as recipe passes** (spikes, flare, grain) for trailers and Orac | 1 in True; high in trailers | S | B | 0.3 | camera (lens) | Photoreal |

**Reading the list.** Items 1 to 3 and 9 are plumbing that everything else stands on, and they are also the three biggest visible gains. Item 4 is Alex's example and can start at once beside them, because it lives inside the giant's own material. Items 5 to 14 are the "years of tuning" list and are each small. Items 22 and 23 are the only ones that hammer the GPU.

---

## 10. Cheap physics wins an expert can test

Each is a few lines of shader or one small formula block, driven by numbers the kernel has or nearly has. Each deserves its block in the Workshop and its line in Explain, because the point is that a clever user can check it.

| # | What the expert sees | The number behind it | Where |
|---|---|---|---|
| 1 | The full Moon is flat-bright to its edge, not shaded like a ball | Dusty ground reflects by Lommel-Seeliger: at full phase every point returns the same | 5.3 |
| 2 | A sharp brightening of airless worlds and rings when the sun is exactly behind you | Opposition surge: the Moon brightens by more than 40% between 4 degrees and 0; Saturn's rings by about a third inside half a degree (checked, section 17) | 5.3, 5.9 |
| 3 | The glint on a sea widens with the wind; Titan's is a pinpoint | Cox and Munk: slope variance = 0.003 + 0.00512 x wind speed in m/s, measured 12.5 m above the sea, good for 1 to 14 m/s (checked, section 17) | 5.7 |
| 4 | An eclipsed moon is copper round a world with air, black round one without | Light bent and reddened through the limb of that air, 10,000 to a few 100,000 times fainter than full; an ozone layer adds a blue-turquoise rim to the shadow (checked, section 17) | 5.8 |
| 5 | The dark limb of a crescent moon faintly lit by its planet | Planet-shine from the planet's phase and apparent size | 5.8 |
| 6 | Seen from their dark side, the densest rings go dark and rings of middling thickness glow. Which ring is brightest shifts with the sun's and the viewer's height above the ring plane | Transmitted single scattering through a layer of that opacity: it peaks at an optical depth set by the two angles (0.41 for sun at 20 degrees and viewer at 30) (checked, section 17) | 5.9 |
| 7 | A ring's big gap sits where a moon's 2:1 resonance says | radius = moon's orbit x (1/2)^(2/3). Mimas gives 116,900 km against the real B-ring edge at a mean 117,570 km. The real edge also breathes in and out by 4 to 71 km over about 5.4 years, which a later package could draw (checked, section 17) | 5.9 |
| 8 | Faint dusty rings and plumes flare when backlit and nearly vanish front-lit | Forward scattering by grains near the light's wavelength | 5.9, 5.10 |
| 9 | A giant's equator laps its temperate bands | Jet speed by latitude (Jupiter's two rotation periods differ by about five minutes) | 5.4 |
| 10 | Storms turn opposite ways in the two hemispheres, and backwards on a world that spins backwards | Sign of the Coriolis term from the spin | 5.4, 5.6 |
| 11 | Curl size on a giant changes with its spin and wind | Rhines scale from the local eddy speed (10 to 40 m/s on Jupiter), which matches the real band widths within a factor of two (checked, section 17) | 5.4 |
| 12 | Cloud shadows sit off to one side of their clouds by the right amount | Cloud-top height and the sun's angle | 5.6 |
| 13 | Sunset on a dusty thin-air world is blue round the sun in a butterscotch sky | Forward scattering by dust of the kernel's grain size | 5.5 |
| 14 | A fast-spinning star is flattened with bright poles and a dim equator | Gravity darkening: temperature follows local gravity to a power that is an input of the block, 0.25 only for slow spinners with radiative envelopes, about 0.19 measured on fast ones, 0.08 for convective envelopes. Test: Altair, 8,450 K at the poles and 6,860 K at the equator (checked, section 17) | 5.12 |
| 15 | A comet is green in its head and not in its tail | The green molecule (C2) is broken by sunlight in about two days at 1 au, before it gets far (checked, section 17) | 5.10 |
| 16 | A comet's gas tail points straight from the star; its dust tail curves | Radiation pressure against orbital motion | 5.10 |
| 17 | A small fast-spinning asteroid is a spinning top with a ridge | Spin near the rate where rubble flows to the equator: a period of about 3.3 hours divided by the square root of its density in g/cm3 (3.5 h for Ryugu's 1.19) (checked, section 17) | 5.10 |
| 18 | The setting sun is squashed; stars twinkle and planets do not; thin-air worlds show stars by day | Refraction and turbulence from air density; angular size | 5.13 |
| 19 | A rainbow in methane rain is wider than one in water | Bow angle from the liquid's refractive index (about 49 degrees for methane against 42 for water, moving between 47 and 52 with temperature and colour (checked, section 17)) | 5.13 |
| 20 | A drive plume is a pencil in thick air and a wide fan in vacuum | Pressure ratio at the nozzle | 5.16 |
| 21 | Beside a sunlit planet the stars vanish; shade the planet and they return | One honest exposure | 5.1, 5.2 |
| 22 | Aurora colours change with the air's gases | Emission lines of what is there | 5.11 |
| 23 | Wavy edges on a ring gap next to a small moon, with the right wavelength | 3π x the gap's half-width | 5.9 |
| 24 | Lava glows dull red at the cool edges and yellow-white in fresh cracks | Planck's law against the exposure | 5.11 |

Wins 1, 3, 4, 6, 7, 9 and 21 are the ones to show first: each is a single screenshot that someone who knows the subject will stop on.

---

## 11. Could the long physics calculations move to the GPU?

Alex: "Not sure, but bear in mind." This section weighs it. Alex accepted the recommendation in 11.4 on 24 Nov (question 10).

### 11.1 What is long today

From the bench in `docs/minimum-spec.md` (test workloads, medians): making and ageing a system about 1 s on AC-ProArt and 2 s on the phone; surfaces, caves and descents 1.3 s and 3.1 s; galaxy and web 1.5 s and 2.8 s; orbits, routes and voyages 3.0 s and 6.2 s. On a slow machine these are several times longer.

### 11.2 The catch

The kernel's founding promise is that every device computes the same bits: no wall clock, no std transcendental maths, no fused multiply-add, golden hashes that change only on purpose. A GPU breaks that promise for floating-point work in three ways:

1. **No 64-bit floats** in WebGPU or WebGL2 shaders. The kernel is f64.
2. **Float results are not guaranteed identical across graphics cards.** Checked against the specifications (section 17): in WebGPU's shading language add, subtract and multiply are correctly rounded but with no stated rounding mode; divide may be off by 2.5 units in the last place and square root by 2; sine and cosine by far more; drivers may reorder and fuse operations with no way to forbid it; very small numbers may be flushed to zero. A 2024 study of 600,000 test programs found NVIDIA and AMD cards disagreeing in 9% of single-precision runs. Two cards can differ in the last bit, and an erosion run of a million steps turns a last bit into a different river.
3. **Compute shaders exist only on WebGPU.** Every WebGL2 device (Alex's phone today) would need the processor path anyway, and the two paths would have to agree exactly.

One thing does stay exact: **32-bit integer arithmetic on the GPU is bit-identical everywhere.** That is the only honest road to GPU facts.

### 11.3 The options

| Option | What | Gain | Cost and risk | View |
|---|---|---|---|---|
| **A. GPU for drawing and preview only** | Everything below the fact floor, the detail ladder, particle giants, texture baking from tiles, and a fast preview of a surface while the kernel computes the real one (shown as a preview, replaced when the facts arrive) | Large for how things look and feel; no waiting on a blank globe | None to determinism | **Do now. It is sections 5 to 9** |
| **B. More processor first** | (1) A pool of kernel workers: tiles, bodies and route candidates are independent by design ("any tile builds without its neighbours"), so four workers give nearly four times the speed with identical bits. (2) WebAssembly SIMD: its fixed-width operations are exact, so results stay identical (not the "relaxed" SIMD set). (3) Threads inside one computation later; they need the site to be cross-origin isolated, a hosting change that affects embedded outside content | Likely 3 to 8 times on the long chunks **[estimate]** | Low. Goldens prove nothing changed. Reduction order must stay fixed | **Do before any GPU physics.** Kernel thread |
| **C. Integer physics on the GPU, with an identical processor twin** | Rewrite one grid-shaped block (stream-power erosion and flow routing are the best fit; crater stamping; the web's density grid) in fixed-point integers, in a compute shader and in Rust, proven equal by golden tiles | Perhaps 10 to 50 times on that block on a good card **[estimate]** | High: fixed-point rewrites change today's results (a deliberate golden change), two implementations to keep equal for ever, no gain on WebGL2 devices, and 32-bit integers make wide multiplies awkward | **One measured experiment later**, on erosion, only if B leaves surfaces too slow |
| **D. Float physics on the GPU, the GM's machine as the authority** | Compute once on one machine, stream the result as data | Fast | Breaks "regenerated from the seed, never stored": a campaign reopened on another device gets different ground unless results are stored and shipped. A founding rule changes | **Not recommended** |
| **E. GPU for the gravity sum in formation** | Direct-sum n-body in a compute shader | Large for big particle counts | Chaotic, so last-bit differences become different systems; same objection as D | **Not recommended**; B (SIMD) suits it |

Universe Sandbox, a native program with a large team, moved its physics to many processor cores in December 2025 rather than to the GPU (their own post). That is some evidence about where the easy gains are.

### 11.4 Recommendation, to bear in mind

A now. B as the kernel's next performance work when a chunk misses its budget. C as a single experiment with a stated pass mark (identical tiles on three different graphics cards and the processor twin, and at least ten times faster), before any wider decision. D and E no.

---

## 12. Packages, by owner thread

Small packages a builder can take one at a time. Each ends with its checks green, screenshots in project files under `sse4/handover-screens/VP<n>-screens/`, its fudge and citation rows, and its row in the owner's status file. None changes a golden unless marked. "Needs" names what must land first.

### Build photoreal renderer (`status/renderer.md`)

| Package | What | Needs | Tier |
|---|---|---|---|
| **VP1** | The shared finish module (5.1): high-range target, metered exposure, glare with SG1's curve, AgX, dither; used first by system True | | B |
| **VP2** | Sky cube behind every True view (5.2) from `sky_from_here` and `sky_glow` | VP1 | B |
| **VP3** | Texture ladder and relief normals (5.3): GPU bake from tiles into cube faces by screen size | | B, H |
| **VP4** | System True, Land, ships and stations under recipes and the governor; the probe (`probe.ts`), placement, `gpu-ultra`, per-pass lines in the performance readout, the GPU bench rows (8.1 to 8.3) | VP1 | all |
| **VP5** | Sharpen at rest in every True view; FSR1-style enlarging on B (8.4, 5.1) | VP4 | S, B |
| **VP6** | The detail ladder module (6): rung table, noise nodes (ported MIT simplex with rotating gradients, curl, cell, ridged), seed and clock, HUD detail line | | B |
| **VP7** | Airless ground reflectance and opposition surge as a formula block with its renderer use (5.3) | | B |
| **VP8** | Shadows and borrowed light between bodies (5.8), except the ring pair | VP1 | B |
| **VP9** | Sea glint from orbit, ice sheen (5.7) | Climate's wind over sea in the tiles or the weather op (ask K-b) | B |
| **VP10** | Land: ground materials, hex-tiling, small craters, air between you and the hills (5.13 rows 1, 3, 4) | VP6; Hillaire look-ups from VP-A3 | B, H |
| **VP11** | Land: stones, shadows, sun disc and glare, sky effects (5.13) | VP10 | H |
| **VP12** | Night side: city lights, aurora ovals in Visible, lightning (5.11) | VP1; Light's aurora colours (VP-L2) | B |
| **VP13** | Ships and stations lit by the sky cube and the near world; pressure-aware plumes (5.16) | VP2 | B |
| **VP14** | Camera flights, worlds turning on the clock in True (5.17) | | B |
| **VP15** | Nebula volumes drawn from the light thread's shapes and lines (5.14) | VP1, VP6, VP-L1 | H |
| **VP16** | Lens passes for recipes (spikes, flare, grain, vignette), off in True | VP1 | B |

Order: VP1, VP2, VP3 first (the three biggest gains), VP4 and VP6 beside them (they unblock the other threads), then by the ranked list.

### Build giant bands, clouds and hazes (`status/true-air.md`, PR 444)

| Package | What | Needs | Tier |
|---|---|---|---|
| **VP-A1** | Differential rotation and turning storms on giants (5.4) | kernel ask K-a if the jet profile is not in the `weather` op | B |
| **VP-A2** | Flow-carried detail and curls on giants through the ladder; the same on brown dwarfs and Venus-like decks | VP6, VP-A1 | B, H |
| **VP-A3** | Hillaire look-ups in TSL from `read_air_optics`; the rim and terminator in system True (extends TA5) | | B, H |
| **VP-A4** | Cloud layer at height with offset shadows and wind flow (5.6 B rows) | VP6 | B |
| **VP-A5** | Phase tables per particle kind from the kernel (size and the sheet's refractive index), used by clouds, hazes, dust, plumes and rings alike; the Jendersie and d'Eon fit only for water droplets | kernel ask K-c and K-e | B |
| **VP-A6** | Short-march cloud shell (R2b part 2, 5.6 H) | VP-A4, VP4 | H |
| **VP-A7** | Full volumetric clouds and light shafts; advected-particle giants | VP4 (ultra gate) | U |

### Build plumes, frosts and hot glow (`status/true-surface.md`, PR 449)

| Package | What | Needs |
|---|---|---|
| **VP-P1** | Hot ground and lava as real emitted light into the shared glare, crust from cell noise (extends TRUE3) | VP1, VP6 |
| **VP-P2** | Plumes with size-driven forward scatter and the umbrella shape (extends TRUE2) | K-c |
| **VP-P3** | Comets: coma switch-on distance, two tails, green head (extends TRUE5) | a kernel tail model (audit R32) |
| **VP-P4** | Ice cracks and frost sparkle below the floor through the ladder | VP6 |

### Build star glow and flares (`status/star-look.md`)

| Package | What | Needs |
|---|---|---|
| **VP-S1** | Hand SG1's glow to the shared glare so stars and worlds share one exposure | VP1 |
| **VP-S2** | Fast spinners: flattening and gravity darkening; lobe-filling shapes in close pairs | spin and lobe figures from `read_activity` or the resolve (ask K-d) |
| **VP-S3** | Corona when the disc is hidden, shaped by the cycle | |
| **VP-S4** | Spots drifting by latitude | |

### Build asteroids and rings up close (no status file yet)

| Package | What | Needs |
|---|---|---|
| **VP-R1** | Ring shadow on the planet and the planet's shadow on the rings (5.8) | VP1 |
| **VP-R2** | Dark side, opposition surge, forward scatter (5.9) | K-c |
| **VP-R3** | Gaps and wave trains at computed resonances; wavy edges beside embedded moons | moons' orbits (have) |
| **VP-R4** | Ring particles near the camera | VP4 |
| **VP-R5** | Small-body shapes from spin and strength; boulders through the ladder | VP6; kernel shapes (`shapes.rs`, planned by the plumes thread: agree the seam) |

### Build light and nebulae (`status/light.md`)

| Package | What |
|---|---|
| **VP-L1** | Nebula shapes and line strengths in a form the renderer can march (density recipe and line colours per kind) from `read_nebulae` |
| **VP-L2** | Aurora colours from the air's gases (MF9 part 2) |
| **VP-L3** | The colour of light inside a world's shadow, from `light_through_gas` along a grazing path (win 4) |
| **VP-L4** | Rainbow and halo angles from refractive index (data sheet look-up) |

### Asks of other threads

- **K-a (climate):** jet speed and eddy speed by latitude, and the Rhines scale worked from the eddy speed, in the `weather` op's giant output.
- **K-b (climate):** wind speed over each sea in the weather field (for the glint).
- **K-c (climate, plumes, rings):** a typical particle size for each cloud deck, haze, plume and ring region.
- **K-d (kernel or light):** a star's flattening and pole-to-equator temperature ratio from its spin, with the gravity darkening exponent as a block input.
- **K-e (light):** a block that turns a particle's size and refractive index into a phase table (Mie theory), run once per particle kind. The giants thread's TA2 already has a cheaper stand-in for albedo; this replaces the single-lobe phase everywhere.
- **Kernel thread:** section 11 option B (worker pool, then SIMD) as its own performance package when wanted.
- **Landing run-through:** owns the Land screen's glue; VP10 and VP11 draw inside it.
- **Exotic stars:** reads the sky cube of VP2 for bending.

---

## 13. How to check the look

1. **The per-body checklist** in the third audit (`SSE4-v3-gaps.md`, R3) stays the test. Rerun it after each VP package and record the date and commit in the owner's status file.
2. **Side by side.** For each body, a reference picture (NASA Photojournal, SVS or ESA) beside SSE4's True at the same phase angle, kept in project files, never in the repo and never shipped.
3. **The expert wins** (section 10) each get one screenshot and one test of the number behind it.
4. **Cost.** Each package reports its pass cost on the cloud runner, AC-ProArt and the reference phone from the performance readout, replacing this note's estimates.

---

## 14. Questions for Alex

**Alex accepted all ten recommendations on 24 Nov 2026, 18:57 UTC ("yes to all").** They stand as rulings; the questions are kept as asked.

1. **Three grades for everything drawn** (fact; drawn from the numbers; camera effect), with one HUD line saying where facts end, and the rule that a flourish with no kernel number behind it is not drawn. *Recommend yes.*
2. **Add an Ultra tier** for the passes that hammer the GPU, switched on only by a measured probe, with every Ultra pass having a plain form on weaker machines. *Recommend yes.*
3. **Place devices by measured speed, not by WebGPU or WebGL2**, so a fast WebGL2 phone can earn High. *Recommend yes.*
4. **Sharpen at rest automatically** in every True view (not only on "Take photograph"). *Recommend yes.*
5. **Start order:** the finish, the sky behind True and the texture ladder in the photoreal thread, with the giants' drift and curls (your example) in the giants thread at the same time. *Recommend yes.*
6. **Eye effects on, lens effects off.** Glare from real brightness is on in True; spikes, lens flare and grain are recipe passes for trailers and Orac, off by default. *Recommend yes.*
7. **Licence rule:** MIT, BSD and Apache code may be ported with notices; GPL, AGPL, NonCommercial and unmarked Shadertoy work are read for ideas and written fresh. *Recommend yes.*
8. **Reference pictures from NASA and ESA** kept in project files for side-by-side checks, never in the repo or the product. *Recommend yes.*
9. **Honest exposure by default:** beside a sunlit world the stars fade, as they really do, with a "Long exposure" choice in the view controls that shows both and says it is a composite. *Recommend yes; build the choice with VP2.*
10. **GPU physics:** use the GPU now only for drawing and previews; make the kernel's first speed-ups on the processor (a pool of workers, then SIMD), which keep every device's results identical; run one erosion experiment on the GPU later before deciding more. *Recommend yes.*

---

## 15. Sources

Checked on 24 Nov 2026 for this note:

- SpaceEngine blog: "Terrain engine upgrade #2", 2 Nov 2017 (`spaceengine.org/news/blog171102/`); "0.990.42.1830 and 0.991 public beta, ongoing work", 13 Nov 2020 (`/news/blog201113/`); "Animated scene in the main menu", 8 Oct 2018 (`/news/blog181008/`).
- Universe Sandbox blog: "Surface Grids & Lasers DevLog #8", 9 Aug 2019; "SPH Fluid Simulation DevLog", 13 Dec 2019; "Space in a New Light Preview", 26 Nov 2024; "Pale Blue DOTS Preview: Physics Simulation Overhaul", 18 Dec 2025.
- `@takram/three-clouds` 0.7.4 and `@takram/three-atmosphere` 0.13.2 READMEs (MIT; GLSL on WebGL only; Earth-only atmosphere; performance figures).
- Jendersie and d'Eon, "An approximate Mie scattering function for fog and cloud rendering", SIGGRAPH 2023 Talks, NVIDIA's project page (HG plus Draine blend; fitted over droplet diameter for water in air; HLSL with no stated licence).
- S. Cameron, "Procedural gas giant planet textures" (slides, 2014): curl noise after Bridson et al. 2007, offline. J. R., "Gas giant particle sim on a sphere" (2022): compute-shader particles in real time. Seed of Andromeda, "Procedural gas giant rendering with GPU noise": notes the Ashima simplex code is MIT.
- NAIF's Cosmographia credits and components pages. Gaia Sky manual, "Procedural generation".
- SSE4 repo at f95a334: `packages/renderer/src` (`gpu.ts`, `true-look.ts`, `gpu-globe.ts`, `gpu-surface.ts`, `recipe.ts`, `passes.ts`, `governor.ts`, `runtime.ts`, `rings.ts`), `docs/minimum-spec.md`, `packages/ui-contract/src/perf-chunks.json`, `crates/sse-weather`, the status files, and screenshots in project files.

The items first listed here as "from memory" were mostly checked on 24 Nov by the research pass: see section 17.3 for what is now checked and what is still from memory, and `SSE4-visual-polish-citation-check.md` for the citations.

---

## 16. Addendum, 24 Nov 2026 19:06 UTC: systems for every world, and aurorae

Alex, 19:06: "As we improve this renderer we are not just fixing it for the solar system. We are instead building SYSTEMS to ensure the right visuals get overlaid the right way, so they are concepts applied to all exoplanets and can be scaled accordingly and applied when the physics says so. Auroras are key too. Now we can have several different types and places they can occur (calculated) and we know their colours (in SSE3, multiband for O2)."

The first part is a ruling as given. The second adds a package group. Its questions are in 16.6 and Alex's answers in 16.7.

### 16.1 The rule: every look is a system that the physics switches on

This joins the four rules of section 3 as **rule 5**, and it binds every VP package and every builder:

1. **A look is switched on by a condition on kernel values, never by a name, a class or a list.** Not "Saturn has a hexagon" but "a polygon jet where the weather op's rule gives one". Not "Titan is orange" but "a haze whose particles have that absorption row". Renderer code contains no body names and no class tests. This is the standing ruling "processes, not classes" applied to drawing.
2. **A look scales with the numbers.** Each layer's declaration says which kernel values set its size, strength, colour, height and speed (the detail ladder's table in section 6 is the model). A world with ten times the wind has visibly different curls without anyone touching the shader.
3. **A look that the physics does not call for is absent, and the HUD or Explain can say why** ("no aurora: this world has no air to glow").
4. **Solar System bodies are the calibration, not the target.** They are where we can compare with a photograph. Every Solar System name in this note (sections 5, 9 and 10) is an example of a condition, and a builder should read it as "worlds like this".
5. **Each package is proven three ways** before it merges, with a screenshot of each in its package log:
   - **on a Solar System body**, against a reference picture;
   - **on at least one world that is not in the Solar System**, where the numbers differ enough to see the look scale;
   - **on a world where the look must stay off**, to show the condition works.

I reread sections 5, 9, 10 and 12 against this. No package is written as a per-body special case; where a row names a body it is an example. Two wordings to read with care: VP-P2's "umbrella shape" means any plume on a world too thin-aired to stop it, and win 7's Mimas figure is one check of a formula that runs for every moon and ring.

**The proving worlds (question 11).** So that "a world not in the Solar System" is the same handful for every thread, I propose ten generated systems kept as shared fixtures, each chosen because it pushes some look far from its Solar System value:

| # | Proving world | Pushes |
|---|---|---|
| 1 | Hot giant, locked to its star, 1,800 K day side | hot glow, day-to-night flow, no moons' shadows, sodium light |
| 2 | Rogue planet in a void with a helium sea (Alex's test world) | no star: its own heat and spin-driven aurora only; exposure at starlight |
| 3 | Locked "eyeball" world round an active red dwarf | red-star colours, flares, storm aurorae, one cloud shield at the sub-star point |
| 4 | Fast-spinning young giant near the brown dwarf line | flattening, narrow bands, bright spin-driven ovals |
| 5 | Heavy rocky world, thick carbon dioxide air, strong field | low scale height, squat aurorae, crushed clouds |
| 6 | Small cold world, thin nitrogen air, low gravity | tall air, detached haze, tall faint aurorae |
| 7 | Hazy moon of a giant, inside the giant's radiation belts | aurora and glow driven by the host's plasma, planet-shine, eclipses |
| 8 | Airless lava world close to its star | regolith law, night-side glow, no air effects at all (the "stays off" world for most air looks) |
| 9 | Ringed giant tipped on its side, with moons generated, not typed | ring shadows at odd angles, ovals far from the spin poles |
| 10 | Ocean world with no land and strong winds | glint width, wave fields, cloud streets |

The per-body checklist in the third audit gains a second half, "beyond Sol", with one line per proving world, rerun with the first.

### 16.2 Aurorae: what already exists

| Piece | State | Where |
|---|---|---|
| Design | Accepted 22 Nov: ovals follow the magnetic axis; power is a share of the wind's; "colours are not chosen", they come from the air's own emission lines; moon footprints; patches over crustal fields; diffuse glow on worlds with air and no field; storms move the oval on a dated event; in True, only what could be seen, by band | `SSE4-magnetic-fields-design.md` 3.4, 6.5, section 9 (MF9, MF14a) |
| Kernel, where they glow | **Built** (MF9 part 1): a wind-driven oval with its quiet-to-gusty range; a spin-driven oval when moons load plasma; a spot per moon inside the bubble; ovals placed in spin latitude for a tilted field; a diffuse flag for airy worlds with no field. Computed on every world; "nothing asks: is this a giant?" | `read_field`'s `aurora`; `handover/packages/MF9.md` |
| Kernel, how bright and what colour | **Not built** (MF9 part 2): power, colours from the line table, storm response, crustal patches | light thread, queued |
| Emission lines | Oxygen's green (557.7 nm) and red (630.0 nm) are in the light pack's line table already | `packs/base-light` |
| Drawing | Mag band only (`field.ts`). Nothing in Visible on the night side (audit R23) | renderer |
| This note before the addendum | One row (5.11), one line in VP12, VP-L2 for colours, win 22 | too thin for something Alex calls key |
| SSE3 | Read on Alex's PC (`physics/aurora.ts`): strength as air x field x incoming particles; a table of colours per gas with **two bands for oxygen** (green main, crimson high, the crimson only in oxygen-rich air), nitrogen purple low, carbon dioxide violet, hydrogen red-pink, methane blue; shells stacked by a height rank of 0, 1 or 2 | reference only |

SSE3's layering by height was its best idea and looked right. Its weakness is that the colours, the efficiencies and the height ranks were typed. SSE4 can compute all three.

### 16.3 Aurorae as a system

One mechanism: **a driver puts energetic particles into some air at some place; the air answers in its own lines, each at the height where that line can survive.** Every kind of aurora is a row in a table of drivers, not a special case.

**Drivers and places (each computed, each present only when its condition holds):**

| Kind | Switched on when | Where | State |
|---|---|---|---|
| Wind-driven oval | a field holds the wind off, and there is air | a ring round each magnetic pole at the footprint of the bubble's edge | built |
| Spin-driven oval | a fast-spinning field is loaded with plasma (from moons, or from the world's own air) | a ring at the footprint of where the plasma stops keeping up | built. This one needs no star, so it lights a rogue planet or a brown dwarf |
| Moon footprints | a moon orbits inside the bubble | a spot and a trailing tail where the moon's field line lands | spots built; tails to add |
| Diffuse glow | air but no field | the whole night side, faint, brighter toward the terminator | flag built |
| Crustal patches | old magnetised crust under thin air | arcs over the magnetised ground | waits on MF10 |
| Proton aurora | wind protons reach the air (weak or no field, or the dayside funnel of a strong one) | a broad soft band toward the equator from the oval, or the whole day side | to add |
| Dayside funnel spots | a strong field | one spot per hemisphere near local noon, poleward of the oval | to add |
| Storm aurora | one of the star's ejections arrives (O10's dated events) | the oval widens toward the equator and brightens for a day or two; red arcs well equatorward | to add (design 6.5) |
| Glow in a host's belts | a moon with any air sits in its planet's trapped plasma | the whole moon, or ovals if it has its own field | to add; SSE3 had it through total incoming flux |
| Spot on the star | a planet orbits inside the star's own magnetic reach | a bright spot on the **star** that follows the planet round | design 6.5; with the star thread |
| Weak-field wandering | the dynamo is dipping or reversing | ovals reach the tropics, several at once | falls out once MF3 gives the field over time |

**The air's answer (the part that makes colour and shape):**

- **Which colours:** every gas in the air above the height where the particles stop, weighted by amount x how readily that line is excited. Rows on the data sheet (emission lines per species with wavelength, upper-state lifetime and yield), read only through "Look up". Oxygen gives green and red; nitrogen blue-violet and a pink lower fringe; hydrogen red and blue Balmer light in Visible and far more in Ultraviolet and Infrared; carbon dioxide mostly Ultraviolet (with oxygen's green from its broken molecules); sodium yellow; and so on for whatever a pack adds. **[line data to check per row]**
- **How high each colour sits:** computed, not ranked. Two limits set it. The particles stop where they have crossed enough air, so more energetic ones glow lower. And a slow line is put out by collisions before it can shine: oxygen's red line takes about two minutes to emit, so it survives only where the air is very thin, high up; the green line takes under a second and survives much lower; nitrogen's light is immediate, so it shines wherever the particles stop and marks the bottom edge. Measured on Earth: green 0.7 s and 100 to 115 km, red about 110 s and 150 to 350 km, nitrogen about a hundred-millionth of a second and 100 to 120 km (checked, section 17). **So "green below, red above, purple fringe at the bottom" comes out of the air's density profile and each line's lifetime**, for any gas mix, with no typed heights.
- **How tall the curtains are:** the air's scale height. A small cold world with light air has curtains several times taller than Earth's; a heavy world's are squat.
- **How bright:** power from the driver (MF9 part 2), spread over the oval's area, per line, in the unit observers use. Compared with the exposure, that decides whether it is visible at all.
- **By band:** the same lines through the Band control. Giants' ovals are brilliant in Ultraviolet and Infrared and modest in Visible; X-ray spots at the strongest. Already the light design's rule.

**Drawing, by tier (grades in brackets):**

| Form | What | Tier |
|---|---|---|
| Glowing shells | Each colour as a thin shell at its own height over the kernel's ovals, spots and patches, emissive into the shared glare (5.1). From orbit: rings of light on the night side, and at the limb the colours visibly stacked (fact) | B, near free |
| Curtain texture | Rays and folds inside the oval, aligned with the field's direction there, drifting, through the detail ladder (drawn from the numbers: oval width, field direction, activity) | B one rung, H three |
| From the ground | The same shells seen from the site: an arc at the right height above the right horizon, overhead curtains inside the oval, nothing outside the kernel's range. The magnetic fields design's Site level already promises "tonight's aurora chance and where in the sky" (fact) | B |
| Ribbons | Curtains as ribbons standing along the field lines with a real height profile per colour | H |
| Volumes | Ray-marched curtains with depth, seen from inside | U |
| Pulsing and flicker | Under the "three flashes a second" cap; none under reduced motion (drawn from the numbers) | B |

**Cheap wins an expert can test** (they join section 10):

25. Ovals circle the **magnetic** pole, so on a world with a tilted, offset field they sit far from the spin poles and are different sizes north and south.
26. Green below red, with a purple lower fringe, from line lifetimes against air density. Change the air's density profile and the layers move.
27. No oxygen, no green. A hydrogen giant's visible aurora is pink low down turning purple higher up, with a strong red hydrogen line (seen at Saturn by Cassini, checked); most of its light is outside Visible. A carbon dioxide world still shows oxygen's green, faintly: measured from the ground on Mars in 2024.
28. Curtain height follows the scale height.
29. A rogue planet with a field, spin and a moon has aurorae with no star at all.
30. A moon's spot leads or trails the moon's position by the right angle, and has a tail.
31. A storm from the star moves the oval toward the equator on the date the ejection arrives.
32. Faint aurorae are grey to the eye and coloured in a long exposure (question 13).

**Not aurora, but the same machinery: airglow (question 14).** A faint glow of the whole upper air, from sunlight's chemistry, not from particles: a thin green line along the limb of an oxygen world all round the night side. It uses the same lines and the same height rule, with a different driver (absorbed ultraviolet). Proposed as one more driver row, built after the aurora kinds.

### 16.4 Packages added, and owners

**Facts: Build light and nebulae.** These are **MF9 part 2 as the magnetic fields design already assigns it**, not a second aurora track: the VP-AU numbers below only name its slices and add the heights and the missing kinds. They stay rows under MF9 in `status/light.md`, and they replace VP-L2:

| Package | What |
|---|---|
| **VP-AU1** | Emission lines move from the light pack's list onto the data sheet, each naming its element or substance id, with lifetime and yield added (16.5); read through Look up. Data sheet thread, with the light thread |
| **VP-AU2** | Power per driver and brightness per line; height per line from stopping depth and collision quenching; `read_field`'s `aurora` gains a list of kinds, each with place, lines, heights and brightness |
| **VP-AU3** | The missing kinds: proton aurora, dayside spots, moon tails, glow in a host's belts, storm events from O10 |
| **VP-AU4** | Airglow as a driver (if question 14 is yes) |

**Drawing: a new builder thread, "Build aurorae and night lights"** (question 12), so the photoreal thread is not the queue. It owns new files (`aurora.ts`, `gpu-aurora.ts`) and takes VP12 (city lights, lightning) with it, since all three are night-side light on a shell feeding the shared glare:

| Package | What | Needs | Tier |
|---|---|---|---|
| **VP-AU5** | Glowing shells from orbit and at the limb, colours stacked by the kernel's heights; first on the fixture fields, then generated worlds | VP1; VP-AU2 (until then, heights and colours from the two oxygen lines only, marked as a stand-in) | B |
| **VP-AU6** | Curtain texture through the detail ladder; pulsing under the flash cap | VP6 | B, H |
| **VP-AU7** | Aurora from the ground on the Land screen | VP-AU5 | B |
| **VP-AU8** | Ribbons along field lines; volumes | VP4 | H, U |
| **VP-AU9** | Other bands (Ultraviolet, Infrared, X-ray) through the Band control | VP-AU2 | B |

Proving worlds for aurora packages: an Earth-like fixture; proving worlds 2 (no star), 6 (tall curtains), 7 (host's plasma) and 9 (ovals far from the spin poles); and world 8 as the one where it stays off.

### 16.5 "Does our element editor have the aurora colours in it?" (Alex, 19:07)

**Not yet.** Checked on main at 9a29dc0:

| Piece | State |
|---|---|
| Where the lines are | A list of 27 named lines in the light pack (`packs/base-light`), each with a wavelength and a **free-text** species. It has oxygen's green (557.7 nm) and red (630.0 nm), nitrogen's violet (391.4 nm), hydrogen's Balmer lines and sodium's yellow. So oxygen already has two of its bands, as SSE3 did |
| What is missing from that list for aurorae | Oxygen's second red line (636.4 nm) and its near-infrared lines; nitrogen's blue (427.8 nm) and the red-pink band that makes the lower fringe; carbon dioxide's and carbon monoxide's ultraviolet bands; hydrogen's ultraviolet bands and the infrared glow of giants' upper air. And for every line, the two numbers the height rule needs: how long the line takes to emit, and how readily it is excited **[each row to be checked against its source]** |
| Lines tied to elements | **Designed, not built.** The matter design says "spectral lines name an element or substance id instead of free text" (its section on the optical group). Today nothing joins "O I" in the light pack to oxygen on the sheet |
| The Elements screen | **Built, read only** (MP10b): the periodic table coloured by where each element was made, with a card per element. It shows no lines and has no editing |
| Aurora colours from the lines | **Designed, not built** (magnetic fields 6.5; MF9 part 2) |

**What it should be** (and question 15):

- Each element's and each substance's card in Reference gains a section, **"Light it gives off"**: its lines as a strip across the spectrum in their real colours, each with its wavelength, how long it takes to emit and how readily it is excited, and its source. One line of plain words under it: "In air like this world's it glows green at about 100 km and red above 200 km", computed for the world you came from.
- On the Build desk a GM can **edit a line or add one**, as a pack overlay like any other sheet edit, marked authored. An invented gas given lines gets its own aurora, at heights the same rule computes, with no code. "Ask Orac for a gas that glows teal" then works end to end.
- The aurora card on a world lists which gases are glowing, in which colours, at which heights, and each links back to its element card. That is SSE3's "multiband for O2" made general: any gas, any number of bands, heights computed.
- Owners: the rows and ids are the data sheet thread's (VP-AU1); the card section is the chart room thread's (the screen side of MP10); editing follows the editing design's rules for sheet values.

### 16.6 Questions for Alex (numbering continues from section 14)

11. **Ten proving worlds** (16.1) kept as shared fixtures, and every VP package shown on a Solar System body, on a world that is not, and on a world where the look must stay off. *Recommend yes.*
12. **A new builder thread, "Build aurorae and night lights"**, for the drawing (aurorae, city lights, lightning), with the facts staying in the light thread. *Recommend yes.*
13. **Aurora to the eye and to the camera.** Really faint aurorae look grey-white to the eye, and green and red only in a long exposure. In True, the exposure adapts on the night side so bright aurorae show their colour, faint ones are pale, and "Long exposure" shows full colour and says so. *Recommend yes: it is true, testable, and the long exposure still gives the spectacular picture.* The alternative is always full colour, which is prettier and not what an eye sees.
14. **Airglow** as one more driver in the same machinery, after the aurora kinds. *Recommend yes.*
15. **"Light it gives off" on every element and substance card**, with the lines editable on the Build desk so an invented gas gets its own aurora. *Recommend yes.*

### 16.7 Alex's answers, 24 Nov 2026 19:12 UTC

"We can have 'long exposure' on by default :) Yes to all."

- **Questions 11, 12, 14 and 15 stand as recommended.** Ten proving worlds; a new builder thread "Build aurorae and night lights" (started by the coordinator at 19:12); airglow as one more driver; "Light it gives off" on element and substance cards, editable on the Build desk.
- **Questions 9 and 13 are reversed in their default.** Ruling: **"Long exposure" is the default, and the eye-true view is the choice.**

How to read that ruling (my reading, chosen to be one mechanism with no special cases; correct it if wrong):

1. **One Exposure control with two states, "Long exposure" (default) and "Eye", in the shared view controls of every True view**, at every scale, remembered with the saved view choice like Band and Scale. It is not switched on only when something faint is in frame: a default that changes by itself would make the same world look different for no visible reason, and the two states look the same anyway when nothing faint is there.
2. **Long exposure** is what a camera gives when bright and faint things are exposed separately and combined: sunlit worlds at their daylight exposure, and the faint layers (stars and the galaxy's band, aurorae, airglow, nebulae, planet-shine, night-side lights, faint rings and plumes) lifted to where a long exposure would show them, in full colour. The HUD's exposure line says "Long exposure: faint light is lifted" whenever it is on, so the picture never claims to be what an eye sees.
3. **The lift is a number, not a look.** Each faint layer is drawn at its real brightness times one stated exposure ratio for the view, so ratios between faint things stay true (a brighter aurora is still brighter, a faint one still faint), and the ratio is shown in the HUD.
4. **Eye** is the honest single exposure with the eye's own limits: beside a sunlit world the stars fade, and aurorae too faint for colour vision go grey-white. Wins 21 and 32 in the expert list are seen by switching to Eye, and each says so in its Explain line.
5. **Glare stays physical in both states** (section 5.1): only things that are truly far brighter than their surroundings bloom. Long exposure does not make aurorae or stars bloom.
6. Ground view: the same control. Long exposure by day shows a daylight sky; it lifts faint light only where the sky is dark enough for a camera to record it, so it does not paint stars onto a bright blue sky.

Builders: VP1 (finish) carries the Exposure control and the lift ratio; VP2 (sky) and VP-AU5 (aurora shells) draw their layers in real brightness and let the finish apply the lift.

---

## 17. Citation check folded in, 24 Nov 2026 19:27 UTC

Alex ran a research pass through Consensus on the eighteen items I listed. The full report, with every citation, DOI and evidence grade, is **`SSE4-visual-polish-citation-check.md`** (project files and `handover/design/`). It was written without sight of this note, so it tested each claim as worded in my request. This section says what changed here. Builders: take formulas and numbers from the check file, and copy its citations into `docs/citations.md` with the status it gives.

### 17.1 What changed a design choice

1. **How particles scatter light (5.5, VP-A5, new ask K-e).** The cheap fit I proposed (Jendersie and d'Eon) is fitted only for water droplets of 5 to 50 µm in air, and nobody has published a version for methane, sulphuric acid, haze or dust. A separate 2024 paper shows the usual single-lobe formula is not good enough for Mars's dust. So the general system is: **the kernel computes a phase table for each kind of particle from its size and the data sheet's refractive index** (Mie theory, once per kind), and every particle layer reads its table. The fit stays as the shortcut for water clouds only. This suits the "systems, not Solar System fixes" rule better than what I first wrote, and it is what makes a dusty world's blue sunset (win 13) come out right.
2. **Curl size on giants (5.4, win 11, ask K-a).** The Rhines scale matches Jupiter's and Saturn's real bands only when it is worked from the local eddy speed (10 to 40 m/s), not the 100 m/s jet speed, and then within a factor of two. The form π√(2U/β) is the width of one jet, not the spacing of eastward jets. The climate thread should publish the eddy speed beside the jet speed.
3. **Gravity darkening on fast stars (5.12, win 14, ask K-d).** The 1/4 power holds only for slow spinners. Measured fast spinners give about 0.19, and stars with convective envelopes about 0.08. The exponent becomes an input of the block, chosen from the star's envelope and spin. Vega's best figures are 10,070 K and 8,910 K (2012), not the older, larger contrast.
4. **Moving physics to the GPU (11.2).** The specifications confirm the caution, and more strongly than I wrote: avoiding fused operations and built-in functions is necessary but not enough, because divide and square root are not exact either and the rounding mode cannot be pinned in WebGPU. Integer arithmetic is the only route that is exact by specification on every device. No peer-reviewed study was found showing bit-identical floating-point compute results across GPU makers. The recommendation in 11.4 stands unchanged.

### 17.2 Numbers corrected or tightened

| Where | Was | Now |
|---|---|---|
| Win 3, sea glint | formula only | The same formula, confirmed, with its conditions: wind measured 12.5 m up, good for 1 to 14 m/s. An oil-like slick halves or thirds the roughness |
| Win 2, opposition surge | no figure | Moon: more than 40% brighter between 4 degrees and 0. Saturn's rings: about a third, inside half a degree |
| Win 4, eclipsed moon | "copper" | 10,000 to a few 100,000 times fainter than full. New touch: an ozone layer gives the shadow's edge a blue-turquoise rim. On a thin dusty world, bending is negligible and dust scattering sets the light. A 2025 paper on drawing eclipses in real time is the reference for 5.8 |
| Win 6, rings' dark side | "dense dark, thin gaps glow" | True for the densest ring. Which ring is brightest depends on the sun's and viewer's angles; the formula's peak is in the check file |
| Win 7, ring edge at a resonance | 117,600 km | Mean 117,570 km, and the real edge moves by 4 to 71 km over about 5.4 years |
| Win 23, wavy gap edges | 3π x half-width | Confirmed: 3π x the distance from the moon's orbit to the edge (about 198 km for the small moon in Saturn's 42 km gap) |
| Win 15, comet green | "broken by sunlight" | C2 lasts about two days at 1 au. Switch-on distances by ice: water 3 au, carbon dioxide 12, carbon monoxide 50 |
| Win 17, top-shaped asteroids | "near the rate" | Critical period about 3.3 h divided by the square root of density in g/cm3; Ryugu's shape formed at 3.5 h, Bennu's surface fails at 3.6 h |
| Win 19, methane rainbow | 49 degrees, from memory | 49 degrees confirmed for a refractive index of 1.29, ranging 47 to 52 with temperature and colour |
| Aurora heights (16.3) | from memory | Green 0.7 s, 100 to 115 km; red about 110 s, 150 to 350 km; nitrogen prompt, 100 to 120 km |
| Win 27, hydrogen aurora | "pink-red" | Pink low, purple at 1,000 to 1,500 km, strong red hydrogen line: that is Saturn (Cassini). No colour has been reported for Jupiter's visible aurora |
| Aurora under carbon dioxide | ultraviolet, plus green | Visible green confirmed on Mars from the ground, 18 March 2024, about 100 rayleighs: faint, so a "Long exposure" subject. Venus's green line was reported as night glow; calling it aurora is a later reading |
| Lightning on giants (5.4) | no figure | A test value for the kernel's rate: Jupiter about 0.06 flashes per km2 per year, some 47 times fewer per area than Earth's 2.8, though storms peak at up to four a second |
| Plumes and faint rings (wins 8) | no figure | Grain sizes confirmed (plume about 3 µm, faint ring about 1 µm). No measured brightness-against-angle factor was found, so none is quoted; the phase table computes it |
| Atmosphere look-ups (5.5) | "small" | Hillaire's own sizes: 256 x 64, 32 x 32, 200 x 100, and a 32-cubed volume. His method drifts in hue at very high scattering and loses accuracy for strongly forward-scattering particles; say so in the fudge register |
| Dust on a Mars-like world | not given | Grains 1 to 2 µm across the seasons |

### 17.3 Now checked, and still open

**Checked** (move to "checked" in `docs/citations.md` when used): Cox and Munk 1954; Hillaire 2020; Jendersie and d'Eon 2023; Hapke's papers and the opposition surge figures; von Zeipel 1924, Lucy 1967 and the measured stars; Rhines 1975; ring resonances and edge waves; transmitted light through rings; the eclipsed Moon; Mars's blue sunset mechanism; the methane rainbow; comet colour and switch-on; top-shaped asteroids; aurora lines and heights; lightning rates; Bridson, Hourihan and Nordenstam 2007; Schneider's Nubis talks (2015, 2017, 2022, 2023: course talks, not peer-reviewed); the shader specifications' floating-point rules.

**Still from memory** (not in the research pass, or not settled by it): the licences of Celestia, Stellarium, OpenSpace's shaders, AMD's FSR 1, Bruneton's code, `psrdnoise`, the hex-tiling sample, NVIDIA's blue noise and Sebastian Lague's repositories; Shadertoy's default licence; the simplex patent's expiry; Perlin and Neyret 2001; Vlachos 2010; Heitz and Neyret 2018; Tessendorf 2001; and the sub-claims the check file lists under "Still unverified" (Hapke's formula in a primary text, the original wind-profile papers, any brightness factor for plumes, and others).

The check file also names supporting notes under a `sse4-citation-check` folder in the session that made it. Those were not passed on; the check file itself is complete enough to build from.
