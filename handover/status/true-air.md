# Status: Air in True (giant bands, clouds and hazes)

Owner thread: "Build giant bands, clouds and hazes". Covers: TA, the third V3 audit's atmosphere items (`handover/design/SSE4-v3-gaps.md`, third audit: R1 giant bands, R2 storms, R3 polar vortices, R4 oblateness, R6 cloud decks, R7 limb, R8 coloured hazes) and R2b part 2 (the cloud shell at height on the Body globe, handed over by the renderer thread on 24 Nov).

Only the owner thread edits this file. Logs are in `handover/packages/`. Files: `packages/renderer/src/true-air.ts` (and its test); hooks in `SystemChart.svelte` and the renderer's index are a few marked lines. Agreed 24 Nov: the renderer thread owns `packages/renderer`; this track owns `true-air.ts` and the coming `gpu-cloud-shell.ts`, may touch `gpu-globe.ts` and `gpu-air.ts` for R2b part 2 and add optional fields to `gpu.ts`; the plumes thread's `trueIces` tints the ground before the air is laid on.

| Package | State | Log |
|---|---|---|
| TA1: air in system True. Every world's True face is read through one `readTrueFace` (ground tiles, then today's clouds and haze from the weather op's field); a giant is its cloud decks, so its bands, storms and polar cyclones show; each deck coloured by its condensate's sheet colour, white where the sheet has none (said in the HUD) | done, merged (PR 444, main d03dd2d0) | packages/TA1.md |
| TA2: colour from what particles are made of: blocks `sse.light.particle_albedo` (anomalous diffraction) and `sse.light.slab_reflectance` (two-stream), `sse_light::haze`; and `sse.shape.spin_flattening` (Darwin-Radau) for oblateness. Golden-changing: waits for its slot | next (code written, in the thread's working tree until the slot) | |
| TA3: Titan orange and Jupiter's brown from the sheet's n,k rows (data sheet MP33: titan-tholin, jupiter-chromophore, nh4sh, h2so4) and the giants' chromophore haze (weather W10a) | waits on MP33 and W10a | |
| TA4: methane above the decks through `light_through_gas` (Uranus and Neptune blue) | waits on the light thread's fix: its methane rows read about 10⁴ too strong (reported 24 Nov) | |
| TA5: the cloud shell at height (R2b part 2) and the limb of air in system True | next after TA2 | |
| TA6: giants visibly flattened in True from TA2's block (GpuItem flatten along the body's frame) | after TA2 | |
| VP-A1 (visual polish 5.4): differential rotation and turning storms. `flowFace` carries a giant's face by the weather op's own winds: each row at its jet's speed, each storm as one piece at its centre's speed, turning at its winds' speed round it. `readTrueFace` returns the flow and the live storms with a giant's face; `GpuItem.surface.version` uploads new pixels. No kernel ask: the cells already carry each deck's wind (K-a not needed) | done, merged in PR 477: shows on the size comparison's turning globes in True, through the renderer's `trueLook`; Lineup3D now reads faces through `readTrueFace`, so giants there have their decks | packages/VP-A1.md |
| VP-A2: flow-carried detail and curls through the detail ladder | waits on the renderer's VP6 | |
| VP-A3: the air's rim and terminator in system True. `air-luts.ts` builds Hillaire's transmittance and multiple-scattering tables from the kernel's air optics; `gpu-air-shell.ts` draws a shell round each world with air (16 view steps, a few texture reads each); `readTrueAir` reads the air through `resolved_system` and `read_air_optics`; `TrueBody.air`, `GpuItem.air`; System3D takes `airs` from the system chart | done (Rayleigh only; hazes with VP-A5) | packages/VP-A3.md |
| VP-A4 part 1: clouds as their own layer. `withAir` returns `ground` (trueSurface's own picture, faces kept for the detail ladder's bake) and `clouds` (premultiplied, alpha their reflectance) beside the flat face; gpu.ts draws `clouds` as a lit layer over the ground | done | packages/VP-A3.md |
| VP-A4 part 2: each deck at its own height with offset shadows | waits on the weather op's layer `base_m` and `top_m` (after PR 418) | |
| Shared finish (renderer VP1, `finish.ts`): switch `gpu-globe.ts`'s physical branch to `startFinish` (exposure, AgX, glare) and drop its own tone mapping | when the renderer thread says `finish.ts` has landed | |

## Looks like the real thing (audit R3 checklist), as of TA1

Not yet run on the live site: the TA1 check is in its log (local build, WebGL2). Rerun on v4.starsystemx.com after each merge and record the date and the main commit here.
