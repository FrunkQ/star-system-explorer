# SSE4: planetary surfaces, landings and browser photorealism

Research note, 18 Nov 2026. Feeds the design handoff (H8 views, H12 evidence, H14 device edges) and the build plan. Written for the design thread and for the coding session that follows. Claims checked against sources are cited at the end; estimates are marked **[estimate]** and must be measured, per H12.

---

## 1. Short answers

**Can the NASA/ESA open-source tools do this as well as or better than Space Engine?** Not as tools you can embed. The two Alex named are desktop applications: OpenSpace is C++ on OpenGL 4.6 (MIT licence, Windows and Linux only, no Apple Silicon), and Gaia Sky is Java on OpenGL (MPL-2.0). Neither runs in a browser, and neither can become a component of SSE4. NASA's browser tool, Eyes on the Solar System, is not published as open source as far as I could find. NASA/ESA WebWorldWind (Apache-2.0) and CesiumJS (Apache-2.0, which added accurate Mars and Moon ellipsoids in 2025) do run in browsers, but they are geospatial globes built to stream measured map tiles, not to generate worlds, and they have no photoreal atmosphere or landing pipeline of the kind needed.

What *is* usable is the layer underneath: the published techniques, MIT-licensed browser implementations of them, and NASA's public-domain elevation data for real bodies. With those, SSE4 can build its own renderer.

**Can SSE4 match Space Engine?** Partly, and it can beat it where it matters to SSE4.
- **Stills:** yes, close to parity on any device, by rendering progressively while the camera is still (section 6.4).
- **Real-time on a desktop or recent laptop with WebGPU:** close. Atmosphere, terrain, lighting and clouds at good quality are well within reach in 2026 browsers.
- **Real-time on phones:** no. A phone gets a good globe with an atmosphere and a convincing descent, but not Space Engine's volumetric clouds and dense ground detail at 60 fps. Phones should get the stylised views at full quality instead, and those are often the better story tool anyway.
- **Where SSE4 beats it:** every surface comes from the world's own dated geological history, so a mountain range has a reason, an age and an explanation, and the same height data drives the map, the hazard estimate, the astrogation console, the text narration and the photoreal render. Space Engine's surfaces, as far as is publicly known, come from procedural noise presets per planet class rather than from a simulated history (unverified: it is closed source).

**Can we have a height map on surveyed worlds that all rendering uses?** Yes, and it should be a first-class kernel product (the "surface atlas", section 3). It is the single source for every view, from text lines to photoreal.

**Can landings be shown in many ways, evolvable by Orac?** Yes. The kernel produces one "descent scene" per landing; each style is a render recipe (data, not code) built from a finite library of passes with a large parameter surface meant for Orac to tune, with degrade rules that tie it to the device's tier (section 5).

---

## 2. What browsers can do in late 2026

- **WebGPU** ships by default in Chrome and Edge (Windows, macOS, ChromeOS since 113; Android 12+ on Qualcomm and ARM GPUs since 121; Linux partly), Safari 26 (macOS, iOS, iPadOS, visionOS) and Firefox (Windows since 141, Apple Silicon macOS since 145 to 147). Firefox on Linux and Android still needs a flag. So almost every iPhone and recent Android phone, and nearly every desktop, has it; a WebGL2 fallback is still required for the rest.
- **three.js `WebGPURenderer`** is production-ready, falls back to WebGL2 automatically, and its shading language TSL compiles to both WGSL and GLSL, so one shader source serves both tiers. Compute shaders are available on WebGPU. Caveat from the field: WebGPU is not automatically faster; profile.
- **Proven browser pieces:** Hillaire's sky and atmosphere technique (the one Unreal uses) has an MIT WebGPU implementation with runtime-editable Rayleigh, Mie and absorption parameters for any rocky planet; Bruneton's precomputed scattering is in `@takram/three-atmosphere` (MIT) but is effectively Earth-only, and its own docs point to Hillaire for other planets. Full-scale planets with precomputed atmospheres, raymarched clouds and meshlet terrain have been demonstrated in WebGPU by independent developers. Cosmos Journeyer (Babylon.js, AGPL-3.0, so a reference only, not a source) shows a whole procedural universe with walkable planet surfaces in a web page.
- **Hard limits to design around:** GPU memory on phones (budget for a few hundred megabytes), sustained performance under thermal throttling (phones drop sharply after a minute or two of full load), 32-bit floats on the GPU (planet-scale needs camera-relative rendering and a reversed or logarithmic depth buffer), download size (assets must be generated, not shipped), and no reliable thermal or battery API across browsers (frame time is the only portable signal).

---

## 3. The surface atlas: one height map, generated from history

### 3.1 What it is

A deterministic kernel product, keyed by address, that describes a body's solid surface (and its liquid and ice cover) at a stated epoch. It is resolved when the body reaches the `surveyed` disclosure tier (H6.7), or on demand by the GM. It is never stored as a world; it is regenerated from the seed and cached (OPFS or IndexedDB) like any other derived result.

```
/body:<path>/shell:surface@<epoch>/tile:<face>/<level>/<x>/<y>
```

- **Geometry:** a cube-sphere quadtree (six faces, each a quadtree). Even sampling, no pole pinch, and it maps straight onto GPU terrain level-of-detail.
- **Level 0 (global survey):** six faces of 512 x 512 samples. For an Earth-sized world that is about 20 km per sample, enough for the globe, the chart, the shaded relief map and the plate view. About 3 MB at 16-bit **[estimate]**.
- **Refinement tiles:** generated on demand down to a **fact floor** (section 3.4), keyed on their own address so any tile can be built without its neighbours. A child tile must downsample to its parent within a stated tolerance, which keeps every zoom level consistent.
- **Layers per sample:**

| Layer | Use |
|---|---|
| elevation (metres from datum) | everything |
| surface age (time since last resurfacing) | crater density, weathering, colour; "ancient versus recent" |
| material class (basalt, granite, regolith, ice, salt, sediment, alien classes from packs) | colour, roughness, hazards |
| flow accumulation and river/channel mask | rivers, canyons, deltas, and Titan-style methane channels |
| ocean, lake, ice cover | coasts, glaciers, seas |
| feature tags (crater with its age, volcano, rift, scarp, dune field, chaos terrain…) | labels, Explain mode, story finder |
| slope and roughness | landing hazard, routing difficulty |
| biome (from the life suite) and structures (authored or settlement layer) | colour, aerostats, cities, bases |

Every feature tag carries a chronicle reference, so Explain mode can answer "why is this ridge here?" with the dated event that made it.

### 3.2 How it is generated: from the chronicle to relief

The body already has a dated history in the chronicle (impacts, volcanic epochs, ocean loss, stellar brightening, tidal heating). The surface pipeline turns that history into relief in four stages, each a box of atoms in the physics suite, editable in the node editor like any other flow.

1. **Regime.** From mass, composition, internal heat, water and tidal heating over time, pick the tectonic regime per era: mobile-lid plate tectonics, stagnant lid, episodic overturn (Venus-like), heat-pipe (Io-like), ice-shell (Europa-like), or contraction (Mercury-like). Regimes can change over the body's life, and that change is an event.
2. **Large-scale structure.** For plate worlds, a procedural plate model in the style of Cortial et al. 2019 ("Procedural Tectonic Planets"): plates move, subduct and collide over hundreds of millions of years to produce continents, ridges, ranges and island arcs, without a physics simulation. Its output is an **uplift history**, not finished terrain. Other regimes produce uplift by their own rules (a Tharsis-like bulge on a one-plate world, global resurfacing on a Venus-like one, a cracked ice shell on an ocean moon).
3. **Erosion and deposition.** Uplift balanced against fluvial erosion with the stream power law, after Schott et al. 2023 ("Large-scale terrain authoring through interactive erosion simulation", MIT code). This gives dendritic drainage and believable ridges from an uplift map. Erosion strength comes from the climate history (rainfall, temperature, ice ages), so a world that lost its oceans 3 billion years ago keeps old, softened river valleys. Aeolian (wind) and glacial terms are added for dry and cold worlds.
4. **Events and amplification.** Impacts placed from the chronicle's impact history, with size-frequency distributions and degradation by age (real planetary science: crater density is how surface ages are measured, which is a good education hook). Volcanoes from volcanic epochs. Then a deterministic fractal amplification down to the fact floor, shaped by material class and age.

**Alien mechanisms** ship as pack atoms that plug into stages 1 to 4, each with its research citation, for example: heat-pipe volcanism (Io), episodic global resurfacing (Venus), ice-shell chaos terrain and cryovolcanism (Europa, Enceladus), methane rain, rivers and dunes (Titan), convecting nitrogen ice plains (Pluto's Sputnik Planitia), global contraction scarps (Mercury), sublimation terrain (Mars's south polar "Swiss cheese"). A physics pack can add more, including fictional ones, tagged as such.

### 3.3 Real bodies and the real universe **[decided 18 Nov: real data fetched dynamically wherever it exists]**

Whenever a campaign sits in the real Milky Way, real measurements are pulled in on demand and override the procedural guess, the way catalogue stars do in H10. Procedural generation only fills what is not known, and the fill is tagged as such.

- **What is fetched:** stars (Gaia DR3 and bright-star lists), known exoplanets with measured orbits, masses and radii (NASA Exoplanet Archive), and Solar System surfaces: elevation (Mars MOLA 463 m, Moon LOLA 118 m and others from USGS and PDS, generally public domain as US government works) and imagery where licences allow. Confirm each source's terms before use.
- **How it fits the model:** every measured value is a **pin** that claims from the cloud's budget first, so a real star or planet is never duplicated by the fill. A known exoplanet with only mass and orbit measured keeps those values and gets its unknowns (surface, atmosphere, history) resolved procedurally around them, consistent with the measured facts.
- **How it is fetched:** through SSE4's own Cloudflare Worker and R2 cache, not straight from the agencies. That gives CORS, rate limiting, tiling into the address scheme and one stable endpoint. Devices cache what they have seen.
- **Determinism:** the run manifest (H5.6) pins the version of each catalogue used, so a campaign resolves the same way for every peer and in the future. Upgrading to a new catalogue release is an explicit GM action that shows what changed.
- **Offline:** without a connection, already-cached real data is used and the rest falls back to procedural, with the edge shown (H14).
- Below a measurement's resolution, deterministic amplification adds detail, tagged as fill.

### 3.4 Determinism, cost and the fact floor

- **Facts above the floor, decoration below it.** The kernel (Rust, pinned maths, fixed iteration counts) produces every tile down to the fact floor, bit-identical on every device. Anything finer (pebbles, grass, sub-floor roughness) is generated on the GPU and may differ slightly between GPUs. That is acceptable because nothing in the game depends on it: landing hazards, routes and Explain answers read kernel tiles only.
- **Recommended fact floor:** about 100 m globally on demand, about 10 m inside a declared landing zone or base site **[estimate; tune by measurement]**.
- **Cost classes (H14):** the tectonic and erosion history is a heavy block (global, billion-year run, likely seconds to tens of seconds per world in WebAssembly on a laptop **[estimate]**). It follows the agreed rule: capable devices compute it, weak devices fetch the finished tiles from the GM's machine over the existing peer-to-peer link (tiles are data: a 256 x 256 16-bit tile is about 128 KB before compression), and the UI shows which happened. Refinement tiles near the fact floor are light and run anywhere.
- **Ageing:** because the atlas is keyed by epoch, ageing the Solar System (Alex's top user wish) re-resolves surfaces at the new epoch. Rerunning the history for a later epoch reuses everything up to the old one.

---

## 4. The descent scene: one input for every landing style

The kernel's NAV engine produces a **descent scene** for any landing, probe drop or flyover. It is plain data, built after the player data-view filter like every scene (H8.1):

- trajectory samples: time, position, velocity, attitude, thrust, heating
- the surface atlas tiles along the ground track and around the landing zone
- the atmosphere profile and weather along the path (density, wind, storms)
- hazard regions from the slope, roughness and weather layers
- telemetry events with times, written in the ship-computer register ("Crosswind 27 kph WNW", "Storm approaching", "Autopilot locked onto LZ beacon")
- the landing zone, any base or structures, and the body's sky (H10)

Alex's reference video (`C:\Development\Misc\m2-res_1080p.mp4`, 1 min 55 s) is a good test that this is enough: everything in it can be drawn from that scene. Its stacked horizontal terrain lines are cross-sections of the height map along the track; its red outlines are hazard contours; its receding rectangles are the descent corridor; its right-hand column is the telemetry events; its countdown is trajectory time. No extra data is needed.

---

## 5. Landing styles as render recipes (evolvable by Orac)

Alex's three examples (text, simple vector, photoreal) are points on a range, not the list. The range should be open-ended and tunable mainly by Orac.

### 5.1 The model

- **Pass library (code, finite, shipped and tested).** Small render passes, each like a kernel atom: `profile-lines`, `contours`, `corridor`, `hud-telemetry`, `shaded-relief`, `terrain-mesh`, `atmosphere-hillaire`, `clouds-raymarch`, `water`, `scatter-detail`, `film-grain`, `crt-phosphor`, `bloom`, `tonemap`, `text-narration`, and so on. Each declares its inputs (scene fields), its tiers, a cost model, and a parameter schema with ranges and defaults.
- **Render recipe (data, unlimited).** A recipe is a JSON graph of passes with parameter values: "green-screen descent", "chart-room survey drop", "photoreal landing", "1970s film footage", "Hitchhiker's Guide narration". Recipes are view configuration, not kernel state, so editing one never touches determinism.
- **Hanging config.** Each pass exposes many parameters (line spacing, phosphor decay, colour ramps, camera lag, scattering coefficients, cloud step counts, lens dirt…). A recipe also defines a handful of **human knobs** ("grit", "retro", "drama") that map to many of those deep parameters. People use the knobs; Orac edits the deep parameters directly.
- **Orac's loop.** Orac edits a recipe, the app validates it against the schemas, estimates its cost against the device's budget, renders a preview frame, and gives that image back to Orac to inspect. Orac can therefore iterate visually ("make the coastlines read more clearly at night"), not blind.
- **Sharing.** Recipes are shareable on the explorers site like flows. **Recommendation:** shared recipes are data-only by default. A recipe that carries custom shader code is marked "contains code", runs only after the user allows it, and is never sent to a player view by default, because shader code from strangers can hang a GPU and is code, not data.

### 5.2 How recipes meet hardware

- Every recipe declares which tiers it supports and a **degrade order**: the sequence of parameter reductions and pass removals to apply when the frame budget is missed (for example: lower render scale to 0.75, halve cloud steps, drop scatter detail, swap raymarched clouds for a cloud layer texture, drop to the `gpu-basic` variant).
- The runtime governor (section 6.3) walks that order. Orac is told the cost of each change as it tunes, so it can make a recipe that holds up on a phone.
- If a recipe cannot run on a device at all, the view falls back to the nearest recipe of the same family that can, and the edge is shown, as agreed in H14.

---

## 6. Scaling to the hardware

### 6.1 Tiers (extends the H8.1 tiers)

| Tier | Typical device | Surface and landing rendering |
|---|---|---|
| `text` | anything, including screen readers | narration from the descent scene and atlas; terrain described by profile ("ridge 2.4 km ahead, crossing left to right") |
| `canvas` | very old devices, print | vector and contour styles (the reference video's look runs here at full quality), shaded relief maps |
| `gpu-basic` | phones, integrated graphics, WebGL2 fallback | cube-sphere globe with level of detail, baked colour and normal maps from the atlas, cheap atmosphere from lookup tables, 2D cloud layer, render scale 0.5 to 0.75, target 30 fps |
| `gpu-high` | WebGPU desktops, recent laptops and tablets | GPU-amplified terrain below the fact floor, Hillaire atmosphere with aerial perspective, raymarched clouds at low resolution with temporal upscaling, shadows, HDR, target 60 fps at 1080p to 1440p on a mid-range GPU **[estimate]** |
| `still` | any GPU device, while the camera is at rest | progressive accumulation: more samples, full-resolution clouds, soft shadows and higher terrain detail build up over a few seconds into a near-photoreal image |

### 6.2 Probe at start

On first run (alongside the kernel's work-unit benchmark from the build plan): WebGPU adapter and its limits and features (e.g. `shader-f16`, timestamp queries), a short GPU benchmark of representative passes, available memory hints, screen size and pixel ratio, and `prefers-reduced-motion`. This sets the starting tier and a per-pass cost scale. The user can override it.

### 6.3 Governor at run time

Frame time is the only portable signal for load and thermal throttling. The governor keeps a frame-time budget per view (lower on battery or when several seats share a device), walks the recipe's degrade order when the budget is missed for a sustained period, and steps back up slowly with hysteresis so quality does not flicker. A small quality badge on the view says what was reduced and why ("Clouds simplified: this phone's GPU is at its limit"), and tapping it shows what would lift it. This is the H14 "edges are visible" rule applied to rendering.

### 6.4 Stills are the great leveller

For an RPG, the most valuable photoreal output is often a picture: "the probe's last image", "the view from the landing pad", a handout. Progressive accumulation lets a phone produce a near-desktop-quality still in a few seconds and a desktop produce one beyond its real-time quality. **Recommendation:** make `still` a first-class tier and give every Body, Sky and descent view a "take photograph" action that renders progressively and saves to Media.

### 6.5 Rendering on the GM's machine and streaming video (considered, not recommended as default)

A strong GM PC could render photoreal frames and stream them as video to weak player devices over the same peer-to-peer link. It breaks the agreed "GM streams only data" rule, costs the GM's GPU and bandwidth, and fails when the GM is offline. **Recommendation:** not in the first release. Revisit later as an opt-in for a shared big screen in the room.

---

## 7. Software choices and licences (SSE4 is Apache-2.0)

| Candidate | Licence | Runs in browser | Verdict |
|---|---|---|---|
| OpenSpace (NASA/AMNH) | MIT | No (C++, OpenGL 4.6, Windows/Linux) | Reference only; its globe-browsing papers are worth reading |
| Gaia Sky (ESA/ARI) | MPL-2.0 (own datasets CC-BY) | No (Java, OpenGL) | Reference only; its GPU procedural planet notes are useful |
| NASA Eyes on the Solar System | Not published as open source (as far as found) | Yes | Look-and-feel reference only |
| NASA/ESA WebWorldWind | Apache-2.0 | Yes (WebGL) | Earth geospatial globe; not suited |
| CesiumJS | Apache-2.0 | Yes (WebGL) | Now supports Mars and Moon ellipsoids; built for streamed measured tiles, heavy, no generator; not recommended as the base, possible for a future "real Mars" view |
| three.js + `WebGPURenderer` + TSL | MIT | Yes (WebGPU, WebGL2 fallback) | **Recommended renderer base** |
| Babylon.js | Apache-2.0 | Yes (WebGPU, WebGL2) | Credible alternative; heavier |
| Hillaire atmosphere, `webgpu-sky-atmosphere` | MIT | Yes (WebGPU) | **Recommended atmosphere**; port to TSL for the WebGL2 path |
| `@takram/three-atmosphere` (Bruneton) | MIT | Yes | Earth-only; not recommended |
| Stream power erosion (Schott et al.) | MIT | Desktop C++ | **Port the algorithm into the Rust kernel** |
| Procedural Tectonic Planets (Cortial et al.) | Paper | n/a | **Implement from the paper** in the kernel |
| Cosmos Journeyer | AGPL-3.0 | Yes (Babylon.js) | Reference only; do not copy code (licence incompatible) |
| Space Engine | Proprietary | No | Benchmark for quality only |
| MOLA, LOLA and other NASA DEMs | US government works, generally public domain | Data | **Use for real bodies**, confirm each |

MIT and Apache-2.0 code can be included in an Apache-2.0 project with notices kept. MPL-2.0 files may be used unmodified as separate files but are irrelevant here since Gaia Sky is Java. AGPL code must not be copied in.

---

## 8. Open questions, each with a recommendation

**Alex accepted all eight recommendations on 18 Nov**, with question 5 widened as shown.

1. **Renderer base.** Three.js `WebGPURenderer` with TSL, own planet layer on top (cube-sphere level of detail, camera-relative rendering, reversed depth). Babylon.js is the fallback choice. Not CesiumJS, not raw WebGPU from Rust (one more Rust surface, slower to iterate, worse for community view plug-ins).
2. **Fact floor.** About 100 m globally on demand and about 10 m in landing zones and base sites, configurable per pack; everything finer is decoration and never affects play.
3. **When is the atlas resolved?** At `surveyed` disclosure or on GM demand, globally at level 0, with refinement only where someone looks or lands. Players before `surveyed` see the `scanned` view (albedo, rough relief) generated from level 0 with detail withheld.
4. **History depth for the surface.** Run the full tectonic and erosion history for worlds that are surveyed; for unsurveyed worlds use a cheap regime-based sketch so the globe still looks right from orbit.
5. **Real-universe data.** **[decided 18 Nov]** Fetch real catalogues and measurements dynamically wherever they exist (stars, exoplanets, Solar System elevation and imagery) as pins, with procedural fill only for the unknown; versions pinned in the run manifest (section 3.3).
6. **Photoreal target.** Aim for parity with Space Engine in stills on every GPU device and close in real time on WebGPU desktops; give phones the full stylised range and good `gpu-basic` globes rather than chase real-time photoreal there.
7. **Shared recipes with code.** Data-only by default; code-bearing recipes opt-in per device and never pushed to players.
8. **GM-rendered video for players.** Not in the first release.

---

## 9. Suggested additions to the build plan

| Id | Package | Needs | Acceptance checks |
|---|---|---|---|
| S1 | Surface atlas: cube-sphere addressing, tile layers, parent-child consistency, caching, disclosure | K2, K9 | any tile builds alone with identical hashes on all harness browsers; children downsample to parents within tolerance |
| S2 | Tectonic regimes and procedural plate model producing uplift histories | P1, S1 | Earth-like fixture forms continents, ridges, arcs; stagnant-lid fixture forms none; replays bit-identically |
| S3 | Stream power erosion with climate-driven strength, aeolian and glacial terms | S2 | drainage networks dendritic on fixtures; old dried world keeps softened valleys |
| S4 | Impacts and volcanism from the chronicle, crater degradation by age, amplification to the fact floor, alien-mechanism atom slots | S3 | crater density versus surface age matches the fixture's chronicle; Explain answers "why is this here" |
| S5 | Hazard layer and descent scene for NAV | S4, NAV | landing difficulty from slope, roughness and weather; descent scene fixture reproduces the reference video's content |
| R1 | Pass library, render recipes, degrade order, governor, quality badge | V1 | a recipe runs on all tiers it declares; governor holds budget on a named low-end phone |
| R2 | Body and descent views on `gpu-basic` and `gpu-high`, Hillaire atmosphere, clouds | R1, S1 | visual fixtures for an Earth-like, a Mars-like and a Titan-like world |
| R3 | `still` tier and "take photograph" to Media | R2 | stills on a phone in under 10 s **[target]** |
| R4 | Orac recipe editing with preview frames | R1, A-track | Orac adjusts a recipe from a written request and the cost stays within the device budget |

---

## Sources

- [WebGPU is now supported in major browsers (web.dev, Nov 2025)](https://web.dev/blog/webgpu-supported-major-browsers)
- [What's new in Three.js 2026 (Utsubo)](https://www.utsubo.com/blog/threejs-2026-what-changed) and [three.js WebGPURenderer manual](https://threejs.org/manual/en/webgpurenderer)
- [OpenSpace repository](https://github.com/OpenSpace/OpenSpace) and [OpenSpace project](https://openspaceproject.com)
- [Gaia Sky licences](https://gaiasky.space/licenses/) and [Procedural generation of planetary surfaces (Gaia Sky author)](https://tonisagrista.com/blog/2021/procedural-planetary-surfaces)
- [NASA WorldWind](https://worldwind.arc.nasa.gov) and [WebWorldWind (worldwind.earth)](https://worldwind.earth/webworldwind.html)
- [Eyes on the Solar System in the browser (PC Gamer)](https://pcgamer.com/nasas-3d-space-exploration-tool-eyes-on-the-solar-system-now-works-in-browser)
- [Lift off to Cesium Mars (Cesium, Sep 2025)](https://cesium.com/blog/2025/09/03/lift-off-to-cesium-mars/)
- [A Scalable and Production Ready Sky and Atmosphere Rendering Technique (Hillaire 2020)](https://diglib.eg.org/handle/10.1111/cgf14050) and [webgpu-sky-atmosphere (MIT)](https://awesome.ecosyste.ms/projects/github.com%2Fjolifantobambla%2Fwebgpu-sky-atmosphere)
- [@takram/three-atmosphere README](https://cdn.jsdelivr.net/npm/@takram/three-atmosphere@0.18.0/README.md)
- [Procedural Tectonic Planets (Cortial et al. 2019)](https://diglib.eg.org/handle/10.1111/cgf13614)
- [Large-scale terrain authoring through interactive erosion simulation (Schott et al. 2023)](https://h-schott.github.io/publications/uplift/publi_uplift.html) and [StreamPowerErosion code (MIT)](https://github.com/H-Schott/StreamPowerErosion)
- [Cosmos Journeyer (AGPL-3.0)](https://github.com/BarthPaleologue/CosmosJourneyer)
- [TerrainView7: full-scale planet rendering in WebGPU](https://dev.to/the_lone_engineer/terrainview7-full-scale-planet-rendering-in-webgpu-emscripten-now-with-precomputed-atmospheric-18ik) and [Open world in the browser, part 22: clouds and meshlets](https://app.cinevva.com/blog/2026-05-07-open-world-browser-part-22-clouds-and-meshlets)
- [NASA Exoplanet Archive](https://exoplanetarchive.ipac.caltech.edu)
- [Mars MGS MOLA DEM 463 m (USGS)](https://astrogeology.usgs.gov/search/map/mars_mgs_mola_dem_463m) and [Moon LRO LOLA DEM 118 m (USGS)](https://astrogeology.usgs.gov/search/map/moon_lro_lola_dem_118m)
- [WebSplatter: Gaussian splatting across devices via WebGPU (2026)](https://arxiv.org/abs/2602.03207v1) (background on cross-device GPU variability)
