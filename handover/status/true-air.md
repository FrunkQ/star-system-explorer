# Status: Air in True (giant bands, clouds and hazes)

Owner thread: "Build giant bands, clouds and hazes". Covers: TA, the third V3 audit's atmosphere items (`handover/design/SSE4-v3-gaps.md`, third audit: R1 giant bands, R2 storms, R3 polar vortices, R4 oblateness, R6 cloud decks, R7 limb, R8 coloured hazes) and R2b part 2 (the cloud shell at height on the Body globe, handed over by the renderer thread on 24 Nov).

Only the owner thread edits this file. Logs are in `handover/packages/`. Files: `packages/renderer/src/true-air.ts` (and its test); hooks in `SystemChart.svelte` and the renderer's index are a few marked lines. Agreed 24 Nov: the renderer thread owns `packages/renderer`; this track owns `true-air.ts` and the coming `gpu-cloud-shell.ts`, may touch `gpu-globe.ts` and `gpu-air.ts` for R2b part 2 and add optional fields to `gpu.ts`; the plumes thread's `trueIces` tints the ground before the air is laid on.

| Package | State | Log |
|---|---|---|
| TA1: air in system True. Every world's True face is read through one `readTrueFace` (ground tiles, then today's clouds and haze from the weather op's field); a giant is its cloud decks, so its bands, storms and polar cyclones show; each deck coloured by its condensate's sheet colour, white where the sheet has none (said in the HUD) | done | packages/TA1.md |
| TA2: colour from what particles are made of: blocks `sse.light.particle_albedo` (anomalous diffraction) and `sse.light.slab_reflectance` (two-stream), `sse_light::haze`; and `sse.shape.spin_flattening` (Darwin-Radau) for oblateness. Golden-changing: waits for its slot | next (code written, in the thread's working tree until the slot) | |
| TA3: Titan orange and Jupiter's brown from the sheet's n,k rows (data sheet MP33: titan-tholin, jupiter-chromophore, nh4sh, h2so4) and the giants' chromophore haze (weather W10a) | waits on MP33 and W10a | |
| TA4: methane above the decks through `light_through_gas` (Uranus and Neptune blue) | waits on the light thread's fix: its methane rows read about 10⁴ too strong (reported 24 Nov) | |
| TA5: the cloud shell at height (R2b part 2) and the limb of air in system True | next after TA2 | |
| TA6: giants visibly flattened in True from TA2's block (GpuItem flatten along the body's frame) | after TA2 | |

## Looks like the real thing (audit R3 checklist), as of TA1

Not yet run on the live site: the TA1 check is in its log (local build, WebGL2). Rerun on v4.starsystemx.com after each merge and record the date and the main commit here.
