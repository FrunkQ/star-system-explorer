# Status: True looks like the real thing (surfaces and small bodies)

Owner thread: "Build plumes, frosts and small bodies". Covers the third V3 audit's items 4, 5, 6 and 10 (R9 to R12, R15, R16, R18, R32, R33): plumes above the limb, tholins and coloured frosts, hot things glowing in Visible and lava tube openings on the ground (small bodies and comets handed over, see below).

Only the owner thread edits this file. Files agreed on 24 Nov: `crates/sse-surface/src/plumes.rs`, a later `shapes.rs`, the tholin and frost cover in `cover.rs` (with "Build SSE4 proven core"); new files `packages/renderer/src/plumes.ts`, `ices.ts`, `hot-glow.ts`, `small-bodies.ts` and their GpuItem fields, filled in the renderer's `trueLook` (with "Build photoreal renderer"). The frost and tholin pass runs after the ground and before "Build giant bands, clouds and hazes"'s air layer. Material colours only through the data sheet's Look up (MP33 titan-tholin, MP34 SO2, sulphur, N2 and CH4 ice, CO2 frost).

| Package | State | Log |
|---|---|---|
| TRUE1 plumes part 1: the kernel's plume facts (ocean, polar cracks, speeds, height, rate) on `resolved_system` surface records | in review | packages/TRUE1.md |
| TRUE2 plumes part 2: drawn above the limb in System True | next | |
| TRUE3 hot things glow in Visible (Planck against the world's own daylight) | module written, drawing next | |
| TRUE4 tholins and coloured frosts | waits on data sheet MP33, MP34 | |
| TRUE5 small bodies and comets | handed over to "Build asteroids and rings up close" (24 Nov) | |
| TRUE6 lava tube openings on the ground (from `survey_caves`) | not started | |

## Live checks (audit R3, per body)
Not yet run. Note: the default Solar System (`packs/base-sol`) has no Enceladus, Io, Europa, Titan, Triton or Charon yet, so those lines cannot be judged on the live site until the big moons are added.

## Small bodies: handed over
On 24 Nov the small-body part (TRUE5: the resolving field for belts and rings, shapes, contact binaries and pairs, Psyche-like bodies from formation, comets, the Add list entry) moved to the thread "Build asteroids and rings up close", with this track's approach and notes (the shape block, the formation records, the Traveller mining hook, what the editing thread wants). Those notes live in that thread's status file once it has one, and until then in project files `sse4/small-bodies-handover.md`.

## Surface colours (TRUE4 scope, 24 Nov)
Coordinator finding from the renderer's Gallery on Sol: Mars reads grey-brown, not red, because tile colours come from a typed table (`packs/base-surface` `cover.colours`, regolith [158, 148, 132]). TRUE4 covers Mars's ferric red as well as tholins and frosts, by one mechanism: the kernel names what stains each surface (ferric oxide where iron met oxidising air or water, tholin where methane meets ultraviolet, sulphur and SO2 frost from volcanic gas, N2 and CH4 frost where they condense), and the colour comes from that material's data-sheet row (MP33, MP34) through the light code, never typed. Run the per-body checklist in the renderer's Gallery on an opened Sol.
