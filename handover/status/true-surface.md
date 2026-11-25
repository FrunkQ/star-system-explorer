# Status: True looks like the real thing (surfaces and small bodies)

Owner thread: "Build plumes, frosts and small bodies". Covers the third V3 audit's items 4, 5, 6 and 10 (R9 to R12, R15, R16, R18, R32, R33): plumes above the limb, tholins and coloured frosts, hot things glowing in Visible and lava tube openings on the ground (small bodies and comets handed over, see below).

Only the owner thread edits this file. Visual polish rule (`design/SSE4-visual-polish-design.md` section 3): every drawn detail is a Fact, Drawn from the numbers, or a Camera effect; a flourish with no kernel number behind it is not drawn. Files agreed on 24 Nov: `crates/sse-surface/src/plumes.rs`, a later `shapes.rs`, the tholin and frost cover in `cover.rs` (with "Build SSE4 proven core"); new files `packages/renderer/src/plumes.ts`, `ices.ts`, `hot-glow.ts`, `small-bodies.ts` and their GpuItem fields, filled in the renderer's `trueLook` (with "Build photoreal renderer"). The frost and tholin pass runs after the ground and before "Build giant bands, clouds and hazes"'s air layer. Material colours only through the data sheet's Look up (MP33 titan-tholin, MP34 SO2, sulphur, N2 and CH4 ice, CO2 frost).

| Package | State | Log |
|---|---|---|
| TRUE1 plumes part 1: the kernel's plume facts (ocean, polar cracks, speeds, height, rate) on `resolved_system` surface records | done (PR 449) | packages/TRUE1.md |
| TRUE2 plumes part 2: drawn above the limb in System True | done (PR with TRUE3 and TRUE6) | packages/TRUE2.md |
| TRUE3 part 1 hot things glow in Visible (Planck against the world's own daylight, one temperature a world) | done (same PR); part 2 is VP-P1 (hot ground per tile) | packages/TRUE2.md |
| TRUE4 part 1 surface stains (rust, tholin, sulphur, frost) from the kernel, hued from the sheet's optics | done (PR 492); shows on Sol once proven core's SOL5 brings the catalogued air into the climate | packages/TRUE4.md |
| TRUE4 part 2 SO2 frost and sulphur rows on the sheet (MP34, agreed with the data sheet thread); per-tile stains | next, after the data sheet's PR 475 | |
| TRUE5 small bodies and comets | handed over to "Build asteroids and rings up close" (24 Nov) | |
| TRUE6 lava tube openings on the ground (from `survey_caves`): cut into the walk patch, named on the HUD; going inside parked to 4.2 | done (same PR) | packages/TRUE6.md |
| VP-P1 hot ground and lava as emitted light into the shared glare, crust from cell noise (visual polish design, accepted 24 Nov; extends TRUE3) | waits on VP1, VP6 | |
| VP-P2 plumes with size-driven forward scatter and the umbrella shape (extends TRUE2) | waits on kernel ask K-c | |
| VP-P3 comets (coma switch-on, two tails, green head) | goes with TRUE5 to "Build asteroids and rings up close" | |
| VP-P4 ice cracks and frost sparkle below the fact floor through the detail ladder | waits on VP6 | |

## Live checks (audit R3, per body)
- Earth (24 Nov, Node on the catalogued Sol): 288 K, plate tectonics, oceans; test-drive rough edge 1 is gone (packages/TRUE2.md).
- Lava tubes: the sample Moon's tube fields have lost all their roof to cratering, so its openings are rilles (packages/TRUE6.md, a question for the surface owner).
- The rest not yet run. Note: the default Solar System (`packs/base-sol`) has no Enceladus, Io, Europa, Titan, Triton or Charon yet, so those lines cannot be judged on the live site until the big moons are added.

## Small bodies: handed over
On 24 Nov the small-body part (TRUE5: the resolving field for belts and rings, shapes, contact binaries and pairs, Psyche-like bodies from formation, comets, the Add list entry) moved to the thread "Build asteroids and rings up close", with this track's approach and notes (the shape block, the formation records, the Traveller mining hook, what the editing thread wants). Those notes live in that thread's status file once it has one, and until then in project files `sse4/small-bodies-handover.md`.

## Surface colours (TRUE4 scope, 24 Nov)
Coordinator finding from the renderer's Gallery on Sol: Mars reads grey-brown, not red, because tile colours come from a typed table (`packs/base-surface` `cover.colours`, regolith [158, 148, 132]). TRUE4 covers Mars's ferric red as well as tholins and frosts, by one mechanism: the kernel names what stains each surface (ferric oxide where iron met oxidising air or water, tholin where methane meets ultraviolet, sulphur and SO2 frost from volcanic gas, N2 and CH4 frost where they condense), and the colour comes from that material's data-sheet row (MP33, MP34) through the light code, never typed. Run the per-body checklist in the renderer's Gallery on an opened Sol.

## Seams agreed after the visual polish design (24 Nov)
- **Detail ladder (VP6, "Build detail ladder and world textures"; log handover/packages/VP6.md).** VP-P1 and VP-P4 are entries in `packages/renderer/src/ladder-layers.json` (`ice.cracks` exists as a starter). Their largest size, amplitude and speed are kernel facts by name, and a missing fact draws nothing. Each frame: `resolveLayer(ladderLayer(name)!, facts)`, then `rungs(layer, discScale(radius_m, disc_px, view_px), tier, t_s, ladderSeed(address), anchor_m)`. On the GPU: `ladderNode(kind, localMetres, { normal, along })` from ladder-node.ts, with `lad.set(rungs, { axis, stretch })`. The HUD line is `detailLine([{ layer, shown }])`.
- **Grain scattering (visual polish section 17, citation check file).** It is not the Jendersie and d'Eon size fit, which holds only for water droplets of 5 to 50 µm. Plume and frost grains read a phase table per particle kind, computed by the light thread (ask K-e) from grain size and the sheet's refractive index. Until that lands, plumes.ts uses Henyey-Greenstein with g = 0.7 as a stand-in, which needs a fudge row. Copy the citations from `handover/design/SSE4-visual-polish-citation-check.md` into docs/citations with the status that file gives.
- **Comets (VP-P3)** go wholly to "Build asteroids and rings up close" (SB4: sublimation and tails, with the drawing).
