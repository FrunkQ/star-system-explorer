---
name: sse4-renderer-track
description: SSE4 renderer track (thread "Build photoreal renderer"): ownership, package order R1-R5, and what plugs in from physics
metadata:
  type: project
  modified: 2026-11-19T14:59:45.515Z
---
- Started 19 Nov 2026 at Alex's request as a fourth build track. Owns packages/renderer (frames.ts and gpu.ts shared, add-only) and mounts the renderer inside packages/views/body (agreed with the UI thread, which keeps the words panel, Cutaway and globe.ts).
- R1 (recipes, passes, governor, quality badge; Body globe on GPU/canvas/words) in PR 20, 19 Nov. Log: handover/packages/R1.md.
- Order agreed: R2 Body and descent views (gpu-high, linear HDR, Hillaire atmosphere in TSL, clouds from the weather track's ops, magnetic field overlay pass from B1 dipole data), R3 still tier and "take photograph", R4 Orac recipe editing, then R5 (proposed) exotic objects (accretion discs, jets, pulsar/magnetar fields) viewed per band through the O1 light switch.
- Alex (19 Nov): wants BH accretion discs and gamma views from raw physics through lenses, and an improved magnetic field overlay (V3 had one; don't copy V3 slavishly). Physics thread's B1 "one body" (plan in /mnt/project-files/sse4/SSE4-star-census.md section 3) will publish emission by band and a magnetic field (dipole moment, tilt, offset, geometry kind); weather ops come first after S5.

Related: [[sse4-build-conventions]]. Phone checks are emulated Moto G4; say so.

CI: don't edit the workflow; push once per PR head.
R1 to R4 part 1 merged 19 Nov; R4 part 2 waits on Orac (applyEdits() edit lists); R2b waits on weather; R5 band views read read_light/read_nebulae/read_activity.
- 20 Nov 01:36 UTC: R5a (stars drawn from physics on the Body screen, Visible/X-ray) merged as PR 51. Next: R5b discs/jets/gamma; R2b waits on weather PR 25; R2c waits on field data.
- Night sky (track Y): renderer owns Y5 (sky drawn; cube map per vantage on GPU later; caps ~20k points phone, ~200k desktop).

View styles: see [[sse4-view-styles]] (designed, building held).
Time-lapse: renderer owns layer passes, frames player, globe API; UI owns toggles, scrubber, why-card, formation replay (kernel op surface_frames).
- 22 Nov: magnetic fields design accepted (SSE4-magnetic-fields-design.md, track MF, PR 126). Renderer owns MF14a, the field layer (was R2c): a layer in every view style, not its own screen; field lines run down into the cutaway to the generating layer and follow the time ruler; can start on the Solar System fixture. R6 time-lapse (PR 123, 140): chart room's BodyTimeline replaces interim [data-lapse-controls]. R2b part 1 (PR 145): today's clouds/haze/storms from the weather op on the Body globe, laid on the ground; lapse.view added for BodyTimeline. Y5 part 1 (PR 156) SkyBeyond panel; part 2 (PR 159) Window 'Stars behind' via kernel system frame (PR 157), sensor limit capped 6.5. Pending: after kernel PR 158, pass max_stars per tier (20k phone/200k desktop) and call sky_glow with limit_mag=kept_to_mag when truncated. MF14a part 1 (PR 166): renderer/src/field.ts tracer, chartCut/cutPaths, project3D/drawField3D, FIELD_LINES; part 1b (PR 176): session.readField + fieldFromRead from read_field fixture 'solar-system' (match by name; needs base-reference). UI label is 'Mag' (Alex 22 Nov); MF14c accepted, Mag band entry driven by seat lens sensors. Next: R2b pt 2 cloud shell at height.
- MF14c system scale: read_field with {system, at_s} (PR 220) gives each planet a `glance` (held_off_by, bubble_m/radii, shielded, ground_dose_share, belts_m, field measured/computed/none/unresolved, dynamo_now).
- GH9: collision marks on galaxies in True (warps, ripples, punched holes, LMC-like survivors). Build after kernel GH3, with V9.
- Interface rules 11-12: zoom/slide/tilt + Free view; one shared ViewControls (chart-grammar, contract ui-contract/view-controls.ts, U25.md). New True/Wireframe: set view's controls.styles entry true, draw from viewOptions(spec, ws.view).shown; no separate switch.
