# Status: True looks like the real thing (surfaces and small bodies)

Owner thread: "Build plumes, frosts and small bodies". Covers the third V3 audit's items 4, 5, 6 and 10 (R9 to R12, R15, R16, R18, R32, R33): plumes above the limb, tholins and coloured frosts, hot things glowing in Visible, small bodies and comets, lava tube openings on the ground.

Only the owner thread edits this file. Files agreed on 24 Nov: `crates/sse-surface/src/plumes.rs`, a later `shapes.rs`, the tholin and frost cover in `cover.rs` (with "Build SSE4 proven core"); new files `packages/renderer/src/plumes.ts`, `ices.ts`, `hot-glow.ts`, `small-bodies.ts` and their GpuItem fields, filled in the renderer's `trueLook` (with "Build photoreal renderer"). The frost and tholin pass runs after the ground and before "Build giant bands, clouds and hazes"'s air layer. Material colours only through the data sheet's Look up (MP33 titan-tholin, MP34 SO2, sulphur, N2 and CH4 ice, CO2 frost).

| Package | State | Log |
|---|---|---|
| TRUE1 plumes part 1: the kernel's plume facts (ocean, polar cracks, speeds, height, rate) on `resolved_system` surface records | in review | packages/TRUE1.md |
| TRUE2 plumes part 2: drawn above the limb in System True | next | |
| TRUE3 hot things glow in Visible (Planck against the world's own daylight) | module written, drawing next | |
| TRUE4 tholins and coloured frosts | waits on data sheet MP33, MP34 | |
| TRUE5 small bodies and comets | not started | |
| TRUE6 lava tube openings on the ground (from `survey_caves`) | not started | |

## Live checks (audit R3, per body)
Not yet run. Note: the default Solar System (`packs/base-sol`) has no Enceladus, Io, Europa, Titan, Triton or Charon yet, so those lines cannot be judged on the live site until the big moons are added.
