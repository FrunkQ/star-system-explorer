# Status files that exist only on open PR branches (copied 24 Nov 2026 21:10 UTC)

Each is the owner thread's own status file as it stands on an open PR branch, copied so it survives if the PR is
still open at the account move. Once that PR merges, the file in `handover/status/` is the truth and the copy here is
stale. Rows changed on open PRs for files that already exist on main are not copied; read the PR.

---

## `handover/status/citations.md` from PR 471 (branch `claude/project-thread-fg4jx3`)

### Status: Citation run-through

Owner thread: "Verify every cited paper". Covers: CR, the Consensus check of every paper cited in a "Why" (exists, and
says what SSE4 claims), the master list `handover/citations/master.jsonl` and its check. Procedure:
`handover/citations/README.md`. The per-area register `docs/citations.md` stays each builder's own record.

Only the owner thread edits this file. Index and standing rules: `handover/START-HERE.md`.

| Package | State | Log |
|---|---|---|
| CR1 master list and check: `scripts/citations.py` gathers author-year citations, DOIs and arXiv ids from packs, crates, packages, the worker and design notes; 1,465 papers in four tiers; CI job `citations` and `scripts/check.sh` fail on a pack or code citation not on the list (`--add` fixes it) | done (this PR) | handover/citations/README.md |
| CR2 the visual polish pass (Alex's Consensus run, PR 464) scored onto the list: 8 confirmed, Rhines 1975 "says something else" (π√(2U/β) is one jet's width; U must be the eddy speed) | done (this PR); raised to the coordinator | project files sse4/citations/results-00-visual-polish.md |
| CR3 daily batches of 30, tier 1 (marked from memory) first: batches 1 to 5 written to project files `sse4/citations/` | next: Alex carries batch 1 to a Claude with Consensus; from 25 Nov the new account runs the round itself | handover/citations/README.md |


---

## `handover/status/quality-tiers.md` from PR 470 (branch `claude/project-thread-p1jhqu`)

### Status: Quality tiers and speed probe

Owner thread: "Build quality tiers and speed probe" (split from "Build photoreal renderer", 24 Nov 2026). Covers: VP4 and VP5 of `design/SSE4-visual-polish-design.md` (sections 8.1 to 8.5): the `gpu-ultra` tier, the speed probe, placement by measured speed, the heat watch, quality override, per-pass lines in the performance readout, the drawing bench rows; then system True, Land, ships and stations under recipes and the governor; then sharpen at rest and enlarging on Light.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

Ownership agreed with the renderer thread (24 Nov): this thread owns `packages/renderer/src/probe.ts`, `probe-gpu.ts`, `bench-drawing.ts`, `recipes.ts` and `governor.ts`; `gpu.ts` is shared and additive-only; the shared finish (`finish.ts`, VP1) is the renderer thread's, and this thread adds the tier knob (`rungs`, glare size) to its `FinishFrame`. "Build detail ladder and world textures" owns `ladder.ts` and `texture-ladder.ts`.

| Package | State | Log |
|---|---|---|
| VP4 part 1: `gpu-ultra` in the Tier type and `TIER_ORDER`; the speed probe (capabilities, three scenes at one megapixel, timestamp or read-back timing, cached per adapter and browser); placement at the richest tier predicted within 60% of budget (WebGL2 can earn High, Ultra needs WebGPU with compute); heat watch; Quality Auto, Light, Full, Ultra and "Measure again" in the performance readout; per-pass estimated shares in the readout; the drawing rows in the bench and `docs/minimum-spec.md`. The Body globe is placed this way | done (this PR) | packages/VP4.md |
| VP4 part 2: system True, Land, ships and stations run on recipes and the governor, with the finish's glare rungs as the tier knob (B 3 at quarter size, H 6 at half); standard recipe per view timed in the bench | next: waits for VP1 `finish.ts` on main | packages/VP4.md |
| VP5: sharpen at rest in every True view (jittered accumulation, more steps when still, drop back on input; fade in under reduced motion); FSR1-style enlarging on Light | after VP4 part 2 | |


---

## `handover/status/aurora.md` from PR 469 (branch `claude/project-thread-b2k6we`)

### Status: Aurorae and night lights (drawing)

Owner thread: "Build aurorae and night lights". Covers the drawing side of track VP-AU (visual polish design `handover/design/SSE4-visual-polish-design.md` section 16, accepted by Alex 24 Nov 2026 19:12 UTC, "yes to all", "Long exposure" the default): aurora shells, curtains, aurora from the ground, other bands, airglow drawn; VP12 (city lights, lightning); night-side lights on constructs (cities on the ground and objects in space, Alex 24 Nov 19:13) and the glow of a drive's plume; item 9 of the SSE3 render audit (city lights). Colour by speed is the shift pass (O7), not this track; every light here passes through it like any other.

Only the owner thread edits this file. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

**The split (agreed with "Build light and nebulae", 24 Nov 2026 19:13 to 19:15 UTC):** this track draws; the light thread supplies the facts as MF9 part 2 (VP-AU1 to AU4, rows under MF9 in `status/light.md`). The data sheet side takes the "Light it gives off" card section on element and substance cards (question 15); not this track's. Part 2 only adds fields to `read_field`'s `aurora`, on each oval and footprint, and later on patches and airglow:
- `power_w` (total, both hemispheres) and `power_by_hemisphere` [north, south]; `width_deg` (full width, equatorward to poleward edge);
- `emissions`: `{line, matter, wavelength_m, power_w, share, peak_height_m, thickness_m (full height range above half peak), colour {hex, linear_srgb}}`, colour a chromaticity with max channel 1 (as every kernel colour), brightness in `power_w`; and `colour`, all emissions blended;
- footprints give `lead_deg` (the angle the spot runs ahead of its moon); the moon's longitude is taken from the system's positions;
- the diffuse glow gets `peak_height_m` and colour too.
`aurora.ts` reads all of these already (tested against the shape) and draws the kernel's lines the moment they arrive.

**Rule 5 (section 16.1):** a look is switched on by kernel numbers, never a name or class; it scales with them; it stays off where the physics says so; each package is proven on a Solar System body, a world outside it, and a world where it stays off. Aurora proving worlds: `tests/fixtures/aurora/proving-worlds.json`.

Files: `packages/renderer/src/aurora.ts`, `gpu-aurora.ts`, their test `packages/renderer/test/aurora.test.ts`, the `aurora` pass in `passes.ts` and the true-colour recipe's `aurora` entry. Small additive hooks, agreed as seams: an optional last argument `aurora` to `gpu-globe.ts` `draw`, `aurora` in `runtime.ts` `GlobeRunOptions` and `aurora_note` in its status, `aurora` in `photograph.ts` options, and the aurora read in `packages/views/body/src/Body.svelte` (renderer and chart room threads told). `gpu.ts` untouched.

| Package | State | Log |
|---|---|---|
| VP-AU5 part 1, glowing shells on the Body globe: each line of each oval a thin shell at its own height round the magnetic poles, limb-stacked, drowned by daylight; long exposure by default (one stated lift, ratios kept), Eye true brightness and grey below colour vision; oxygen stand-in only where the air holds oxygen, marked on screen | in review (PR to come) | packages/VP-AU5.md |
| VP-AU5 part 2: moon spots and tails from the system's moon positions and `lead_deg`; shells on system True (V10a) and System 3D worlds; the Exposure control moves to the shared view controls when VP1 lands | next | |
| VP-AU6 curtains through the detail ladder (VP6), pulsing under the three-flashes cap | queued (needs VP6) | |
| VP-AU7 aurora from the ground on the Land screen | queued (after AU5) | |
| VP-AU8 ribbons along field lines; volumes | queued (needs VP4) | |
| VP-AU9 other bands (UV, IR, X-ray) through the Band control | queued (needs part 2) | |
| Storm aurora drawn on the date an ejection arrives (VP-AU3 facts) | queued (needs part 2) | |
| Airglow drawn (VP-AU4 facts): the green limb line on oxygen worlds | queued (needs AU4) | |
| VP12 night side: city lights from population facts, lightning from the weather op | queued | |
| Construct lights: lamps from emit Conditions (sodium or white, with power) on the ground and in space; a drive plume's glow from `sse_sense::fog::signature` (RK14, PR 406) | queued | |


---

## `handover/status/detail-ladder.md` from PR 467 (branch `claude/project-thread-ftsszw`)

### Status: Detail ladder and world textures

Owner thread: "Build detail ladder and world textures" (split from "Build photoreal renderer", 24 Nov 2026). Covers: VP6 (the detail ladder) and VP3 (texture ladder and relief normals) of `handover/design/SSE4-visual-polish-design.md`, accepted by Alex 24 Nov 2026 18:57 UTC.

Only the owner thread edits this file. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

Files: `packages/views/body/src/finer-tiles.ts` (VP3 part 2), `packages/renderer/src/ladder.ts`, `ladder-noise.ts`, `ladder-node.ts`, `ladder-layers.json` (each area thread edits its own layer's entry), `texture-ladder.ts`, `texture-ladder-gpu.ts` (VP3), their tests `packages/renderer/test/ladder.test.ts`, `tests/ui/ladder-gpu.e2e.mjs` and `tests/ui/ladder-gpu/`. `gpu.ts` is shared and additive-only. `probe.ts`, `recipes.ts` and `governor.ts` belong to "Build quality tiers and speed probe".

| Package | State | Log |
|---|---|---|
| VP6 the detail ladder: rung table by screen size and tier, bindings to kernel values (missing value, nothing drawn), seed and clock, anchors folded on the processor, noises written once for processor and GPU (simplex with turning gradients, curl, cells, ridged, flow), TSL node, HUD detail line, six starter declarations | done | packages/VP6.md |
| VP3 part 1: texture ladder and relief normals: cube faces 64 to 2048 px by screen size, baked on the GPU from the kernel's tiles at any levels, relief normals at true scale, LRU by budget; System True wears it | done | packages/VP3.md |
| VP3 part 2: finer tiles (levels 1 and 2) for the chosen world, 4 tiles a request after a 1.2 s settle, stopping when not wanted, kept for the page in `views/body/src/finer-tiles.ts`; the chart room agreed the edit in `SystemChart.svelte`'s True surface reader | done | packages/VP3.md |
| VP3 part 2b: clouds as their own shell over the baked ground, so worlds with air keep their relief in True (with the air thread, true-air.ts) | next | |
| VP3 part 3: the Body screen and the body card's globe read `finerTiles` through the shared surface cache (after PR 428); the detail ladder's ground roughness below the tiles on the globe | next | |

No kernel change and no golden change.


---

## `handover/status/kernel-handoffs.md` from PR 463 (branch `claude/project-thread-bvlyq1`)

### Status: kernel hand-offs

Owner thread: "Build galaxy arm fix and atmosphere advice". Covers kernel packages handed off from the kernel
thread's queue ("Build kernel resolve and rules" keeps `orbits.rs`, `stability.rs`, `withdraw.rs`, the barycentre
work and the editing K-list). Only the owner thread edits this file. Index: `handover/START-HERE.md`.

| Package | State | Log |
|---|---|---|
| GH arm sign fix: arms trail (envelope arm phase and galaxy chart crests) | built, in the golden lane (after 403, BC1, 400, 418) | packages/GH3-leading-arm.md |
| GH3 leading arm: a dated `leading_arm` mark from a small backwards pass or meal; rarity check (1 in 100, Iye et al. 2019) | built, same PR | packages/GH3-leading-arm.md |
| ATMOS `atmosphere_advice`: the science expects (redox sets the outgassed mix), differs, clashes (E8) with lifetimes, hydrogen escape, one suggested batch; V3 air imported as hand-set | built, same PR | packages/ATMOS.md |
| V3 `bary:N` nodes onto the hierarchy on import (`adopt.rs`) | next, after the kernel thread's barycentre PR (BC1) merges | |


---

## `handover/status/small-bodies.md` from PR 459 (branch `claude/project-thread-ll6wwd`)

### Status: Small bodies up close (track SB)

Owner thread: "Build asteroids and rings up close" (took small bodies over from "Build plumes, frosts and small bodies" on 24 Nov 2026; that thread keeps plumes, frosts and hot glow). Brief: Alex, 24 Nov 2026 18:02 and 18:05 UTC: belts and rings are a probability field until you look at one member, which then resolves into a definite body (M, S, C, P and so on, rubble pile, contact binary, with a short history); a Psyche-like stripped core is pre-resolved because formation made it; contact binaries and the other kinds join the Add an object list.

Only the owner thread edits this file. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

#### The method (one way, no second)
`sse_formation::leftovers::Cloud::resolve(seed, k)` gives the k-th largest member's mass, size and orbit. `sse_small::resolve` (crate `crates/sse-small`) takes that and adds what the member is, from the seed and its address alone (stream `small.member.v1`): makeup, type label read off it, rubble pile or monolith, single, contact or separated pair, spin, figure, weathering, albedo and history. The largest member of a shattered world's debris is that world's bared core (`sse_small::notables`); nothing is stored, it is computed again on every ask. Every number is in `packs/base-small-bodies` (`provides.other.small_bodies`), cited in `docs/citations.md` (section "Small bodies") with simplifications in `docs/fudges.md` (rows SB-*).

#### Seams agreed (24 Nov)
- Kernel ("Build kernel resolve and rules"): SB2 adds `crates/sse-kernel/src/small.rs` and dispatch lines in `lib.rs` and `protocol.rs`, a cost row in `ops()`, contract cases appended to `tests/contract/requests.json`, a perf-chunks entry. After their golden split (PR 454) a new op only appends golden lines. Check member addresses do not collide with moon addresses under a planet (BC1 puts 36 bodies in base-sol).
- Traveller: a resolved member's facts are `mass_kg`, `radius_m`, `makeup.{metal,rock,carbon,ice,gas}`, so the resource books (PR 400) mine it unchanged once the kernel lists it at its own address.
- Editing ("Build planet and star editing"): op `make_small_body` {seed, kind, diameter_km or mass_kg, a_au, around?}; recipes in `provides.other.small_bodies.recipes` {id, label, kernel_kind, inputs [{id, default, unit (a sheet Units id), kind}], citation}: small-body, comet, contact-binary, binary-asteroid. Tell them when SB2 merges.
- Renderer ("Build photoreal renderer"): the mesh side is this track's, in a new file (not `small-bodies.ts`, which the plumes thread owns), an optional `figure` on TrueBody in `true-body.ts` (after PR 442) and an additive optional `mesh` on GpuItem in `gpu.ts`. Every stand-in in the fudge register.
- Shape: the shared flattening block `sse.shape.spin_flattening` is "Build giant bands, clouds and hazes"'s; this track's breakup-side figure (Maclaurin, Jacobi) is in `sse-small/src/figure.rs` and becomes a formula block `sse.shape.spin_figure` in its own formulas file with SB2.

| Package | State | Log |
|---|---|---|
| SB1 the resolve: crate `sse-small`, pack `base-small-bodies`, notables (a shattered world's core), 14 checks (radar pair shares, spin barrier, Chandrasekhar figures, Arrokoth-like lobes, rings) | done (this PR) | packages/SB1.md |
| SB2 kernel ops `small_body` {system, member address} and `make_small_body` (Add list), facts for the resource books, `sse.shape.spin_figure` block; golden lane (appends lines after PR 454) | next | |
| SB3 True draws figures: ellipsoids, two lobes with a neck, relief from the kernel's tiles, weathering colour; new renderer file plus the optional fields above | queued (after SB2 and PR 442) | |
| SB4 comets whole (agreed with the plumes thread 24 Nov): sublimation from the energy balance, coma switch-on distance, dust and ion tails, and their drawing (takes over VP-P3; reuse `plumeBrightness` in `renderer/src/plumes.ts` for the dust tail) (render audit R32) | queued | |
| SB5 type gradient across a belt (fudge SB-belt-gradient), per-member makeup from where it formed; asks proven core | queued | |
| VP-R1 ring shadow on the planet and the planet's shadow on the rings (visual polish design 5.8) | queued (needs the renderer's VP1 finish) | |
| VP-R2 rings' dark side, opposition surge, forward scatter (5.9); grains read a phase table per particle kind from size and the sheet's refractive index (ask K-e of the light thread), not the Jendersie and d'Eon droplet fit (citation check, PR 464) | queued (needs K-c particle size and K-e phase tables) | |
| VP-R3 gaps and wave trains at moons' computed resonances, wavy edges beside embedded moons (5.9) | queued (moons' orbits exist) | |
| VP-R4 ring particles near the camera, instanced (5.9) | queued (needs VP4) | |
| VP-R5 small-body shapes from spin and strength (this track's figure in `sse-small`, drawn by SB3), boulders through the detail ladder (5.10) | queued (needs VP6; folds into SB3) | |

Visual polish packages (VP-R1 to VP-R5) come from `handover/design/SSE4-visual-polish-design.md` section 12, accepted by Alex 24 Nov 2026 18:57 ("yes to all"). Its section 3 rule applies: every drawn detail is a fact, drawn from the numbers, or a camera effect; a flourish with no kernel number behind it is not drawn.


---

## `handover/status/cosmic-web.md` from PR 453 (branch `claude/project-thread-761nqm`)

### Status: Cosmic web view

Owner thread: "Build cosmic web view" (from 24 Nov 2026; it took the web over from "Build photoreal renderer"). Covers: V9w and what follows it for the web cube, the 3D and True looks of the foam map.

One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

Alex's ruling (23 Nov 2026): the cosmic web stays close to real; the void fraction goes toward the measured 77% as a cited editable number; the view must read as the 3D spiderweb of filaments, nodes and voids, not a "wrinkled rug". Lean into the physics: the kernel's numbers drive it, every shortcut is in `docs/fudges.md`.

Boundaries agreed 24 Nov 2026:
- This track owns `packages/renderer/src/web-cube.ts` and its test, `packages/views/foam-map/src/Web3D.svelte`, `crates/sse-cosmos/src/web_light.rs` and its tests, and the cube parts of `web_slice` in `crates/sse-kernel/src/cosmos.rs`.
- "Build photoreal renderer" keeps the rest of `packages/renderer` (ask it before touching `gpu.ts`).
- "Build galaxy charts and histories" owns `FoamMap.svelte` (flat slice, time scrub, ledger) and the cluster map; this track changes FoamMap only in small hunks that feed Web3D, with a heads-up first.
- The kernel thread owns the rest of sse-cosmos and sse-kernel; G2's `Haloes::cell` stays bit-identical (tested).

| Package | What it does | State | Log |
|---|---|---|---|
| V9w | Voids fill 77% of space (cited, editable `void_fraction`), one web for slice, cube and ledger; the foam map turns a cube of it in 3D and True | done (PR 368, by the renderer thread); voids hold about 40% of the matter against 15% measured (fudge) | packages/V9w.md |
| V9w2 | Light from haloes: True shines by the cube's galaxies, G2's haloes from 10¹¹ M☉ (pack `web_light.floor_msun`, the floor grows with the cube's volume) moved by the cube's own truncated Zel'dovich step, each lit by its G3 galaxy's stars and star formation then (old light warm, young blue); HUD adds the dark matter faintly; kernel `web_slice` with `cube` and `haloes` | in review | packages/V9w2.md |

#### Next
- Check the live view against Alex's ruling (spiderweb, not wrinkled rug) and fix what does not read that way.
- Galaxies in 3D (Chart) as well as True; void galaxies bluer and fainter by environment (Hoyle et al. 2005).
- A contract case for `haloes: true` in `tests/golden/contract.jsonl`, through the golden lane (the coordinator keeps the order).
- The voids' matter share toward 15% (NEXUS+ or a 2LPT step), a kernel change in the golden lane.


---

## `handover/status/true-surface.md` from PR 449 (branch `claude/project-thread-3jxtpq`)

### Status: True looks like the real thing (surfaces and small bodies)

Owner thread: "Build plumes, frosts and small bodies". Covers the third V3 audit's items 4, 5, 6 and 10 (R9 to R12, R15, R16, R18, R32, R33): plumes above the limb, tholins and coloured frosts, hot things glowing in Visible and lava tube openings on the ground (small bodies and comets handed over, see below).

Only the owner thread edits this file. Visual polish rule (`design/SSE4-visual-polish-design.md` section 3): every drawn detail is a Fact, Drawn from the numbers, or a Camera effect; a flourish with no kernel number behind it is not drawn. Files agreed on 24 Nov: `crates/sse-surface/src/plumes.rs`, a later `shapes.rs`, the tholin and frost cover in `cover.rs` (with "Build SSE4 proven core"); new files `packages/renderer/src/plumes.ts`, `ices.ts`, `hot-glow.ts`, `small-bodies.ts` and their GpuItem fields, filled in the renderer's `trueLook` (with "Build photoreal renderer"). The frost and tholin pass runs after the ground and before "Build giant bands, clouds and hazes"'s air layer. Material colours only through the data sheet's Look up (MP33 titan-tholin, MP34 SO2, sulphur, N2 and CH4 ice, CO2 frost).

| Package | State | Log |
|---|---|---|
| TRUE1 plumes part 1: the kernel's plume facts (ocean, polar cracks, speeds, height, rate) on `resolved_system` surface records | in review | packages/TRUE1.md |
| TRUE2 plumes part 2: drawn above the limb in System True | next | |
| TRUE3 hot things glow in Visible (Planck against the world's own daylight) | module written, drawing next | |
| TRUE4 tholins and coloured frosts | waits on data sheet MP33, MP34 | |
| TRUE5 small bodies and comets | handed over to "Build asteroids and rings up close" (24 Nov) | |
| TRUE6 lava tube openings on the ground (from `survey_caves`) | not started | |
| VP-P1 hot ground and lava as emitted light into the shared glare, crust from cell noise (visual polish design, accepted 24 Nov; extends TRUE3) | waits on VP1, VP6 | |
| VP-P2 plumes with size-driven forward scatter and the umbrella shape (extends TRUE2) | waits on kernel ask K-c | |
| VP-P3 comets (coma switch-on, two tails, green head) | goes with TRUE5 to "Build asteroids and rings up close" | |
| VP-P4 ice cracks and frost sparkle below the fact floor through the detail ladder | waits on VP6 | |

#### Live checks (audit R3, per body)
Not yet run. Note: the default Solar System (`packs/base-sol`) has no Enceladus, Io, Europa, Titan, Triton or Charon yet, so those lines cannot be judged on the live site until the big moons are added.

#### Small bodies: handed over
On 24 Nov the small-body part (TRUE5: the resolving field for belts and rings, shapes, contact binaries and pairs, Psyche-like bodies from formation, comets, the Add list entry) moved to the thread "Build asteroids and rings up close", with this track's approach and notes (the shape block, the formation records, the Traveller mining hook, what the editing thread wants). Those notes live in that thread's status file once it has one, and until then in project files `sse4/small-bodies-handover.md`.

#### Surface colours (TRUE4 scope, 24 Nov)
Coordinator finding from the renderer's Gallery on Sol: Mars reads grey-brown, not red, because tile colours come from a typed table (`packs/base-surface` `cover.colours`, regolith [158, 148, 132]). TRUE4 covers Mars's ferric red as well as tholins and frosts, by one mechanism: the kernel names what stains each surface (ferric oxide where iron met oxidising air or water, tholin where methane meets ultraviolet, sulphur and SO2 frost from volcanic gas, N2 and CH4 frost where they condense), and the colour comes from that material's data-sheet row (MP33, MP34) through the light code, never typed. Run the per-body checklist in the renderer's Gallery on an opened Sol.

#### Seams agreed after the visual polish design (24 Nov)
- **Detail ladder (VP6, "Build detail ladder and world textures"; log handover/packages/VP6.md).** VP-P1 and VP-P4 are entries in `packages/renderer/src/ladder-layers.json` (`ice.cracks` exists as a starter). Their largest size, amplitude and speed are kernel facts by name, and a missing fact draws nothing. Each frame: `resolveLayer(ladderLayer(name)!, facts)`, then `rungs(layer, discScale(radius_m, disc_px, view_px), tier, t_s, ladderSeed(address), anchor_m)`. On the GPU: `ladderNode(kind, localMetres, { normal, along })` from ladder-node.ts, with `lad.set(rungs, { axis, stretch })`. The HUD line is `detailLine([{ layer, shown }])`.
- **Grain scattering (visual polish section 17, citation check file).** It is not the Jendersie and d'Eon size fit, which holds only for water droplets of 5 to 50 µm. Plume and frost grains read a phase table per particle kind, computed by the light thread (ask K-e) from grain size and the sheet's refractive index. Until that lands, plumes.ts uses Henyey-Greenstein with g = 0.7 as a stand-in, which needs a fudge row. Copy the citations from `handover/design/SSE4-visual-polish-citation-check.md` into docs/citations with the status that file gives.
- **Comets (VP-P3)** go wholly to "Build asteroids and rings up close" (SB4: sublimation and tails, with the drawing).


---

## `handover/status/atomic-rockets.md` from PR 447 (branch `claude/project-thread-7xs7xp`)

### Status: Atomic Rockets (track RK)

Owner thread: "Build ship flow and power views" (from 24 Nov 2026). Covers: the RK packages handed on by the chart room
and renderer threads: RK13 (Flow view and Power card), RK12 (range rings as a HUD overlay), RK10 (remaining band
strings), RK3 (ship sizing formulas as named Equations). Other RK packages stay rows in their owners' files: RK9 in
`galaxy-and-travel.md`, RK6 in the Paper spacecraft pack's thread.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are
in `handover/packages/`. Plan: `handover/design/SSE4-atomic-rockets-plan.md` (accepted in full, questions 1 to 25).

| Package | State | Log |
|---|---|---|
| RK13: the general Flow view (`ui-contract/flow.ts`, `views/flow` Flow) and the Power card on a ship's card (three columns, "Nowhere" dashed band on the GM seat only, five readings, live starlight and load switches) | done 24 Nov; next: `reject_k` on the kernel's heat rows (golden lane); later users of Flow: a world's energy budget (weather), an element's journey (matter) | packages/RK13-flow-and-power.md |
| RK12 part 2: range rings as a HUD overlay: `ui-contract/range-rings.ts` (`rangeRings`, `ringPoints`, layer `range`), session `effectAtRange`, `system-chart/RangeRings.svelte`, `system-3d/range-rings.ts` | done 24 Nov; next: the chart room mounts them and adds `range` to `HUD_OVERLAY_LAYERS`; a pack's weapons feed them | packages/RK12-rings.md |
| RK10 part 2: band strings in ui-contract `light.ts` (BANDS, the band-to-view map) read `SHEET_BANDS`; `view-controls.ts` already did; renderer `GALAXY_BANDS` and `bandOf` too | done 24 Nov; left: star.ts and live-star.ts types (star glow thread's files) | packages/RK10-ui-strings.md |
| RK3: ship sizing formulas as named Equations in `packs/base-ships` (may change goldens: golden lane) | queued | |


---

## `handover/status/landing.md` from PR 437 (branch `claude/project-thread-zcty3l`)

### Status: landing run-through (track LR)

Owner thread: "Build landing run-through" (from 24 Nov 2026, taken over from the chart room). Covers Alex's milestone of
23 Nov 2026: "landing sequence and end to end run through ... the GM plotting the Players experience of the events".
The journey: the party's ship flies a voyage to a world, arrives, the world is revealed, the GM picks a site, the
ship descends and lands, the party looks about; players on their own seats see each step the GM sends.

Owned here: this file, `tests/ui/run-through.e2e.mjs` (the whole journey, GM and a player side by side), the hand-off
from Voyages to Land, and the Land screen's glue (`packages/views/surface/src/Surface.svelte` controls; the renderer
thread owns its drawing). Not owned: `shell/src/run/`, `Players.svelte`, `landing-table.e2e`, `party.e2e` (Run desk
thread, status/run-desk.md); `shell/src/chart/` (chart room); `nav/` (selector); the drawing in `renderer/` (renderer).
Only the owner thread edits this file. Earlier landing rows stay where they were written: chart-room.md ("What the table
is shown", "The table watches a landing", "Players see the party's ship"), galaxy-and-travel.md (party ship, blocker 4),
renderer.md (SL1, V10g, V10i, V10j, V10k), run-desk.md (RD1 reveal on arrival).

#### The journey on main 9a5912c (24 Nov 2026, walked by `run-through.e2e`)

| Step | GM | Players | State |
|---|---|---|---|
| Party's ship picked (Ships and stations) | yes | header "Aboard Beowulf, at Earth" | works |
| Voyage Earth to Mars, "Fly Beowulf there" | yes, At-world card with dangers | header follows to Mars | works |
| Arrival reveal | RD1 slip (PR 421) | through set_known | in PR 421 |
| Approach and orbit | At-world card (aerocapture, entry, powered landing) | nothing until sent | gap LR4 |
| Pick a site (Land here, landmark or anywhere) | yes, party's ship as lander | nothing until sent | works |
| Send this view | yes | Landing screen, same site and verdict on own kernel | works (send race fixed in RD1) |
| Descend | flown from the kernel's log | flies too, a few seconds behind | works (a torch ship flies down from orbit since LR1) |
| Land, look about, walk | yes, hour by Dawn/Midday/Dusk/Night (LR2) | follows the GM's pose | works; 3D ground a flat slab and night black, gap LR3 |
| Take off | yes | back to the site | works |

#### Gaps and packages

| Package | Owner | State | Notes |
|---|---|---|---|
| LR0 this file and `run-through.e2e` (the whole journey as one check) | here | done | not in CI (workflow edits need Alex's yes); in `npm run ui:e2e` |
| LR1 a ship that can fly down flies down: `sse-nav` `plan_down` flies the entry and, for a lander whose thrust is over its weight, a powered descent from the entry interface, keeping the powered one only when it ends better (method `engines-from-orbit`, the reason first in `why`). The far trader at Mars now lands ("engines from orbit, 7 min 50 s"); stock landers keep their entries | here | done | packages/LR1.md; no golden change |
| LR2 choosing the hour: the Land slider in the world's local hours with the local time beside it, and Dawn, Midday, Dusk, Night presses on the site card and the walk pad | here | done | packages/LR2.md |
| LR3 the ground at night and in 3D: True at night is black with no stars, 3D is a flat grey slab (headless WebGL2) | renderer | asked 24 Nov | |
| LR4 approach and orbit for the table: between arrival and the site the players see only the header; a "Beowulf in orbit of Mars" view (the world turning below, the At-world card's chosen act) for Send this view | here, with the Run desk thread for the preset | queued | |
| LR5 a whole landing in its link: `~surface?site=lat,lon&time=midday&lander=party` (place.ts); Land applies hour and lander with the site, and a site link is followed even with Land open, so a bookmark or Playbook preset is a place link | here | done | packages/LR2.md |
| LR5b the Land screen's own link and title: on Land the place (and so the address bar) carries the site, the quick hour and the lander as the GM picks them (`landingLinkSite`), and `here()`'s title names the landing ("Mars, Dribeque Planitia at midday", `landingTitle`), so a kept beat or a copied link brings it back. Bookmarks (PB2b `Whereabouts`) do not carry the site yet: Run desk thread told | here | done | packages/LR2.md |
| Gap 10, Mass on the Run desk card at Curious | Run desk thread (asking Alex) | open | |

#### Next
LR4 (an in-orbit view for the table), after PR 437 merges.


---

## `handover/status/galaxy.md` from PR 431 (branch `claude/project-thread-z535je`)

### Status: Galaxy charts and histories

Owner thread: "Build galaxy charts and histories" (split from "Build galaxy charts and travel screens" on 24 Nov 2026; travel keeps V8, voyages, the flight log and the travel panels). Covers: U6, U8, V9, GH8, V3 gap 39.

Files: `packages/views/galaxy-chart`, `packages/views/galaxy-3d`, `packages/views/foam-map`, `packages/views/cluster-map`, `packages/views/story-finder`, `packages/ui-contract/src/galaxy.ts`, `tests/ui/galaxy*.e2e.mjs`, `tests/ui/foam*`, `tests/ui/cluster*`, `tests/ui/stories*`. The True picture (GH9, `GalaxyScars`, renderer galaxy files) is the renderer thread's; the 3D and True web cube (`foam-map/src/Web3D.svelte`, renderer `web-cube.ts`) is "Build cosmic web view"'s.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

| Package | State | Log |
|---|---|---|
| U6 galaxy and patch charts (part 1: Galaxy under the Explore tab, from the `galaxy_chart` op) | parts 1, 2a (inked history preview) and 2c (what other galaxies brought, GH3) done; 24 Nov: the stellar halo row is `data-stellar-halo` (it clashed with the halo picker's `data-halo`); next: part 2b, patch survey, find a home, save inked history | packages/U6.md |
| V3 gap 39: system hover summary (GM only: counts of stars, planets, moons, minor bodies, ships, any life, anything remarkable; the selector of gap 37 reads the same summary) | part 1 done 24 Nov: pure reader `ui-contract/src/system-summary.ts` (`systemSummary`, `summaryWords`; reads scene subjects or picker chips; life only from life/biosphere tags or a Life chip, never guessed), test `system-summary.test.ts`; the navigation thread puts it on the picker's system rows; the hover on Your systems done with gap 17 part 1 | packages/GAP17-your-systems.md |
| GH8 leading-arm mark in the chart's history (design SSE4-galaxy-histories-design.md 9.4; rare, about 1 in 100, a cited editable number) | chart side done 24 Nov (`leadingArmWords`, the line and mark name in "Built from other galaxies"), hidden until the kernel's GH3 `leading_arm` mark merges (golden lane, "Build galaxy arm fix and atmosphere advice"); then an e2e | packages/GH8.md |
| V3 gap 17: Your systems, the campaign's systems on one map (U6 part 2b) | part 1 done 24 Nov: screen `systems` at the galaxy scale, view `sse.view.your-systems`; systems where their own V3 map puts them, the rest listed; hover says what each holds; press opens; loader `readCampaignSystems`; every system's row has a "Names here ›" door to the names gallery's `NamesHere` block (NM6 part 5); part 2a done 24 Nov: routes between systems from the record's links (`links_in`, `systemRoutes`; dashed with an arrow when one way), a grid under HUD (gap 18 in part), height stalks that join the map to the system; part 2b in part: "Stars around home", the home 10 pc cell's drawn stars from `star_cell` (read when opened; mass, companions, [Fe/H]; promoted ringed, campaign ones coloured); next: "Grow a system here" on a drawn star, waiting on the kernel's `from_star` on run_formation and adopt (kernel thread, golden lane, bundled with class_sample) | packages/GAP17-your-systems.md |
| V9 map views (Cosmic web foam under Explore from `web_slice` with time scrub and Play; cluster map "Galaxies nearby" on `halo_cell`) | parts 1 and 2 done | packages/V9.md |
| U8 story finder (Stories under Explore, on the `sift` op; starter sifters as data, hits with evidence, the GM's own sifter, a storyteller brief) | part 1 done; 24 Nov: "Tell it with Minerva" on each hit (ctx.describe, GM machine), the brief still copies for any storyteller; first event starter (a star changing phase); next: rarity (kernel passes clouds to sift), more event starters as writers appear, a story pack | packages/U8.md |
