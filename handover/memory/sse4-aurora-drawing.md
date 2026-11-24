---
name: sse4-aurora-drawing
description: SSE4 track VP-AU drawing (thread "Build aurorae and night lights", 24 Nov 2026): split with light and star glow threads, files, first PR, next packages
metadata:
  type: project
  modified: 2026-11-24T19:31:30.459Z
---
- Started 24 Nov 2026 19:12 UTC; Alex "yes to all" design 16 questions 11 to 15; Long exposure is the default, Eye the choice. Scope widened by Alex 19:13: lights on constructs (cities, objects in space) and drive plume glow; city lights = SSE3 render audit item 9. Colour by speed is O7's shift pass, not ours.
- Split: this thread draws; "Build light and nebulae" supplies MF9 part 2 facts (power_w total both hemispheres + power_by_hemisphere, width_deg full width, emissions {line, matter, wavelength_m, power_w, share, peak_height_m, thickness_m FWHM, colour chromaticity max 1}, footprints lead_deg, diffuse gets height and colour). "Light it gives off" card is the data sheet side's.
- Star glow thread: one shared flare reader (readLiveStar gains optional targets; use hits[].arrives_s, never recompute); shared flash cap capFlashes(events,t,clock,calm) in star-events.ts; no own exposure: VP1 finish.ts does lift and glare, my lift in aurora.ts is marked temporary.
- Storm aurora reader (star glow PR 460): readLiveStar(session, system, address, given, "system") fills targets from orbit facts (targetsFromFacts); use ejection.hits[].arrives_s. Flash cap capFlashes(events {start_s,size,rise_s,decay_s}[], t, clock, calm) -> {index,k}[] from packages/renderer.
- Files: packages/renderer/src/aurora.ts, gpu-aurora.ts, test aurora.test.ts, fixture tests/fixtures/aurora/proving-worlds.json, e2e tests/ui/aurora.e2e.mjs, status handover/status/aurora.md, log handover/packages/VP-AU5.md.
- PR 469 (VP-AU5 part 1, Body globe shells, oxygen stand-in only where O2 in air). Generated worlds draw nothing until MF2 gives magnetic moments.
- Constructs: lamps exist as emit Conditions (sodium/white with power); plume glow from sse_sense::fog::signature (RK14, PR 406).

Related: [[sse4-visual-polish]], [[sse4-star-look]], [[sse4-light-track]], [[sse4-magnetic-fields]].
